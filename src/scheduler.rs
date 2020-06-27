extern crate log;

use log::{info, trace, warn};

use std::{thread, time};
use std::sync::mpsc;

mod workflow;

pub struct Scheduler {
	workflows: Vec<workflow::Workflow>,
}

fn calc_loop_pause(loop_start: time::Instant) -> u64 {
	const LOOP_INTERVAL_SEC : u64 = 30;
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
		Scheduler{workflows: Vec::new()}
	}

	pub fn run(&mut self) {
		let (tx, rx) = mpsc::channel();

		info!("Starting scheduler loop");
		loop {
			let now = time::Instant::now();
			
			self.harvest_workflows();
			self.process_workflows(move tx);

			thread::sleep(time::Duration::from_secs(calc_loop_pause(now)));
			break;
		}
	}

	fn harvest_workflows(&mut self) {
		let w = workflow::Workflow::from_file("msg/serializedFile").unwrap();

		self.workflows.push(w);
	}

	fn process_workflows(&mut self, tx: mpsc::Sender) {
		for workflow in self.workflows.iter_mut() {
			info!("{}", workflow.workflow.workflow_id);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
