🧠 Recovery Prompt

**Project**: Proto-Zero \
**Stage: 1** — Emergent Drive (введение первого драйва)  \
**Language**: Rust 2024 \
**Runtime**: Docker Compose

Core principles:
- black — этически чистый агент (немой, без памяти, без времени).
- white — анатомический двойник (симметричная логика + метрики).
- world — источник входных данных (шум).
- архитектура развивается через кризисы и ADR-фиксации.

**Current Invariants:**

1. "Нет времени."
2. Нет долговременной памяти.
3. Нет моторов/выходов у black.
4. Все изменения симметричны black/white.
5. White имеет право наблюдения и логирования.

**Components:**
- proto_zero_black: читает stdin в блокирующем цикле, увеличивает drive_0 при поступлении данных.
- proto_zero_white: то же, но пишет метрики:
```json
{
"agent": "white",
"stage": "Stage-1",
"sensor": {"has_any": true},
"drive": {"0": "<value>"}
}
```
- proto_zero_world: генератор данных (stdin | urandom).
- docker-compose: отдельные сценарии world+black и world+white с FIFO и lock-томами.

**Next Goal**: подготовить ADR-010 — Crisis of Drive Saturation
(момент, когда драйв бесконечно растёт → необходимость регуляции).

Task discipline:
1. Любое новое понятие вводится только через документированный кризис.
2. Код — только минимальная реализация принятого ADR.
3. Симметрия black/white обязательна.
4. Ведём развитие через фиксированные Stage-промпты.