use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use stockbit_auth::auth::{login, register};
use stockbit_auth::constants::NOT_FOUND;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7879").unwrap();
    println!("Server running on http://127.0.0.1:7879");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(err) => println!("unable to connect: {}", err),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /login") => login(r),
                r if r.starts_with("POST /register") => register(r),
                _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
            };

            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .unwrap();
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}
