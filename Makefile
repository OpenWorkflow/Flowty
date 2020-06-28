default: help

.PHONY: build run clean help docker

build:	#: cargo build
	@echo "🚧 Building Flowty"
	cargo build
	@echo "✅ Done building"

run: LOG_LEVEL:=trace
run:	#: cargo run
	@echo "🏃‍ Running Flowty"
	RUST_LOG=$(LOG_LEVEL) cargo run
	@echo "✅"

clean:			#: cargo clean
	@echo "🗑️ Cleaning up"
	cargo clean
	@echo "✅ Cleanup complete"

docker: DOCKER_TAG:=latest
docker:			#: Build the docker image
	@echo "🐋 Building docker image"
	docker build -t flowty:$(DOCKER_TAG) -f docker/Dockerfile .
	@echo "✅ Build complete"

help:			#: Show help topics
	@grep "#:" Makefile* | grep -v "@grep" | sort | sed "s/\([A-Za-z_ -]*\):.*#\(.*\)/$$(tput setaf 3)\1$$(tput sgr0)\2/g"
