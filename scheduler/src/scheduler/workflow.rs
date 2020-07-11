extern crate cron;

use std::io::Cursor;
use std::str::FromStr;

use cron::Schedule;
use chrono::prelude::*;

use prost::{Message, DecodeError};

use super::openworkflow;
use super::workflow_instance::{WorkflowInstance, RunState};

fn validate_openworkflow(openworkflow: Result<openworkflow::Workflow, DecodeError>) -> Result<openworkflow::Workflow, DecodeError> {
	// #TODO
	match openworkflow {
		Ok(w) => {
			Ok(w)
		},
		Err(e) => Err(e)
	}
}

#[allow(dead_code)]
pub fn openworkflow_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<openworkflow::Workflow, DecodeError> {
	let c = std::fs::read(path).unwrap();
	let c = Cursor::new(c);
	validate_openworkflow(Message::decode(c))
}

pub fn openworkflow_from_binary(binary: &[u8]) -> Result<openworkflow::Workflow, DecodeError> {
	let c = Cursor::new(binary);
	Message::decode(c)
}

pub struct Workflow {
	pub workflow: openworkflow::Workflow,
	schedule: Schedule,
	last_tick: Option<DateTime<Utc>>,
	workflow_instances: Vec<WorkflowInstance>,
}

impl Workflow {
	pub fn new(workflow: openworkflow::Workflow) -> Workflow {
		let schedule = Schedule::from_str(workflow.schedule.as_str().clone()).unwrap();
		Workflow{workflow: workflow, schedule: schedule, last_tick: None, workflow_instances: vec![]}
	}

	pub fn update_workflow(&mut self, openworkflow: openworkflow::Workflow, reset_tick: bool) {
		if openworkflow == self.workflow {
			return;
		}

		if self.workflow.schedule != openworkflow.schedule {
			warn!("Changing schedule. This is can cause undefined behaviour and is not recommend!");
			// ToDo: Actually make it work nicely
			self.schedule = Schedule::from_str(openworkflow.schedule.as_str().clone()).unwrap();
		}
		self.workflow = openworkflow;
		if reset_tick {
			self.last_tick = None;
		}
	}

	pub async fn tick(&mut self, sql_client: &tokio_postgres::Client, now: DateTime<Utc>) {
		if self.last_tick.is_none() {
			trace!("First tick");
			self.last_tick = Some(now);
			return;
		}

		let active_instances: Vec<&WorkflowInstance> = self.workflow_instances
			.iter()
			.filter(|i| matches!(i.get_run_state(), RunState::Queued | RunState::Running))
			.collect();
		if active_instances.len() as u32 >= self.workflow.max_active_runs {
			trace!("Amount of active runs exceeds max active runs for '{}': {} >= {}",
				self.workflow.workflow_id,
				active_instances.len(),
				self.workflow.max_active_runs);
			return;
		}
		let remaining_slots = self.workflow.max_active_runs as usize - active_instances.len();
		trace!("{} remaining slots for workflow '{}'", remaining_slots, self.workflow.workflow_id);

		for instance in self.schedule.after(&self.last_tick.unwrap()).take(remaining_slots) {
			if instance > now {
				break;
			}
			info!("Creating workflow instance for '{}' at time {}", self.workflow.workflow_id, instance);
			if let Ok(mut wi) = WorkflowInstance::new(sql_client, &self.workflow.workflow_id, instance).await {
				wi.queue(sql_client).await;
				self.workflow_instances.push(wi);
			}
		}
		self.last_tick = Some(now);
	}
}
