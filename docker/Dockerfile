FROM rust:1.44 as builder

LABEL MAINTAINER="Milchdealer"
LABEL repository="https://github.com/OpenWorkflow/Flowty"

ENV APP_DIR="/usr/src/flowty"

WORKDIR $APP_DIR
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/flowty /usr/local/bin/flowty
CMD ["flowty"]
