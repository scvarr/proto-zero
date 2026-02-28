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
- Observer UI: `http://localhost:7070`

## Workspace structure
- `crates/core` (`protozero_core`): `run_stdin_forever`, `run_sensor_loop`, `drive_update`, `DriveDiff`, `FrameCounter`, `SensorEvent`.
- `crates/black` (`agent_black`): немой агент.
- `crates/white` (`agent_white`): зеркало black + `/metrics` (включая `protozero_sensor_ticks_total`).
- `crates/world` (`proto_zero_world`): генератор среды (`continuous`, `bursty`, `file_replay`) + `GET/POST /config`.
- `crates/observer` (`protozero_observer`): relay + aggregator + live UI на `:7070` (ADR-022/024).

## Invariants

### Agent invariants
- Время агента = счётчик тиков сенсора (попыток чтения stdin), не физические секунды (ADR-016).
- Сенсор различает три состояния: **Data** (стимул), **Silence** (тишина), **EOF** (смерть) (ADR-016).
- Black/White symmetry: логика совпадает, white отличается только observability.
- One-way data flow on Stage-1: `world -> agent`.
- Container isolation via lock/FIFO conventions.
- Drive bounded in `[0, 1)` by `d' = 1 / (2 - d)`.

### Observer invariants
- Observer — внешний слой наблюдения; не меняет агентную логику и не подаёт агенту обратную связь (ADR-022).
- Внешний `silence_gap` observer — операторская реконструкция, а не агентный факт (ADR-022).
- Управление `world` из observer — control plane среды, не моторное действие агента (ADR-024).

## Methodology

Proto-Zero — это не инженерный проект, а **выращивание протожизни** через кризисы.
Каждый ADR фиксирует наблюдение за тем, «о чём просит организм», а не проектное решение.

### Абсолютные законы
- **Атомарные изменения**: каждый ADR вводит строго минимальное расширение.
- **Запрет на параметры без кризиса**: новые переменные/константы появляются только когда без них невозможно решить текущий кризис.
- **Предыдущие ADR актуальны на момент своего кризиса**, но не являются догмой навсегда — инварианты могут эволюционировать, если новый кризис этого требует (пример: «нет времени» может быть снято, когда организм «попросит» время).

### Нумерация ADR
Нумерация — это **пространство имён**, не строгая последовательность:
- `docs/future/` (017-019, 023+) — зарезервированные слоты под конкретные направления из roadmap.
- ADR-016 реализован (non-blocking sensor); ADR-017 — следующий кандидат (event-based decay).
- Два трека развития: **Agent Track** (014→016→017→...) и **Observer Track** (008→021→022→024→...).
- Когда кризис из прошлого получает решение позже — он занимает свободный номер.
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
