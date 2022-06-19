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
use lazy_static::lazy_static;
use std::sync::RwLock;



lazy_static! {
    static ref  CLIENT_NUM:RwLock<u32> = RwLock::new(0);
}


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

    let addr ="127.0.0.1:7878";

    debug!("start TcpListener. {}",addr);

    let listener = TcpListener::bind(addr).unwrap();
    let pool = ThreadPool::new(4);
    // debug!("Listen");

    // Thread終了・解放を試すために、最初の２個だけ処理する。
    // take はIterator trait に定義される。最初のN個だけ処理する。
    // for stream in listener.incoming().take(2){
    for stream in listener.incoming(){
        let mut clinet_num_ptr = CLIENT_NUM.write().unwrap();
        let num  = *clinet_num_ptr; 

        // debug!("CLIENT_NUM=[{}]",num);
        // let mut client_num_write = CLIENT_NUM.write().unwrap();
        *clinet_num_ptr = *clinet_num_ptr + 1;
        // *CLIENT_NUM +=1 ;

            match stream {
            Ok(stream) => {
                debug!("incoming stream [{}] {:?}",num,stream.peer_addr().unwrap());
                pool.execute(move || {
                    handle_connection(num,stream);
                });
            }
            Err(e) => {error!("connection failed {:?}",e);}
        }
    }
}
fn handle_connection(no:u32 ,mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/";

   
    let sleep = b"GET /sleep HTTP/";
    let threadsig = format!("[{}][{}]",no,thread::current().name().unwrap_or("unknown thread"));
    let (status_line, filename) = if buffer.starts_with(get) {
        info!("{} GET /",threadsig);
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        info!("{} GET /sleep",threadsig);
        thread::sleep(Duration::from_secs(3));
        // debug!("sleep done.");
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        warn!("{} 404 NOT FOUND.req=[{}]",threadsig,str::from_utf8(&buffer).unwrap());
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