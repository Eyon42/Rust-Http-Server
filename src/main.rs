use std::fmt::Display;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::{thread, time};

enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}
impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
        };
        write!(f, "{}", method)
    }
}

fn main() -> std::io::Result<()> {
    let address = "127.0.0.1:5001";

    let listener = TcpListener::bind(address).unwrap();
    println!("Initialized http server on port {}", address);

    for s in listener.incoming() {
        let mut stream = match s {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };
        let mut buffer_reader = BufReader::new(&mut stream);
        let mut first_line = String::new();
        buffer_reader.read_line(&mut first_line).unwrap();
        let mut info = first_line.split_whitespace();
        let method = match info.next() {
            Some("GET") => HttpMethod::GET,
            Some("POST") => HttpMethod::POST,
            Some("PUT") => HttpMethod::PUT,
            Some("DELETE") => HttpMethod::DELETE,
            Some(m) => panic!("Method {} not supported", m),
            _ => panic!("shit"),
        };
        let resource = match info.next() {
            Some(r) => r,
            _ => "/",
        };
        println!("{} {}", method, resource);

        match resource {
            "/demo" => stream_html(&mut stream, resource, method),
            _ => serve_file(&mut stream, resource, method),
        }
    }
    Ok(())
}

fn stream_html(stream: &mut TcpStream, resource: &str, method: HttpMethod) {
    stream
        .write_all(
            b"HTTP/1.1 405 OK\r\n\r
        <!doctype html>
        <html lang=\"en\">
            <head>
                <title>Rust HTTP Server</title>
            </head>
            <body>
                <ul>
        ",
        )
        .unwrap();
    for i in 0..10 {
        thread::sleep(time::Duration::from_secs(1));
        let value = format!("<li>{}</li>", i);
        stream.write_all(value.as_bytes()).unwrap()
    }
    stream
        .write_all(
            b"</ul>
        </body>
    </html>
    ",
        )
        .unwrap();
}

fn serve_file(stream: &mut TcpStream, resource: &str, method: HttpMethod) {
    match method {
        HttpMethod::GET => (),
        _ => {
            stream
                .write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")
                .unwrap();
            return;
        }
    }
    let mut path = PathBuf::new();
    path.push("public");
    path.push(
        resource
            .trim_start_matches("/")
            .replace("/../", "/")
            .replace("../", "")
            .replace("/..", ""),
    );
    if resource.ends_with("/") {
        println!("ends with /");
        path.push("index.html");
    }

    println!("Serving: {}", path.to_str().unwrap());
    match std::fs::read(path) {
        Ok(f) => {
            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            stream.write_all(f.as_slice()).unwrap();
        }
        Err(_) => {
            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
            stream.write_all(b"404 - Not Found").unwrap();
        }
    }
}
