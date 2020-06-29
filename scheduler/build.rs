fn main() {
	prost_build::compile_protos(&["../OpenWorkflow/proto/workflow.proto"],
								&["../OpenWorkflow/proto/"]).unwrap();
}