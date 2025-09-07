
FROM rust:1.89 AS builder

WORKDIR /usr/src/npm-proxy

COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/npm-proxy /usr/local/bin/npm-proxy