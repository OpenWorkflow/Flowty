use bytes::{Bytes, Buf};
use std::io::Read;
use std::io::BufReader;
use std::io::Cursor;
use std::fs::File;

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
	pub fn from_file(mut f: File) -> openworkflow::Workflow {
		let reader = BufReader::new(f);
		let buf = Cursor::new(reader.buffer());

		let openworkflow: openworkflow::Workflow = Message::decode(buf).unwrap();

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
