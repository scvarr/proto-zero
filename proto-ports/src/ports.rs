/// Minimal diagnostic hook. In black we use NoopProbe; in white we provide a logger/metrics.
pub trait Probe: Send + Sync {
    fn gauge(&self, _name: &str, _value: f64) {}
    fn event(&self, _name: &str) {}
}
pub struct NoopProbe;
impl Probe for NoopProbe {}

/// The kernel (black/white share the same behavior; white only instruments internally).
pub trait Kernel: Send {
    fn ingest(&mut self, _s: crate::types::StimulusPresence) {}
}

/// World source (pull or push via adapters). Stage-1+ will connect this to sensors.
pub trait StreamSource: Send {
    /// Fill provided buffer with bytes; return number of bytes written (0 = no data now).
    fn fill(&mut self, buf: &mut [u8]) -> usize;
}

/// Sensor converts raw stream into minimal presence stimulus.
pub trait Sensor: Send {
    fn read(&mut self) -> crate::types::StimulusPresence;
}
