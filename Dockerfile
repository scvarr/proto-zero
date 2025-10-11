# syntax=docker/dockerfile:1

FROM rust:1.80-slim AS build
ARG BUILD_PKG
ARG RUN_BIN
WORKDIR /app

# Cache deps
COPY Cargo.toml ./
COPY proto-ports/Cargo.toml proto-ports/Cargo.toml
COPY kernel-core/Cargo.toml kernel-core/Cargo.toml
COPY kernel-black/Cargo.toml kernel-black/Cargo.toml
COPY kernel-white/Cargo.toml kernel-white/Cargo.toml
COPY world-noise/Cargo.toml world-noise/Cargo.toml
RUN mkdir -p proto-ports/src kernel-core/src kernel-black/src kernel-white/src world-noise/src &&     echo 'fn main(){}' > kernel-black/src/main.rs &&     echo 'fn main(){}' > kernel-white/src/main.rs &&     echo 'fn main(){}' > world-noise/src/main.rs &&     echo 'pub fn f(){}' > proto-ports/src/lib.rs &&     echo 'pub fn f(){}' > kernel-core/src/lib.rs &&     cargo build -p ${BUILD_PKG} --release || true

# Build
COPY . .
RUN cargo build -p ${BUILD_PKG} --release

FROM debian:stable-slim AS runtime
ARG RUN_BIN
WORKDIR /app
COPY --from=build /app/target/release/${RUN_BIN} /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]
