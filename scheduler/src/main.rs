#[macro_use]
extern crate log;
extern crate env_logger;

use tokio_postgres::NoTls;

mod utils;

mod scheduler;
use scheduler::Scheduler;

#[tokio::main]
async fn main() {
	env_logger::init();
	info!("Starting flowty!");

	let (client, connection) = tokio_postgres::connect(
		&utils::get_env("PSQL_URL", "postgres://postgres:postgres@localhost:5432".into()), NoTls)
		.await.unwrap();
	tokio::spawn(async move {
		if let Err(e) = connection.await {
			error!("Connection error: {}", e);
		}
	});

	let mut scheduler: Scheduler = Scheduler::new();
	scheduler.run(&client).await;
}
