fn main() {
    tonic_build::compile_protos("../OpenWorkflow/proto/execution_broker.proto").unwrap();
}