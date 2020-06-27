extern crate log;

use std::fs::File;

use std::{thread, time};
use log::{info, trace, warn};

mod workflow;

fn calc_loop_pause(loop_start: time::Instant) -> u64 {
	const LOOP_INTERVAL_SEC : u64 = 30;
	let elapsed_sec : u64 = loop_start.elapsed().as_secs();

	trace!("Calculating loop pause.\nLOOP_INTERVAL_SEC: {}\nElapsed time in seconds: {}", LOOP_INTERVAL_SEC, elapsed_sec);
	if elapsed_sec < LOOP_INTERVAL_SEC {
		return LOOP_INTERVAL_SEC - elapsed_sec;
	}
	warn!("Loop took longer than loop interval. Considering increasing the loop interval {}.", LOOP_INTERVAL_SEC);
	0
}

pub fn scheduler() {
	info!("Starting main scheduler loop");
	loop {
		let now = time::Instant::now();
		
		let w = workflow::Workflow::from_file("msg/serializedFile");
		info!("{}", w.workflow_id);

		thread::sleep(time::Duration::from_secs(calc_loop_pause(now)));
		break;
	}
}
