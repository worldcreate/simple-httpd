use std::str;
use std::thread;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
            Err(_) => { panic!("connection failed") }
        };
        
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buf;

    loop {
        buf = [0; 1024];

        let _ = match stream.read(&mut buf) {
            Err(e) => panic!("Got an error: {}", e),
            Ok(m) => {
                if m == 0 {
                    // EOF
                    break;
                }
                m
            },
        };

        let line = str::from_utf8(&buf).unwrap();
        let first_line = line.lines().nth(0);
        println!("{}", first_line.unwrap());
        let mut params = first_line.unwrap().split_whitespace();

        match params.next().unwrap() {
            "GET" => {
                get_operation(params.next().unwrap(), &stream);
            },
            _ => {}
        }

    }
}

fn get_operation(file_name: &str, mut stream: &TcpStream) {
    let file_path = format!("./www{}", file_name).to_string();
    let path = Path::new(&file_path);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };
    stream.write("HTTP/1.1 200 OK\n".as_bytes()).unwrap();
    stream.write("Content-Type: text/html; charset=UTF-8\n".as_bytes()).unwrap();
    stream.write(format!("Content-Length: {}\n", file.metadata().unwrap().len()).as_bytes()).unwrap();
    stream.write("\n".as_bytes()).unwrap();

    let mut file_buf;
    loop {
        file_buf = [0; 1024];

        let _ = match file.read(&mut file_buf) {
            Err(e) => panic!("{}", e),
            Ok(m) => {
                if m == 0 {
                    break;
                }
                m
            },
        };

        stream.write(&file_buf).unwrap();
    }

    println!("{:?}", path);
}
