FROM rust:1-bullseye AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo fetch
RUN cargo build --release -p protozero_observer

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=build /app/target/release/protozero_observer /usr/local/bin/observer
USER nobody
ENTRYPOINT ["observer"]
