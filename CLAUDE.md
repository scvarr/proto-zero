# CLAUDE.md

This repository is a Rust workspace for Proto-Zero.

## Project overview
Proto-Zero — фреймворк развития этичного агента через кризис-ориентированную архитектуру.
Текущая стадия: **Stage-1 (Emergent Drive)**.

Ключевая документация:
- `docs/README.md`
- `docs/roadmap/ADR-ROADMAP.md`

## Build and checks
```bash
cargo build
cargo build --release
cargo test
cargo clippy
cargo fmt --check
```

## Docker stacks
```bash
# Black: немой агент без метрик
docker-compose -f docker-compose.black.yml up --build

# White: наблюдаемый агент + Prometheus + Grafana
docker-compose -f docker-compose.white.yml up --build
```

Endpoints white-стека:
- Grafana: `http://localhost:3000` (`admin/admin`)
- Prometheus: `http://localhost:9090`
- World config API: `http://localhost:8080/config`

## Workspace structure
- `crates/core` (`protozero_core`): `run_stdin_forever`, `drive_update`, `DriveDiff`, `FrameCounter`.
- `crates/black` (`agent_black`): немой агент.
- `crates/white` (`agent_white`): зеркало black + `/metrics`.
- `crates/world` (`proto_zero_world`): генератор среды (`continuous`, `bursty`, `file_replay`) + `GET/POST /config`.

## Invariants
- No time / no wall-clock logic inside agents.
- Black/White symmetry: логика совпадает, white отличается только observability.
- One-way data flow on Stage-1: `world -> agent`.
- Container isolation via lock/FIFO conventions.
- Drive bounded in `[0, 1)` by `d' = 1 / (2 - d)`.

## Conventions
- Rust edition 2024.
- Комментарии в коде на русском языке.
- Тесты рядом с реализацией (`#[cfg(test)]`), фокус на инвариантах.
