extern crate cron;

use std::io::Cursor;
use std::str::FromStr;

use cron::Schedule;
use chrono::prelude::*;

use prost::{Message, DecodeError};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

fn validate_openworkflow(openworkflow: Result<openworkflow::Workflow, DecodeError>) -> Result<openworkflow::Workflow, DecodeError> {
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

		self.schedule = Schedule::from_str(openworkflow.schedule.as_str().clone()).unwrap();
		self.workflow = openworkflow;
		if reset_tick {
			self.last_tick = None;
		}

	}

	pub fn tick(&mut self, now: DateTime<Utc>) {
		if self.last_tick.is_none() {
			trace!("First tick");
			self.last_tick = Some(now);
			return;
		}

		let active_instances: Vec<&WorkflowInstance> = self.workflow_instances
			.iter()
			.filter(|i| {
				match i.run_state {
					RunState::QUEUED | RunState::RUNNING(_) => true,
					_ => false
				}
			})
			.collect();
		if active_instances.len() as u32 >= self.workflow.max_active_runs {
			trace!("Amount of active runs exceeds max active runs for '{}': {} >= {}",
				self.workflow.workflow_id,
				active_instances.len(),
				self.workflow.max_active_runs);
			return;
		}

		for workflow_instance in self.workflow_instances.iter() {

		}
	}
}

enum RunState {
	NOTHING,
	QUEUED,
	RUNNING(std::string::String),  // task_id
	SUCCESS,
	FAILED,
}

pub struct WorkflowInstance {
	run_state: RunState,
	run_date: DateTime<Utc>,
	//last_task: TaskInstance,  // Copy of task in case Weak ptr gets disassociated during exec
}

impl WorkflowInstance {
	pub fn new(run_date: DateTime<Utc>) -> WorkflowInstance {
		WorkflowInstance{run_state: RunState::NOTHING, run_date: run_date}
	}
}
