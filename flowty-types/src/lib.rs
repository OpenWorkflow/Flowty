use std::convert::TryInto;
use std::io::Cursor;
use std::collections::HashMap;

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
	Message::decode(c)
}

// ===== ===== ===== ===== ===== \\
// Dag
// ===== ===== ===== ===== ===== //
pub fn i32_to_run_condition(condition: i32) -> Option<RunCondition> {
	match condition {
		0 => Some(RunCondition::AllDone),
		1 => Some(RunCondition::OneDone),
		2 => Some(RunCondition::AllSuccess),
		3 => Some(RunCondition::OneSuccess),
		4 => Some(RunCondition::AllFailed),
		5 => Some(RunCondition::OneFailed),
		_ => None
	}
}

pub struct TaskInstance {
	retries: u32,
	max_retries: u32,
	retry_interval: Duration,
	execution_details: Execution,
	execution_status: Option<ExecutionStatus>,
	run_condition: Option<RunCondition>,
	downstream_tasks: Vec<String>,
}

// Todo: Optimize
fn check_dependencies(
	graph: &HashMap<String, TaskInstance>, task_id: String, downstream_tasks: Vec<String>
) -> bool {
	for d in graph.get(&task_id).unwrap().downstream_tasks.iter() {
		if downstream_tasks.contains(&d) {
			return false;
		}
		let mut dd = downstream_tasks.clone();
		dd.push(task_id.clone());
		if check_dependencies(graph, d.into(), dd) == false {
			return false;
		}
	}
	true
}

pub struct Dag (HashMap<String, TaskInstance>);

impl Dag {
	pub fn from_tasks(tasks: &Vec<Task>) -> Result<Dag, FlowtyError> {
		// Add tasks
		let mut graph = HashMap::with_capacity(tasks.len());
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
				retries: 0,
				max_retries: task.retries,
				retry_interval: retry_interval,
				execution_details: task.execution.clone().unwrap(),
				execution_status: None,
				run_condition: i32_to_run_condition(task.condition),
				downstream_tasks: task.downstream_tasks.clone(),
			};

			graph.insert(task.task_id.clone(), ti);
		}

		// Check for cycles
		for (task_id, _) in graph.iter() {
			if check_dependencies(&graph, task_id.into(), Vec::new()) == false {
				return Err(FlowtyError::CyclicDependencyError);
			}
		}

		Ok(Dag(graph))
	}

	pub fn get_next_for_execution(&self) -> &TaskInstance {

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
