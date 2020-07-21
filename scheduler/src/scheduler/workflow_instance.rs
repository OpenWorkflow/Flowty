use std::convert::TryFrom;

use chrono::prelude::*;
use chrono::Duration;
use tokio::task;
use tokio::task::JoinHandle;
use tonic::Request;

use flowty_types;
use flowty_types::{Dag, FlowtyError};
use flowty_types::openworkflow::execution_broker_client::ExecutionBrokerClient;
use flowty_types::openworkflow::{
	Task,
	Execution,
	SearchRequest,
	ExecutorDefinition,
	ExecutorKind,
	LocalSpecification,
	ExecutionStatus
};

use crate::utils;
/*
	RunState is a state automaton:
	Nothing => Queued
	Queued => Running
	Running => {Success, Failed}
	Failed => Queued
*/
pub enum RunState {
	Nothing,
	Queued,
	Running,
	Success,
	Failed,
}

pub struct WorkflowInstance {
	wiid: i32,
	workflow_id: String,
	run_state: RunState,
	run_date: DateTime<Utc>,
	dag: Dag,
}

impl WorkflowInstance {
	pub async fn new(
		sql_client: &tokio_postgres::Client,
		workflow_id: &String,
		tasks: &Vec<Task>,
		run_date: DateTime<Utc>
	) -> Result<WorkflowInstance, FlowtyError> {
		let result = sql_client.query_one(include_str!("new_workflow_instance.sql"), &[workflow_id, &run_date.to_rfc3339()]).await;
		match result {
			Ok(row) => {
				let dag = Dag::try_from(tasks);
				match dag {
					Ok(dag) => {
						Ok(WorkflowInstance {
							wiid: row.get("wiid"),
							workflow_id: workflow_id.to_string(),
							run_state: RunState::Nothing,
							run_date,
							dag,
						})
					},
					Err(e) => {
						Err(e)
					}
				}
			},
			Err(e) => {
				error!("Failed to insert workflow_instance into database:{}\nScheduler state might de-sync!", e);
				Err(FlowtyError::ParsingError)
			},
		}
	}

	/// Update the internal run_state and the run_state in the DB.
	/// Does not perform any checks!
	async fn update_run_state(&mut self, sql_client: &tokio_postgres::Client, run_state: RunState) {
		let state: &str = (match run_state {
			RunState::Nothing => "nothing",
			RunState::Queued => "queued",
			RunState::Running => "running",
			RunState::Success => "success",
			RunState::Failed => "failed",
		}).into();
		let result = sql_client.execute(include_str!("update_run_state.sql"), &[&self.wiid, &state]).await;
		match result {
			Err(e) => error!("Failed to update workflow_instance:{}\nScheduler state might de-sync!", e),
			_ => (),
		};
		self.run_state = run_state;
	}

	pub async fn queue(&mut self, sql_client: &tokio_postgres::Client) {
		if matches!(self.run_state, RunState::Queued | RunState::Running | RunState::Success) {
			return;
		}
		self.update_run_state(sql_client, RunState::Queued).await;
		/*
			Err(e) => {
				error!("Failed to find an apprioriate executor: {}", e);
				let result = sql_client.execute(
					"UPDATE workflow_instance SET run_state = 'failed', modified_at = NOW() WHERE wiid = $1",
					&[&self.wiid]
				).await;
				match result {
					Err(e) => error!("Failed to update workflow_instance:{}\nScheduler state might de-sync!", e),
					_ => (),
				};
				self.run_state = RunState::Failed;
			}
		*/
	}

	pub async fn run(&mut self, sql_client: &tokio_postgres::Client) {
		info!("Starting workflow '{}'", self.workflow_id);

		match self.dag.next() {
			Some(next_tasks) => {
				for task in next_tasks {
					let ti = self.dag.get_task_instance(task);
					match ti.get_executor_definition() {
						Ok(executor_definition) => {
							let executor = find_executor().await;
							let handle = task::spawn_blocking(|| {

							});
						},
						Err(fe) => {
							// todo: Fail Task
						}
					};
				}
			},
			None => self.finish(sql_client).await,
		};
	}

	pub async fn finish(&mut self, sql_client: &tokio_postgres::Client) {

	}

	pub fn get_run_state(&self) -> &RunState {
		&self.run_state
	}
}

/// Asks the ExecutionBroker for a fitting executor.
/// Returns the URI to the executor
async fn find_executor(definition: &ExecutorDefinition) -> Result<String, FlowtyError> {
	let broker_uri = utils::get_env("EXECUTION_BROKER_URI", "http://[::1]:50051".into());
	match ExecutionBrokerClient::connect(broker_uri).await {
		Ok(client) => {
			match client.find_executor(Request::new(SearchRequest {
				executor_definition: Some(ExecutorDefinition {
					kind: ExecutorKind::Local.into(),
					specs: Some(executor_definition::Specs::Local(LocalSpecification{packages: vec![]}))
				})
			})).await {
				Ok(response) => Ok(response.into_inner().uri),
				Err(e) => Err(FlowtyError::ExecutorNotFound)
		}
		},
		_ => Err(FlowtyError::ExecutionBrokerUnreachable)
	}
}
