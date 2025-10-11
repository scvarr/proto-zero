use rand::Rng;
use std::{io::Write, net::{TcpListener, TcpStream}, thread, time::Duration};

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    loop {
        let mut buf = [0u8; 256];
        for b in &mut buf {
            *b = rng.gen();
        }
        stream.write_all(&buf)?;
        thread::sleep(Duration::from_millis(50));
    }
}

fn main() -> std::io::Result<()> {
    let addr = std::env::var("WORLD_ADDR").unwrap_or_else(|_| "0.0.0.0:7000".to_string());
    println!("world-noise: listening on {addr}");
    let listener = TcpListener::bind(addr)?;
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                println!("world-noise: client connected {}", stream.peer_addr().unwrap());
                std::thread::spawn(|| {
                    let _ = handle_client(stream);
                });
            }
            Err(e) => eprintln!("world-noise: accept error: {e}"),
        }
    }
    Ok(())
}
