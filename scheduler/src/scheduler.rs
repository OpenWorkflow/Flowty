extern crate log;

use log::{info, trace, warn};

use std::string::String;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

use tonic::Request;

use workflow::openworkflow::executor_client::ExecutorClient;
// use workflow::openworkflow::{
// 	Task,
// 	ExecutionOutput,
// 	ExecutionStatus
// };

mod workflow;

const LOOP_INTERVAL_SEC : u64 = 2;

pub struct Scheduler {
	workflow_bundle: HashMap<String, workflow::Workflow>,
	//sql_client: Client,
}

fn calc_loop_pause(loop_start: Instant) -> u64 {
	let elapsed_sec : u64 = loop_start.elapsed().as_secs();

	trace!(
		"Calculating loop pause.\nLOOP_INTERVAL_SEC: {}\nElapsed time in seconds: {}",
		LOOP_INTERVAL_SEC,
		elapsed_sec
	);
	if elapsed_sec < LOOP_INTERVAL_SEC {
		return LOOP_INTERVAL_SEC - elapsed_sec;
	}
	warn!(
		"Loop took longer than loop interval. Considering increasing the loop interval {}.",
		LOOP_INTERVAL_SEC
	);
	0
}

impl Scheduler {
	pub fn new() -> Scheduler {
		Scheduler{workflow_bundle: HashMap::new()}
	}

	pub fn run(&mut self) {
		info!("Starting scheduler loop");
		loop {
			let now = Instant::now();
			
			self.harvest_workflows();
			self.process_workflows();

			thread::sleep(Duration::from_secs(calc_loop_pause(now)));
			break;
		}
	}

	fn harvest_workflows(&mut self) {
		// Replace this with reading from DB and looping over results:
		let openworkflow = workflow::openworkflow_from_file("msg/workflow").unwrap();
		let workflow_id = openworkflow.workflow_id.clone();

		match self.workflow_bundle.get_mut(&workflow_id) {
			Some(w) => w.add_workflow(openworkflow),
			_ => {
				self.workflow_bundle.insert(workflow_id, workflow::Workflow::from_openworkflow(openworkflow));
			},
		}
	}
// let mut stream = client
//         .list_features(Request::new(rectangle))
//         .await?
//         .into_inner();

//     while let Some(feature) = stream.message().await? {
//         println!("NOTE = {:?}", feature);
//     }
	fn process_workflows(&mut self) {
		for (workflow_id, workflow) in self.workflow_bundle.iter_mut() {
			info!("{}", workflow_id);
			let (_, workflow) = workflow.get_latest_workflow();
			if let Some(w) = workflow {
				for task in w.workflow.tasks.iter() {
					info!("Executing task {}", task.task_id);
					let task = task.clone();
					tokio::spawn(async move {
						let mut client = ExecutorClient::connect("http://[::1]:50052".to_string()).await.unwrap();
						let mut stream = client.execute_task(Request::new(task)).await.unwrap().into_inner();
						while let Some(output) = stream.message().await.unwrap() {
							info!("Response = {:?}", output);
						}
					});
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
