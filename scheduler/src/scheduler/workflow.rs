use std::io::Cursor;

use chrono::prelude::*;

use prost::{Message, DecodeError};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

#[allow(dead_code)]
pub fn openworkflow_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<openworkflow::Workflow, DecodeError> {
	let c = std::fs::read(path).unwrap();
	let c = Cursor::new(c);
	Message::decode(c)
}

pub fn openworkflow_from_binary(binary: &[u8]) -> Result<openworkflow::Workflow, DecodeError> {
	let c = Cursor::new(binary);
	Message::decode(c)
}

#[derive(Clone)]
pub struct Workflow {
	pub workflow: openworkflow::Workflow,
	last_tick: Option<DateTime<Utc>>,
}

impl Workflow {
	pub fn new(workflow: openworkflow::Workflow) -> Workflow {
		Workflow{workflow: workflow, last_tick: None}
	}

	pub fn update_workflow(&mut self, openworkflow: openworkflow::Workflow, reset_tick: bool) {
		self.workflow = openworkflow;
		if reset_tick {
			self.last_tick = None;
		}
	}
}

/*
enum RunState {
	NOTHING,
	QUEUED,
	RUNNING(std::string::String),  // task_id
	SUCCESS,
	FAILED,
}

pub struct WorkflowInstance {
	pub workflow : &Workflow,
	version : u32,
	run_state: RunState,
	run_date: DateTime<Utc>,
}
*/