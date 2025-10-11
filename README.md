# Proto-Zero (Stage-0 skeleton)

**Цель:** стартовый каркас для развития по шагам: Мир → Сенсор (позже) → Ядро (black немой, white наблюдаемый).  
Сейчас реализован Stage-0: ядро существует, ничего не принимает и не излучает; мир генерирует шум отдельно.

## Структура
- `proto-ports` — стабильные интерфейсы (Kernel, Probe, StreamSource, Sensor) и минимальные типы.
- `kernel-core` — общая реализация ядра (одна логика, параметризованная Probe).
- `kernel-black` — бинарь с `NoopProbe` (полная тишина).
- `kernel-white` — бинарь с логированием внутренних шагов (stdout).
- `world-noise` — генератор шума в TCP (пока не подключён к ядру).
- `Dockerfile` / `docker-compose.yml` — контейнеризация трёх процессов.

## Быстрый старт (Docker)
```bash
docker compose up --build -d
docker compose logs -f --tail=50 kernel-white
# Увидишь "white: starting" и "white: tick"
# kernel-black молчит; world слушает на 7000/tcp
```

## Локальный запуск
```bash
cargo run -p kernel-white
# в другом терминале
cargo run -p world-noise
```

## Дальше
1) Stage-1: добавить реализацию `Sensor` и протянуть поток от `world` к сенсору (без реакций ядра).
2) Stage-2: внутренняя реакция ядра на StimulusPresence (без внешних эффектов).
3) Stage-3: первый Drive (метаболизм времени / backpressure / latency) — только внутренние изменения.
4) Позже — внешние эффекты (команды/сигналы).
