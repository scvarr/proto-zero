//! Stage-2 (ADR-011): параметр-свободная регуляция drive_0.
//! Состояние d ∈ [0,1); событие: d ← 1 / (2 - d)

#[inline]
pub fn drive_update(d: f64) -> f64 {
    // Формула из ADR-011: d' = 1 / (2 - d)
    let next = 1.0 / (2.0 - d);
    // Чисто для численной устойчивости на правой границе:
    next.clamp(0.0, 1.0 - f64::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stays_in_unit_interval() {
        let mut d = 0.0;
        for _ in 0..1_000 {
            d = drive_update(d);
            assert!(d >= 0.0 && d < 1.0);
        }
    }

    #[test]
    fn monotone_and_saturating() {
        let mut d = 0.0;
        let mut prev = d;
        for _ in 0..100 {
            d = drive_update(d);
            assert!(d > prev);
            prev = d;
        }
        assert!(d > 0.5); // после первого события уже > 0.5
    }
}