use std::io::Cursor;

use chrono::prelude::*;

use prost::{Message, DecodeError};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

pub fn openworkflow_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<openworkflow::Workflow, DecodeError> {
	let c = std::fs::read(path).unwrap();
	let c = Cursor::new(c);
	Message::decode(c)
}

#[derive(Clone)]
pub struct WorkflowVersion {
	// Expose the underlying OpenWorkflow message
	pub workflow: openworkflow::Workflow,
	version: u32,
	// parse_date is useless right now, but I plan to use it for conditional version
	// i.e. version X is valid for a certain time range or version Y is valid until the next parse_date.
	// additionally it will be written to db, so just for completness & debug it's here as well
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

pub struct Workflow {
	workflows : Vec<WorkflowVersion>,
	last_tick: Option<DateTime<Utc>>,
}

impl Workflow {
	pub fn from_openworkflow(workflow: openworkflow::Workflow) -> Workflow {
		Workflow{workflows: vec![WorkflowVersion::new(workflow, 0)], last_tick: None}
	}

	pub fn get_latest_version(&self) -> u32 {
		match self.workflows.iter().map(|w| w.version).max() {
			Some(v) => v,
			None => 0,
		}
	}
	pub fn get_latest_workflow(&self) -> (u32, Option<WorkflowVersion>) {
		let version = self.get_latest_version();
		(version, self.get_workflow_by_version(version))
	}
	pub fn get_workflow_by_version(&self, version: u32) -> Option<WorkflowVersion> {
		self.workflows.iter().filter(|w| w.version == version).last().cloned()
	}

	pub fn add_workflow(&mut self, openworkflow: openworkflow::Workflow) {
		match self.get_latest_workflow() {
			(v, Some(w)) => {
				if w.workflow != openworkflow {
					trace!("New version of workflow '{}' received", w.workflow.workflow_id);
					self.workflows.append(&mut vec![WorkflowVersion::new(openworkflow, v + 1)]);
				} else {
					trace!("Workflow '{}' already present", w.workflow.workflow_id);
				}
			},
			(_, _) => {
				info!("New workflow '{}' received!", openworkflow.workflow_id);
				self.workflows.append(&mut vec![WorkflowVersion::new(openworkflow, 0)]);
			}
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