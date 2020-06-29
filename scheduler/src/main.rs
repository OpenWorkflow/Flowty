#[macro_use] extern crate log;
extern crate env_logger;
//extern crate config;

//use postgres::{Client, NoTls};

mod scheduler;

use scheduler::Scheduler;

fn main() {
	env_logger::init();
	info!("Starting flowty!");

	/*
	let mut settings = config::Config::default();
	settings
		.merge(config::File::with_name("config")).unwrap()
		.merge(config::Environment::with_prefix("FLOWTY")).unwrap();

	let mut sql_client = Client::connect("host=localhost user=postgres password=flowty_debug", NoTls)?;
	*/
	let mut scheduler: Scheduler = Scheduler::new();
	scheduler.run();
}
