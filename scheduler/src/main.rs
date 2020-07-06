#[macro_use]
extern crate log;
extern crate env_logger;

use dotenv::dotenv;

use postgres::{Client, NoTls};

mod scheduler;
use scheduler::Scheduler;

#[tokio::main]
async fn main() {
	dotenv().ok();
	env_logger::init();
	info!("Starting flowty!");
	
	let mut client = Client::connect(dotenv::var("PSQL_URL").unwrap().as_str(), NoTls).unwrap();

	let mut scheduler: Scheduler = Scheduler::new();
	scheduler.run(&mut client);
}
