#[macro_use] extern crate log;
extern crate env_logger;

use std::process::Command;

use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;

use openworkflow::executor_server::{Executor, ExecutorServer};
use openworkflow::{
	Task,
	ExecutionOutput
};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

#[derive(Default)]
pub struct LocalExecutor {}

impl Executor for LocalExecutor {
	type ExecuteTaskStream = mpsc::Receiver<Result<ExecutionOutput, Status>>;

	async fn execute_task(&self, request: Request<Task>) -> Result<Response<Self::ExecuteTaskStream>, Status> {
		trace!("ExecuteTask = {:?}", request);
		info!("Trying to execute task '{}'", request.into_inner().task_id);

		let (mut tx, rx) = mpsc::channel(4);

		tokio::spawn(async move {
			Command::new("sh")
					.arg("-c")
					.arg("echo hello")
					.stdout(Stdio::piped())
					.stderr(Stdio::piped())
					.spawn()
					.expect("failed to execute process");

			{
				let stdout = cmd.stdout.as_mut().unwrap();
				let stdout_reader = BufReader::new(stdout);
				let stdout_lines = stdout_reader.lines();

				for line in stdout_lines {
					tx.send(Ok(line))
				}

				match status = cmd.wait() {
					Some(s) if s.success() => {

					},
					Some(s) => {
						
					}
					_ => 
				}
			}
		});

		Ok(Response::new(rx))
	}
}

const URI:String = String::from("[::1]:50052");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let addr = URI.parse().unwrap();
	let executor = LocalExecutor::default();

	info!("Executor listening on {}", addr);

	Server::builder()
		.add_service(ExecutorServer::new(executor))
		.serve(addr)
		.await?;

	Ok(())
}