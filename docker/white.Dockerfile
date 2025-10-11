FROM rust:1-bullseye AS build
WORKDIR /app
COPY ../../Cargo.toml ../../rust-toolchain.toml ./
COPY ../../crates crates

RUN cargo fetch

RUN cargo build --release -p agent_white

FROM debian:bullseye-slim
COPY --from=build /app/target/release/agent_white /usr/local/bin/white
USER nobody
ENTRYPOINT ["white"]
