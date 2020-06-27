fn main() {
	prost_build::compile_protos(&["proto/workflow.proto"],
								&["proto/"]).unwrap();
}