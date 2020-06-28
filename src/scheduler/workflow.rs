use std::io::Cursor;

//use chrono::{DateTime, Utc};
use chrono::prelude::*;
use prost::{Message, DecodeError};

pub mod openworkflow {
    include!(concat!(env!("OUT_DIR"), "/openworkflow.workflow.rs"));
}

pub fn openworkflow_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<openworkflow::Workflow, DecodeError> {
	let c = std::fs::read(path).unwrap();
	let c = Cursor::new(c);
	Message::decode(c)
}

pub struct WorkflowVersion {
	// Expose the underlying OpenWorkflow message
	pub workflow: openworkflow::Workflow,
	version: u32,
	parse_date: DateTime<Utc>,
}

impl WorkflowVersion {
	fn new(openworkflow: openworkflow::Workflow, version: u32) -> WorkflowVersion {
		WorkflowVersion{
			workflow: openworkflow,
			version: version,
			parse_date: Utc::now(),
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

pub struct Workflow {
	workflows : Vec<WorkflowVersion>,
	last_tick: Option<DateTime<Utc>>,
}

impl Workflow {
	pub fn new() -> Workflow {
		Workflow{workflows: Vec::new(), last_tick: None}
	}

	pub fn from_openworkflow(workflow: openworkflow::Workflow) -> Workflow {
		Workflow{workflows: vec![WorkflowVersion::new(workflow, 0)], last_tick: None}
	}

	pub fn get_latest_version(&self) -> u32 {
		match self.workflows.iter().map(|w| w.version).max() {
			Some(v) => v,
			None => 0,
		}
	}
	pub fn get_latest_workflow(&self) -> Option<&WorkflowVersion> {
		self.get_workflow(self.get_latest_version())
	}
	pub fn get_workflow(&self, version: u32) -> Option<&WorkflowVersion> {
		self.workflows.iter().filter(|w| w.version == version).last()
	}

	pub fn add_workflow(&mut self, openworkflow: openworkflow::Workflow) {
		match self.get_latest_workflow() {
			Some(w) => {
				if w.workflow != openworkflow {
					self.workflows.append(&mut vec![WorkflowVersion::new(openworkflow, w.version + 1)]);
				} else {
					trace!("Workflow '{}' already present", w.workflow.workflow_id);
				}
			},
			_ => {
				self.workflows.append(&mut vec![WorkflowVersion::new(openworkflow, 0)]);
			}
		}
	}
}

/*
pub struct WorkflowInstance {
	pub workflow : Workflow,
	version : u32,
	run_state: RunState,
	run_date: DateTime<Utc>,
}
*/