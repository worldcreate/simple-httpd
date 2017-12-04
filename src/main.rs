use std::str;
use std::thread;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::io;
use std::io::{Write, BufRead};
use std::net::{TcpListener, TcpStream};

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

fn handle_client(stream: TcpStream) {
    // バッファリングを行うため BufReader を用いる
    let mut stream = io::BufReader::new(stream);

    // stream から最初の一行を読み取る
    let mut first_line = String::new();
    if let Err(err) = stream.read_line(&mut first_line) {
        panic!("error during receive a line: {}", err);
    }

    // unwrap() をなくしたかったのでパターンマッチで直接値を取り出すようにした
    let mut params = first_line.split_whitespace();
    let method = params.next();
    let path = params.next();
    match (method, path) {
        (Some("GET"), Some(file_path)) => {
            // BufReader が所有権を持っていくため，get_mut() で内部の（可変）参照を受け取る
            get_operation(file_path, stream.get_mut());
        }
        _ => panic!("failed to parse"),
    }
}

fn get_operation(file_name: &str, stream: &mut TcpStream) {
    // パスの構築を少しだけスマートにした
    let path = PathBuf::from(format!("./www{}", file_name));
    let mut file = match File::open(&path) {
        Err(why) => {
            panic!(
                "couldn't open {}: {}",
                path.display(),
                Error::description(&why)
            )
        }
        Ok(file) => file,
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);

    // 直接 write() を呼び出さず write!() マクロを用いた
    writeln!(stream, "HTTP/1.1 200 OK").unwrap();
    writeln!(stream, "Content-Type: text/html; charset=UTF-8").unwrap();
    writeln!(stream, "Content-Length: {}", len).unwrap();
    writeln!(stream).unwrap();

    // file -> stream
    // ファイルを読み込むための追加のバッファが必要ない点に注意
    io::copy(&mut file, stream).unwrap();
}
