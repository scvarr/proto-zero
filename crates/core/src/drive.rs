//! Stage-2..3: drive и его дифференциал (ADR-011, ADR-014)

#[inline]
pub fn drive_update(d: f64) -> f64 {
    // Формула из ADR-011: d' = 1 / (2 - d)
    let next = 1.0 / (2.0 - d);
    // Чисто для численной устойчивости на правой границе:
    next.clamp(0.0, 1.0 - f64::EPSILON)
}

#[derive(Debug, Clone, Copy)]
pub struct DriveDiff {
    prev: f64,
}

impl DriveDiff {
    #[inline]
    pub fn new() -> Self {
        Self { prev: 0.0 }
    }

    #[inline]
    pub fn step(&mut self, current: f64) -> f64 {
        let delta = current - self.prev;
        self.prev = current;
        delta
    }
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