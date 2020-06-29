#[macro_use] extern crate log;
extern crate env_logger;

use uuid::Uuid;
use std::time::{Duration, Instant};

use tonic::{transport::Server, Request, Response, Status};

use openworkflow::execution_broker_server::{ExecutionBroker, ExecutionBrokerServer};
use openworkflow::{
	ExecutorKind,
	OperatingSystem,
	LocalSpecification,
	RegistrationRequest,
	RegistrationReply,
	Heartbeat,
	SearchRequest
};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

struct Executor {
	uuid: Uuid,
	pub executor: RegistrationRequest,
	last_heartbeat: Instant,
}
impl Executor {
	pub fn new(registration_request: RegistrationRequest) -> Executor {
		let uuid = Uuid::new_v4();
		Executor{uuid: uuid, executor: registration_request, last_heartbeat: Instant::now()}
	}

	pub fn get_uuid(&self) -> std::string::String {
		self.uuid.to_hyphenated().to_string()
	}

	pub fn get_time_since_last_heartbeat(&self) -> Duration {
		self.last_heartbeat.elapsed()
	}

	pub fn heartbeat(&self) {
		self.last_heartbeat = Instant::now();
	}
}

#[derive(Default)]
pub struct FlowtyExecutionBroker {
	executors: Vec<Executor>,
}
impl FlowtyExecutionBroker {
	pub fn new() -> FlowtyExecutionBroker {
		FlowtyExecutionBroker{executors: Vec::new()}
	}

	pub fn find_by_uri(&self, uri: std::string::String) -> Option<Executor> {
		self.executors.iter().filter(|e| e.executor.uri == uri).collect().last()
	}

	/*
	pub fn already_registered(&self, uri: str) -> bool {
		match self.find_by_uri(uri) {
			Some(e) => true,
			_ => false,
		}
	}
	*/
}

#[tonic::async_trait]
impl ExecutionBroker for FlowtyExecutionBroker {
	async fn register_executor(&mut self, request: Request<RegistrationRequest>) -> Result<Response<RegistrationReply>, Status> {
		trace!("Got a request from {:?}", request.remote_addr());
		let request = request.into_inner();
		let existing_executor = self.find_by_uri(request.uri);
		match existing_executor {
			Some(e) => {
				e.heartbeat();
				let reply = openworkflow::RegistrationReply{unique_id: e.get_uuid()};
				Ok(Response::new(reply))
			},
			_ => {
				let e = Executor::new(request);
				self.executors.push(e);
				let reply = openworkflow::RegistrationReply{unique_id: e.get_uuid()};
				Ok(Response::new(reply))
			}
		}

		
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let execution_broker = FlowtyExecutionBroker::default();

    println!("ExecutionBrokerServer listening on {}", addr);

    Server::builder()
        .add_service(ExecutionBrokerServer::new(execution_broker))
        .serve(addr)
        .await?;

    Ok(())
}
