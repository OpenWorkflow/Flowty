default: help

.PHONY: build run clean help

build:	#: cargo build
	@echo "Building Flowty"
	cargo build

run: LOG_LEVEL:=trace
run:	#: cargo run
	@echo "Running Flowty"
	RUST_LOG=$(LOG_LEVEL) cargo run

clean:			#: cargo clean
	@echo "Cleaning up"
	cargo clean

help:			#: Show help topics
	@grep "#:" Makefile* | grep -v "@grep" | sort | sed "s/\([A-Za-z_ -]*\):.*#\(.*\)/$$(tput setaf 3)\1$$(tput sgr0)\2/g"
