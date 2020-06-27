#[macro_use] extern crate log;
extern crate env_logger;

mod scheduler;

fn main() {
	env_logger::init();
    info!("Starting flowty!");

    scheduler::scheduler();
}
