use bytes::Buf;
use std::io::Cursor;

//use chrono::prelude::*;	
//use cron::Schedule;
use prost::{Message, DecodeError};

pub mod openworkflow {
    include!(concat!(env!("OUT_DIR"), "/openworkflow.workflow.rs"));
}

pub struct Workflow {
	// Expose the underlying OpenWorkflow message
	pub workflow: openworkflow::Workflow,
}

impl Workflow {
	pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> openworkflow::Workflow {
		let contents = std::fs::read(path).unwrap();
		let cursor = Cursor::new(contents);

		let openworkflow: openworkflow::Workflow = Message::decode(cursor).unwrap();

		openworkflow
	}

	pub fn new<B>(buf: B) -> Result<openworkflow::Workflow, DecodeError> 
	where
		B: Buf,
	{
		let openworkflow: openworkflow::Workflow = Message::decode(buf).unwrap();

		Ok(openworkflow)
	}
}
