# ADR-016: Event-based Decay

**Статус:** superseded → [ADR-016-non-blocking-sensor](../16-adr/ADR-016-non-blocking-sensor.md)

Исходный драфт предполагал event-based decay. При детальном анализе выяснилось, что корневая проблема — **невозможность наблюдать тишину** из-за blocking read. Решение оформлено как ADR-016: Non-blocking Sensor, который вводит `SensorEvent::Silence` и рождает внутреннее время агента. Event-based decay станет следствием этого решения в будущем ADR.
