extern crate log;

use log::{info, trace, warn};

use std::string::String;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

use tonic::Request;

use chrono::prelude::*;

use postgres::Client;

use workflow::openworkflow::executor_client::ExecutorClient;

mod workflow;

pub struct Scheduler {
	workflow_bundle: HashMap<String, workflow::Workflow>,
}

fn calc_loop_pause(loop_start: Instant) -> u64 {
	let loop_interval_sec: u64 = dotenv::var("LOOP_INTERVAL_SEC")
		.unwrap_or_default()
		.parse()
		.unwrap_or_else(|_| 30);
	let elapsed_sec : u64 = loop_start.elapsed().as_secs();

	trace!(
		"Calculating loop pause.\nLOOP_INTERVAL_SEC: {}\nElapsed time in seconds: {}",
		loop_interval_sec,
		elapsed_sec
	);
	if elapsed_sec < loop_interval_sec {
		return loop_interval_sec - elapsed_sec;
	}
	warn!(
		"Loop took longer than loop interval. Considering increasing the loop interval {}.",
		loop_interval_sec
	);
	0
}

impl Scheduler {
	pub fn new() -> Scheduler {
		Scheduler{workflow_bundle: HashMap::new()}
	}

	pub fn run(&mut self, client: &mut Client) {
		info!("Starting scheduler loop");
		loop {
			let now = Instant::now();
			
			self.harvest_workflows(client);
			self.process_workflows();

			thread::sleep(Duration::from_secs(calc_loop_pause(now)));
			break;
		}
	}

	fn harvest_workflows(&mut self, client: &mut Client) {
		let result = client
			.query("WITH latest AS (
				SELECT MAX(wid) AS wid, workflow_id FROM workflow GROUP BY workflow_id
			)
			SELECT workflow.workflow_id, workflow.openworkflow_message
			FROM workflow JOIN latest ON workflow.wid = latest.wid;", &[]
			);

		match result {
			Ok(rows) => {
				for row in rows {
					let workflow_id: &str = row.get(0);
					let openworkflow: Option<&[u8]> = row.get(1);

					info!("Parsing workflow with workflow_id '{}' from db", workflow_id);
					if let Some(w) = openworkflow {
						let w = workflow::openworkflow_from_binary(w);
						match w {
							Ok(w) => {
								if self.workflow_bundle.contains_key(workflow_id) {
									trace!("Old version of workflow already known. Replacing");
									let workflow = self.workflow_bundle.get_mut(workflow_id).unwrap();
									workflow.update_workflow(w, false);
								} else {
									trace!("Brand new workflow received");
									self.workflow_bundle.insert(workflow_id.to_string(), workflow::Workflow::new(w));
								}
							},
							Err(e) => {
								error!("Failed to parse workflow '{}':\n{}", workflow_id, e);
							}
						}
					} else {
						trace!("No binary data received for workflow_id '{}' from db", workflow_id);
					}
				}
			},
			Err(e) => {
				error!("Failed to retrieve workflows from postgres:\n{}", e);
				panic!("Failed to retrieve workflows from postgres:\n{}", e);
			}
		}
	}

	fn process_workflows(&mut self) {
		let now = Utc::now();
		for (workflow_id, workflow) in self.workflow_bundle.iter_mut() {
			info!("Processing workflow: '{}'", workflow_id);
			workflow.tick(now);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
