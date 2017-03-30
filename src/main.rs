use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    fn handle_client(mut stream: TcpStream) {
        println!("got something!");
        let mut buffer = vec![0; 512];
        stream.read(&mut buffer).unwrap();
        let get = b"GET / HTTP/1.1\r\n";

        let start = &buffer[..get.len()];

        if start == get {
            println!("valid header!");
            let response = b"HTTP/1.1 200 OK\r\n\r\n 
            <!DOCTYPE html> 
<html lang=\"en\"> 
  <head> 
    <meta charset=\"utf-8\">
    <title>Hello!</title>
  </head>
  <body>
  <h1>Hello!</h1>
  <p>Hi from Rust</p>
  </body>
</html>";
            stream.write(&*response).unwrap();
            stream.flush().unwrap();
        }
    }

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => { /* connection failed */ }
        }
    }
}