FROM rust:1-bullseye AS build
WORKDIR /app
COPY ../../Cargo.toml ../../rust-toolchain.toml ./
COPY ../../crates/core/Cargo.toml crates/core/Cargo.toml
COPY ../../crates/black/Cargo.toml crates/black/Cargo.toml
RUN cargo fetch
COPY ../../crates crates
RUN cargo build --release -p proto_zero_black

FROM debian:bullseye-slim
COPY --from=build /app/target/release/proto_zero_black /usr/local/bin/black
USER nobody
ENTRYPOINT ["black"]
