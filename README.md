# Proto-Zero — Stage-1 (Emergent Drive)
Proto-Zero — исследовательский фреймворк для развития этичного агента через кризис-ориентированную архитектуру (ADR-driven).

## Текущий статус реализации
- Текущая стадия: `Stage-1 (Emergent Drive)`.
- `black`: немой агент, без логов/вывода, событийно обновляет drive.
- `white`: анатомический двойник black + экспорт метрик Prometheus.
- `world`: настраиваемый источник событий с HTTP API (`GET/POST /config`).
- `observer`: внешняя надстройка над white-стеком; relay + live UI без вмешательства в логику агента.
- `observer`: внешний control+observe инструмент; relay потока, UI наблюдения и управление `world`.

## Сборка и проверки
- `cargo build`
- `cargo test`

## Docker-стеки
- Black: `docker-compose -f docker-compose.black.yml up --build`
- White (с мониторингом и observer UI): `docker-compose -f docker-compose.white.yml up --build`

Endpoints white-стека:
- Grafana: `http://localhost:3000` (`admin/admin`)
- Prometheus: `http://localhost:9090`
- Observer UI: `http://localhost:7070`
- World API: `http://localhost:8080/config`

Пример конфигурации мира:
```bash
curl -X POST http://localhost:8080/config ^
  -H "Content-Type: application/json" ^
  -d "{\"mode\":\"bursty\",\"bytes_per_sec\":400000,\"chunk_size\":4096,\"frame_bytes\":65536,\"jitter_ms\":20,\"file_path\":\"\",\"seed\":42}"
```

## Основные метрики white
- `protozero_drive_0`
- `protozero_drive_delta`
- `protozero_events_total`
- `protozero_drive_0_frame`
- `protozero_drive_delta_frame`
- `protozero_frames_total`
- `protozero_sensor_ticks_total`

## Observer API
- `GET /api/state` — агрегированное состояние relay + world + metrics.
- `GET /api/events` — SSE-лента наблюдений (`chunk`, `silence_gap`, `config_changed`, `relay_status`).
- `POST /api/world/config` — proxy-обновление `world /config` через observer UI/API.
- `GET /healthz` — health check.

Замечание: `silence_gap` в observer — это внешняя реконструкция по wall-clock отсутствию входных чанков. Это не внутренний сенсорный тик ADR-016.

## Документация
- Индекс docs: `docs/README.md`
- Roadmap: `docs/roadmap/ADR-ROADMAP.md`
