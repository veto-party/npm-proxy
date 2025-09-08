
FROM node:24 as front_builder

WORKDIR /usr/src/frontend

COPY ./frontend/package.json package.json
COPY ./frontend/package-lock.json package-lock.json
RUN npm i

COPY ./frontend .

RUN npm run build

FROM rust:1.89 AS back_builder

WORKDIR /usr/src/npm-proxy

COPY ./backend/ .
RUN cargo install --path .

FROM debian:bullseye-slim

WORKDIR /opt/npm-proxy/

# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=back_builder /usr/src/npm-proxy /usr/local/bin/npm-proxy
COPY --from=front_builder /usr/src/frontend/build ./public