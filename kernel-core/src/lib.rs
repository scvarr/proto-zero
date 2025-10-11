use proto-ports::{Probe, Kernel as KernelTrait, StimulusPresence};

/// Single core implementation parametrized by a Probe.
/// Stage-0: it is effectively "mute" and does not produce external effects.
pub struct KernelCore<P: Probe> {
    probe: P,
    ticks: u64,
    last_ts: u64,
}

impl<P: Probe> KernelCore<P> {
    pub fn new(probe: P) -> Self {
        Self { probe, ticks: 0, last_ts: 0 }
    }

    /// Internal heartbeat; in Stage-0 we keep the process alive without emitting anything.
    pub fn idle_step(&mut self) {
        self.ticks = self.ticks.wrapping_add(1);
        // White will implement Probe and can log this internally; Black's NoopProbe does nothing.
        self.probe.gauge("ticks", self.ticks as f64);
        // Sleep duration is controlled outside (binary main), to keep the core pure.
    }
}

impl<P: Probe> KernelTrait for KernelCore<P> {
    fn ingest(&mut self, s: StimulusPresence) {
        // Stage-0: accept a stimulus but only touch internal state; no external signals.
        self.last_ts = s.ts_micros;
        // Optional internal logging for white:
        self.probe.gauge("last_ts", self.last_ts as f64);
        if s.has_any {
            self.probe.event("presence");
        }
    }
}

pub type Core<P> = KernelCore<P>;
