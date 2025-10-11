
use std::{thread, time::Duration};
use kernel-core::Core;
use proto-ports::Probe;

#[derive(Clone)]
struct Logger;
impl Probe for Logger {
    fn gauge(&self, name: &str, value: f64) { println!("gauge {name}={value}"); }
    fn event(&self, name: &str) { println!("event {name}"); }
}

fn main() {
    println!("white: starting");
    let mut k = Core::new(Logger);
    loop {
        k.idle_step();
        println!("white: tick");
        thread::sleep(Duration::from_millis(500));
    }
}
