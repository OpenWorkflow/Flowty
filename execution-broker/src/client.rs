#[macro_use] extern crate log;
extern crate env_logger;

use tonic::Request;

use openworkflow::execution_broker_client::ExecutionBrokerClient;
use openworkflow::{
	RegistrationRequest,
	ExecutorDefinition,
	ExecutorSpecification,
	executor_specification,
	LocalSpecification,
	Heartbeat,
	SearchRequest
};

pub mod openworkflow {
	tonic::include_proto!("openworkflow");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let mut client = ExecutionBrokerClient::connect("http://[::1]:50051".to_string()).await?;
	let response = client
		.register_executor(Request::new(RegistrationRequest {
			uri: "http://[::1]:10000".to_string(),
			executor_definition: Some(ExecutorDefinition{
				kind: 0,
				specs: Some(ExecutorSpecification{
					specs: Some(executor_specification::Specs::Local(LocalSpecification{packages: vec![]}))
				})
			})
		}))
		.await?;
	info!("RESPONSE = {:?}", response);

	let uuid = response.into_inner().unique_id;
	let response = client
		.heart_beat(Request::new(Heartbeat{
			unique_id: uuid
		}))
		.await?;
	info!("HEARTBEAT = {:?}", response);

	let response = client
		.find_executor(Request::new(SearchRequest{
			executor_definition: Some(ExecutorDefinition{
				kind: 0,
				specs: Some(ExecutorSpecification{
					specs: Some(executor_specification::Specs::Local(LocalSpecification{packages: vec![]}))
				})
			}),
			block_list: vec![],
		}))
		.await?;
	info!("SearchRequest Reply = {:?}", response);

	Ok(())
}
