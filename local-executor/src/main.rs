#[macro_use] extern crate log;
extern crate env_logger;

use tonic::{transport::Server, Request, Response, Status};

use openworkflow::executor_server::{Executor, ExecutorServer};
use openworkflow::{
	Task,
	ExecutionOutput
};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

pub struct LocalExecutor {}

impl Executor for LocalExecutor {
	async fn execute_task(&self, request: Request<Task>) -> Result<Response<Self::ExecuteTaskStream>, Status> {
		info!("Trying to execute task '{}'", task.task_id.as_ref());
		
	}
}

const URI:String = "[::1]:50052".parse().unwrap();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let addr = URI.parse().unwrap();
	let executor = Executor::default();

	info!("Executor listening on {}", addr);

	Server::builder()
		.add_service(ExecutionBrokerServer::new(execution_broker))
		.serve(addr)
		.await?;

	Ok(())
}