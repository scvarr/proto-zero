FROM rust:1-bullseye AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo fetch

RUN cargo build --release -p agent_black

FROM debian:bullseye-slim
COPY --from=build /app/target/release/agent_black /usr/local/bin/black
USER nobody
ENTRYPOINT ["black"]
