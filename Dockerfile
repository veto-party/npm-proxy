
FROM rust:1.89 AS builder

RUN cd / && \
    cargo new playground
WORKDIR /playground

ADD Cargo.toml /playground/Cargo.toml

RUN cargo build --release

WORKDIR /usr/src/PROJ

WORKDIR /usr/src/npm-proxy
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/npm-proxy /usr/local/bin/npm-proxy