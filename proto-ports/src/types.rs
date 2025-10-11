#[derive(Clone, Copy, Debug)]
pub struct StimulusPresence {
    pub ts_micros: u64,
    pub has_any: bool,
}
