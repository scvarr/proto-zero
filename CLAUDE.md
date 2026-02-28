# CLAUDE.md

This repository is a Rust workspace for Proto-Zero.

## Project overview
Proto-Zero — фреймворк развития этичного агента через кризис-ориентированную архитектуру.
Текущая стадия: **Stage-1 (Emergent Drive)**.

Ключевая документация:
- `docs/README.md`
- `docs/roadmap/ADR-ROADMAP.md`

## Build and checksВоо
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

## Methodology

Proto-Zero — это не инженерный проект, а **выращивание протожизни** через кризисы.
Каждый ADR фиксирует наблюдение за тем, «о чём просит организм», а не проектное решение.

### Абсолютные законы
- **Атомарные изменения**: каждый ADR вводит строго минимальное расширение.
- **Запрет на параметры без кризиса**: новые переменные/константы появляются только когда без них невозможно решить текущий кризис.
- **Предыдущие ADR актуальны на момент своего кризиса**, но не являются догмой навсегда — инварианты могут эволюционировать, если новый кризис этого требует (пример: «нет времени» может быть снято, когда организм «попросит» время).

### Нумерация ADR
Нумерация — это **пространство имён**, не строгая последовательность:
- `docs/future/` (016-019) — зарезервированные слоты под конкретные направления из roadmap.
- Когда кризис из прошлого получает решение позже — он занимает свободный номер (пример: ADR-020 решает кризис из ADR-012, перескочив через зарезервированные 016-019).
- Параллельные ветки кризисов из одного ADR могут развиваться независимо и сходиться.

### Git-стратегия развития
Проект — **дерево эволюции**:
- При появлении нескольких кризисов-кандидатов — ветка на каждый.
- Выбирается одна, развивается до тупика или результата.
- При тупике — возврат к развилке, выбор другого пути.
- Git позволяет вернуться к любой точке — пространство для маневров не ограничено.

### Долгосрочная цель
Подключение реальных потоков данных (код, видео, звук, файлы) к сенсорам агента.
Агент не получает объяснений о природе данных — он должен **сам** понять через взаимодействие.

## Conventions
- Rust edition 2024.
- Комментарии в коде на русском языке.
- Тесты рядом с реализацией (`#[cfg(test)]`), фокус на инвариантах.
