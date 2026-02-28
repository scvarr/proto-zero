FROM rust:1-bullseye AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo fetch

RUN cargo build --release -p proto_zero_world

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=build /app/target/release/proto_zero_world /usr/local/bin/world
USER nobody
ENTRYPOINT ["world"]
