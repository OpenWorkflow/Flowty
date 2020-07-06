default: help

.PHONY: docker psql

psql: PSQL_DATA_FOLDER:=data
psql: PSQL_PASSWORD:=flowty
psql:
	@echo "🐘 Starting Postgres in 🐋docker"
	mkdir -p $(PSQL_DATA_FOLDER)
	docker run -d -e POSTGRES_PASSWORD=$(PSQL_PASSWORD) -e PGDATA=/var/lib/postgresql/data -p 5432:5432 -v `pwd`/$(PSQL_DATA_FOLDER):/var/lib/postgresql/data postgres
	@echo "✅ Started"

docker: DOCKER_TAG:=latest
docker:			#: Build the docker image
	@echo "🐋 Building docker image"
	docker build -t flowty:$(DOCKER_TAG) -f docker/Dockerfile .
	@echo "✅ Build complete"

help:			#: Show help topics
	@grep "#:" Makefile* | grep -v "@grep" | sort | sed "s/\([A-Za-z_ -]*\):.*#\(.*\)/$$(tput setaf 3)\1$$(tput sgr0)\2/g"
