# Proto-Zero — Recovery Prompt / Stage-1

## 🎯 Stage-1: Emergent Drive (Введение первого драйва)

**Дата фиксации:** 2025-10-12  
**ADR:** 009  
**Состояние:** зафиксировано

---

## 🧩 Определение Stage-1

Stage-1 — это первый этап, где у агента появляется **внутренняя динамика** через минимальный драйв (`drive.0`).

| Компонент | Роль | Поведение |
|------------|------|------------|
| **black** | Этичный агент | Бесконечно читает stdin, увеличивает `drive_0` при поступлении данных. Без вывода. |
| **white** | Анатомический агент | Аналогичный код, но публикует Prometheus-метрики `drive`, `delta`, frame-динамику. |
| **world** | Источник данных | Порождает поток байтов (`stdin` или `/dev/urandom`). |
| **observer** | Внешний наблюдатель | Relay потока + timeline/UI + control plane для `world`, не вмешивается в логику агента. |
| **docker-compose** | Оркестрация | Два сценария: `world+black`, `world+observer+white` с FIFO и блокировкой запуска. |

---

## ⚙️ Инварианты Stage-1

1. **Нет wall-clock времени внутри агента.**  
   Внутреннее время после ADR-016 = счётчик сенсорных тиков, а не физические интервалы.

2. **Нет долговременной памяти.**  
   `drive_0` существует только в рамках жизни процесса.

3. **Нет моторов.**  
   Black не имеет внешнего эффекта.

4. **Симметрия black/white обязательна.**

5. **White** остаётся лишь наблюдателем, не влияющим на агентную логику.
6. **Observer** существует только снаружи и не является частью агента.

---

## 🧠 Recovery Prompt

> **Project:** Proto-Zero  
> **Stage:** 1 — *Emergent Drive*  
> **Language:** Rust 2024  
> **Runtime:** Docker Compose
>
> **Principles:**
> - black — этически чистый агент (немой, без моторов, без внешнего наблюдения);
> - white — анатомический двойник (Prometheus-метрики);
> - world — источник данных (шум / bursty / replay);
> - observer — внешний relay и визуальный слой эксперимента;
> - эволюция идёт только через кризисы (ADR).
>
> **Current Invariants:**
> 1. Нет wall-clock времени внутри агента
> 2. Нет памяти
> 3. Нет моторов
> 4. Симметрия black/white
> 5. Метрики только через white
> 6. Observer только внешний
>
> **Components:**
> - `proto_zero_black`: живой процесс с внутренним `drive_0`.
> - `proto_zero_white`: экспортирует `/metrics` с `protozero_drive_0`, `protozero_drive_delta`, `protozero_drive_0_frame`, `protozero_drive_delta_frame`, `protozero_events_total`, `protozero_frames_total`, `protozero_sensor_ticks_total`.
> - `proto_zero_world`: генератор входных байтов с `GET/POST /config`.
> - `protozero_observer`: внешний relay/UI, строит timeline, organism view и проксирует `POST /api/world/config`.
> - docker-compose: два сценария (`world+black`, `world+observer+white`), одновременный запуск запрещён через lock-том.
>
> **Next Goal:**  
> ADR-010 — *Crisis of Drive Saturation* (перенакопление драйва и необходимость регуляции).

---
