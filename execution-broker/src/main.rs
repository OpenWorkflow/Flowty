#[macro_use] extern crate log;
extern crate env_logger;

use uuid::Uuid;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use tonic::{transport::Server, Request, Response, Status};

use openworkflow::execution_broker_server::{ExecutionBroker, ExecutionBrokerServer};
use openworkflow::{
	ExecutorKind,
	OperatingSystem,
	LocalSpecification,
	RegistrationRequest,
	RegistrationReply,
	Heartbeat,
	SearchRequest,
	SearchReply
};

pub mod openworkflow {
	tonic::include_proto!("openworkflow.execution_broker");
}

#[derive(Clone)]
pub struct Executor {
	uuid: Uuid,
	pub executor: RegistrationRequest,
	last_heartbeat: Arc<Mutex<Instant>>,
}
impl Executor {
	pub fn new(registration_request: RegistrationRequest) -> Executor {
		let uuid = Uuid::new_v4();
		Executor{
			uuid: uuid,
			executor: registration_request,
			last_heartbeat: Arc::new(Mutex::new(Instant::now()))}
	}

	pub fn get_uuid(&self) -> std::string::String {
		self.uuid.to_hyphenated().to_string()
	}

	pub fn get_time_since_last_heartbeat(&self) -> Duration {
		let heart_beat = self.last_heartbeat.lock().unwrap();
		heart_beat.elapsed()
	}

	pub fn heartbeat(&self) {
		let mut heart_beat = self.last_heartbeat.lock().unwrap();
		*heart_beat = Instant::now();
	}
}

#[derive(Default)]
pub struct FlowtyExecutionBroker {
	executors: Arc<Mutex<Vec<Executor>>>,
}
impl FlowtyExecutionBroker {
	pub fn new() -> FlowtyExecutionBroker {
		FlowtyExecutionBroker{executors: Arc::new(Mutex::new(Vec::new()))}
	}

	pub fn find_by_uri(&self, uri: std::string::String) -> Option<Executor> {
		let executors = Arc::clone(&self.executors);
		let executors = executors.lock().unwrap();
		executors.iter().filter(|e| e.executor.uri == uri).last().cloned()

		// for executor in &executors[..] {
		// 	if executor.executor.uri == uri {
		// 		let executor = *executor;
		// 		return Some(executor.cloned());
		// 	}
		// };
		// None
	}

	pub fn find_by_uuid(&self, uuid: std::string::String) -> Option<Executor> {
		let executors = Arc::clone(&self.executors);
		let executors = executors.lock().unwrap();
		executors.iter().filter(|e| e.get_uuid() == uuid).last().cloned()
	}

	pub fn find_by_request(&self, request: SearchRequest) -> Option<Executor> {
		let executors = Arc::clone(&self.executors);
		let executors = executors.lock().unwrap();

		// todo: honor block_list, local_spec, and return a full list
		executors.iter().filter(|e| e.executor.kind == request.kind).last().cloned()
	}
}

#[tonic::async_trait]
impl ExecutionBroker for FlowtyExecutionBroker {
	async fn register_executor(&self, request: Request<RegistrationRequest>) -> Result<Response<RegistrationReply>, Status> {
		trace!("Got a request from {:?}", request.remote_addr());
		let request = request.into_inner();

		let existing_executor = self.find_by_uri(request.uri.clone());
		match existing_executor {
			Some(e) => {
				e.heartbeat();
				let reply = openworkflow::RegistrationReply{unique_id: e.get_uuid()};
				Ok(Response::new(reply))
			},
			_ => {
				let e = Executor::new(request);
				let uuid = e.get_uuid().clone();

				let executors = Arc::clone(&self.executors);
				tokio::spawn(async move {
					let mut executors = executors.lock().unwrap();
					executors.push(e);
				});

				let reply = openworkflow::RegistrationReply{unique_id: uuid};
				Ok(Response::new(reply))
			}
		}
	}

	async fn heart_beat(&self, request: Request<Heartbeat>) -> Result<Response<Heartbeat>, Status> {
		trace!("Got a heartbeat from {:?}", request);
		let request = request.into_inner();

		let executor = self.find_by_uuid(request.unique_id.clone());
		if match executor {
					Some(e) => {
						e.heartbeat();
						true
					},
					_ => false
			} {
			return Ok(Response::new(request));
		}

		let executor = self.find_by_uri(request.uri.clone());
		match executor {
			Some(e) => {
				e.heartbeat();
				Ok(Response::new(request))
			},
			_ => Ok(Response::new(Heartbeat{uri: "".to_string(), unique_id: "".to_string()}))
		}
	}

	async fn find_executor(&self, request: Request<SearchRequest>) -> Result<Response<SearchReply>, Status> {
		trace!("Got a search request from {:?}", request.remote_addr());
		let request = request.into_inner();

		let result = self.find_by_request(request);
		match result {
			Some(e) => Ok(Response::new(SearchReply{uri: e.executor.uri.clone()})),
			_ => Ok(Response::new(SearchReply{uri: "".to_string()}))
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
