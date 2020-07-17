use std::convert::TryInto;
use std::io::Cursor;
use petgraph::{Graph, Direction};
use petgraph::graph::NodeIndex;
use petgraph::algo;
use chrono::Duration;
use prost::{Message, DecodeError};

use openworkflow::{Execution, Task, RunCondition, ExecutionStatus};

// ===== ===== ===== ===== ===== \\
// protobuf interactions
// ===== ===== ===== ===== ===== //
pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

pub fn validate_openworkflow(openworkflow: Result<openworkflow::Workflow, DecodeError>) -> Result<openworkflow::Workflow, DecodeError> {
	// #TODO
	match openworkflow {
		Ok(w) => {
			Ok(w)
		},
		Err(e) => Err(e)
	}
}

pub fn openworkflow_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<openworkflow::Workflow, DecodeError> {
	let c = std::fs::read(path).unwrap();
	let c = Cursor::new(c);
	validate_openworkflow(Message::decode(c))
}

pub fn openworkflow_from_binary(binary: &[u8]) -> Result<openworkflow::Workflow, DecodeError> {
	let c = Cursor::new(binary);
	validate_openworkflow(Message::decode(c))
}

// ===== ===== ===== ===== ===== \\
// Dag
// ===== ===== ===== ===== ===== //
// From implements the standard cast
// Everything not matching will be defaulted to None.
impl From<i32> for RunCondition {
	fn from(condition: i32) -> Self {
		match condition {
			1 => RunCondition::AllDone,
			2 => RunCondition::OneDone,
			3 => RunCondition::AllSuccess,
			4 => RunCondition::OneSuccess,
			5 => RunCondition::AllFailed,
			6 => RunCondition::OneFailed,
			_ => RunCondition::None,
		}
	}
}

#[derive(PartialEq, Clone)]
pub struct TaskInstance {
	task_id: String,
	retries: u32,
	max_retries: u32,
	retry_interval: Duration,
	execution_details: Execution,
	execution_status: Option<ExecutionStatus>,
	run_condition: RunCondition,
	downstream_tasks: Vec<String>,
}

type Node = TaskInstance;
type Edge = RunCondition;
pub struct Dag {
	graph: Graph::<Node, Edge>,
	task_instances: Vec<TaskInstance>,
}

impl Dag {
	pub fn from_tasks(tasks: &Vec<Task>) -> Result<Dag, FlowtyError> {
		let mut graph = Graph::<Node, Edge>::new();
		for task in tasks {
			if task.execution.is_none() {
				return Err(FlowtyError::ParsingError);
			}
			let retry_interval = Duration::from_std(
				task.retry_interval
				.clone()
				.unwrap_or_default()
				.try_into()
				.unwrap_or_default()
			).unwrap();
			let ti = TaskInstance {
				task_id: task.task_id.clone(),
				retries: 0,
				max_retries: task.retries,
				retry_interval,
				execution_details: task.execution.clone().unwrap(),
				execution_status: None,
				run_condition: RunCondition::from(task.condition),
				downstream_tasks: task.downstream_tasks.clone(),
			};
			graph.add_node(ti);
		}

		for parent_index in graph.node_indices() {
			let ti = &graph[parent_index].clone();
			for downstream_task in &ti.downstream_tasks {
				match graph.node_indices().find(|i| graph[*i].task_id == *downstream_task) {
					Some(child_index) => {
						graph.update_edge(parent_index, child_index, graph[child_index].run_condition);
					},
					None => ()
				};
			}
		}

		if algo::is_cyclic_directed(&graph) {
			return Err(FlowtyError::CyclicDependencyError);
		}

		Ok(Dag{
			graph,
			task_instances: Vec::new(),
		})
	}

	pub fn get_roots(&self) -> Vec<NodeIndex> {
		self.graph.externals(Direction::Incoming).collect()
	}
}

pub fn task_instance_is_ready(ti: &TaskInstance) -> bool {
	match ti.execution_status {
		None | 
		Some(ExecutionStatus::Initializing) | 
		Some(ExecutionStatus::Running) => true,
		_ => false,
	}
}

impl Iterator for Dag {
	type Item = Vec<NodeIndex>;

	fn next(&mut self) -> Option<Self::Item> {
		let mut stage: Self::Item = Vec::new();
		let mut downstream: Vec<String> = Vec::new();
		for node in algo::toposort(&self.graph, None).unwrap() {
			let downstream_tasks = &self.graph[node].downstream_tasks;
			if downstream.contains(&self.graph[node].task_id) {
				match self.graph[node].run_condition {
					RunCondition::OneDone => {
						let parents = self.graph.edges_directed(node, Direction::Incoming)
							.map(|edge| (edge.source()));

					},
				}

				if edge.weight() == &RunCondition::AllDone {
					downstream.append(&mut downstream_tasks.clone());
					stage.push(node);
					continue;
				}
				continue;
			}
			if task_instance_is_ready(&self.graph[node]) {
				for edge in self.graph.edges_directed(node, Direction::Incoming) {
					
				}
			}
		}

		if stage.len() == 0 {
			None
		} else {
			Some(stage)
		}
	}
}

// ===== ===== ===== ===== ===== \\
// Errors
// ===== ===== ===== ===== ===== //
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum FlowtyError {
	#[snafu(display("Failed to execute task: {}", message))]
	ExecutionError {
		message: String,
	},
	#[snafu(display("Failed to parse DAG"))]
	ParsingError,
	#[snafu(display("No executor found for task '{}' in workflow '{}': {}", task, workflow, message))]
	ExecutorNotFound {
		task: String,
		workflow: String,
		message: String,
	},
	#[snafu(display("Cyclic dependency detected!"))]
	CyclicDependencyError,
}
