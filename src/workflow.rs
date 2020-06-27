use crate::task::Task;

use chrono::prelude::*;
use cron::Schedule;

pub mod openworkflow {
    include!(concat!(env!("OUT_DIR"), "/openworkflow.workflow.rs"));
}

pub struct Workflow {
	pub workflow: openworkflow::Workflow,
}

impl Workflow {
	
}
