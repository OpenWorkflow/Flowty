use chrono::prelude::*;

use tonic::Request;

use crate::utils;
use super::FlowtyError;
use super::openworkflow::execution_broker_client::ExecutionBrokerClient;
use super::openworkflow::{
	SearchRequest,
	ExecutorDefinition,
	executor_definition,
	ExecutorKind,
	LocalSpecification
};

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
	run_state: RunState,
	run_date: DateTime<Utc>,
	//last_task: TaskInstance,
}

impl WorkflowInstance {
	pub async fn new(sql_client: &tokio_postgres::Client, workflow_id: &String, run_date: DateTime<Utc>) -> Result<WorkflowInstance, tokio_postgres::Error> {
		let result = sql_client.query_one("INSERT INTO workflow_instance (workflow_id, run_date)
			VALUES ($1, $2) RETURNING wiid;", &[workflow_id, &run_date.to_rfc3339()]).await;
		match result {
			Ok(row) => {
				Ok(WorkflowInstance{wiid: row.get("wiid"), run_state: RunState::Nothing, run_date: run_date})
			},
			Err(e) => {
				error!("Failed to insert workflow_instance into database:{}\nScheduler state might de-sync!", e);
				Err(e)
			},
		}
	}

	pub async fn queue(&mut self, sql_client: &tokio_postgres::Client) {
		if matches!(self.run_state, RunState::Queued | RunState::Running | RunState::Success) {
			return;
		}

		match find_executor().await {
			Ok(uri) => {
				let result = sql_client.execute(
					"UPDATE workflow_instance SET run_state = 'queued', modified_at = NOW() WHERE wiid = $1",
					&[&self.wiid]
				).await;
				match result {
					Err(e) => error!("Failed to update workflow_instance:{}\nScheduler state might de-sync!", e),
					_ => (),
				};
				self.run_state = RunState::Queued;
			},
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
		};
	}

	pub fn get_run_state(&self) -> &RunState {
		&self.run_state
	}
}

async fn find_executor() -> Result<String, FlowtyError> {
	let broker_uri = utils::get_env("EXECUTION_BROKER_URI", "http://[::1]:50051".into());
	let mut client = ExecutionBrokerClient::connect(broker_uri).await.unwrap();
	match client.find_executor(Request::new(SearchRequest {
			executor_definition: Some(ExecutorDefinition {
				kind: ExecutorKind::Local.into(),
				specs: Some(executor_definition::Specs::Local(LocalSpecification{packages: vec![]}))
			})
		})).await {
		Ok(response) => {
			Ok(response.into_inner().uri)
		},
		Err(e) => {
			error!("Failed to find executor for ");
			Err(FlowtyError::ExecutorNotFound {
				task: "",
				workflow: "",
				message: e,
			})
		}
	}
}
