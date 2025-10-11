
use std::{thread, time::Duration};
use kernel-core::Core;
use proto-ports::NoopProbe;

fn main() {
    let mut k = Core::new(NoopProbe);
    // Stage-0: keep running quietly. No logs, no stdout.
    loop {
        k.idle_step();
        thread::sleep(Duration::from_millis(500));
    }
}
