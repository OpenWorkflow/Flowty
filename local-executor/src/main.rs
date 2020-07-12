#[macro_use] extern crate log;
extern crate env_logger;

use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};

use tonic::{transport::Server, Request, Response, Status, Code};
use tokio::sync::mpsc;

use flowty_types::openworkflow::executor_server::{Executor, ExecutorServer};
use flowty_types::openworkflow::{
	Task,
	ExecutionOutput,
	ExecutionStatus
};

#[derive(Default)]
pub struct LocalExecutor {}

#[tonic::async_trait]
impl Executor for LocalExecutor {
	type ExecuteTaskStream = mpsc::Receiver<Result<ExecutionOutput, Status>>;

	async fn execute_task(&self, request: Request<Task>) -> Result<Response<Self::ExecuteTaskStream>, Status> {
		trace!("ExecuteTask = {:?}", request);
		let task = request.into_inner();
		info!("Trying to execute task '{}'", task.task_id);

		match task.execution.unwrap().execution.unwrap().exec {
			Some(openworkflow::execution_definition::Exec::Local(local_execution)) => {
				let (mut tx, rx) = mpsc::channel(1);

				tokio::spawn(async move {
					tx.send(Ok(ExecutionOutput{
						status: ExecutionStatus::Initializing as i32,
						message: String::from("Local-Executor: Initializing task"),
					})).await.unwrap();
					info!("Executing command: {}", local_execution.command.clone());
					let mut cmd = Command::new(local_execution.command)
						.stdout(Stdio::piped())
						.stderr(Stdio::piped())
						.spawn()
						.expect("failed to execute process");
					{
						let stdout = cmd.stdout.as_mut().unwrap();
						let stdout_reader = BufReader::new(stdout);
						let stderr = cmd.stderr.as_mut().unwrap();
						let stderr_reader = BufReader::new(stderr);

						// Todo: Pack this into threads
						for line in stdout_reader.lines() {
							tx.send(Ok(ExecutionOutput{
								status: ExecutionStatus::Running as i32,
								message: line.unwrap(),
							})).await.unwrap();
						}
						for line in stderr_reader.lines() {
							tx.send(Ok(ExecutionOutput{
								status: ExecutionStatus::Running as i32,
								message: line.unwrap(),
							})).await.unwrap();
						}

						match cmd.wait() {
							Ok(s) if s.success() => {
								tx.send(Ok(ExecutionOutput{
									status: ExecutionStatus::Success as i32,
									message: s.code().unwrap().to_string(),
								})).await.unwrap();
							},
							Ok(s) => {
								tx.send(Ok(ExecutionOutput{
									status: ExecutionStatus::Failed as i32,
									message: s.code().unwrap().to_string(),
								})).await.unwrap();
							},
							_ => {
								tx.send(Err(Status::new(Code::Aborted, "Failed to execute task"))).await.unwrap();
							}
						}
					}
				});

				Ok(Response::new(rx))
			},
			_ => {
				error!("Task is not supposed to run with LocalExecutor");
				Err(Status::new(Code::FailedPrecondition, "Task is not supposed to run with LocalExecutor"))
			}
		}
	}
}

const URI: &str = "[::1]:50052";

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