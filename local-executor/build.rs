fn main() {
    tonic_build::compile_protos("../OpenWorkflow/proto/openworkflow.proto").unwrap();
}