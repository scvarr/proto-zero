FROM rust:1-bullseye AS build
WORKDIR /app
COPY ../../Cargo.toml ../../rust-toolchain.toml ./
COPY ../../crates/core/Cargo.toml crates/core/Cargo.toml
COPY ../../crates/white/Cargo.toml crates/white/Cargo.toml
RUN cargo fetch
COPY ../../crates crates
RUN cargo build --release -p proto_zero_white

FROM debian:bullseye-slim
COPY --from=build /app/target/release/proto_zero_white /usr/local/bin/white
USER nobody
ENTRYPOINT ["white"]
