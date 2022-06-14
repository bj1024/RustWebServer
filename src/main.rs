#[macro_use]
extern crate log;
use env_logger::Env;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use std::thread;
use std::time::Duration;
use std::str;

use rust_web_server::ThreadPool;



fn main() {    
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::Builder::from_env(env)
    .format_module_path(false)
    .format_level(true)
    .format_timestamp_millis()
    .format_target(false)
    .format_indent(Some(20))
    .init();

    debug!("start listner.");

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    debug!("cliennt req=[{}]",str::from_utf8(&buffer).unwrap());

    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        debug!("GET /");
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        debug!("GET /sleep");
        thread::sleep(Duration::from_secs(5));
        debug!("sleep done.");
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        debug!("404 NOT FOUND.");
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}