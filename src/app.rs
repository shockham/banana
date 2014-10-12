use std::collections::HashMap;

use std::io::{TcpListener, TcpStream, BufferedStream};
use std::io::{Acceptor, Listener};

pub struct App<'a> {
    pub routes: HashMap<&'a str,fn() -> &'static str>,
}

impl<'a> App<'a>  {
    pub fn new() -> App<'a> {
        App{
            routes : HashMap::new()
        }
    }
    
    fn handle_client(&self, stream: &mut BufferedStream<TcpStream>) -> () {
            let request:String = match stream.read_line() {
                Err(e) => { format!("error: {}", e) }
                Ok(stream_output) => { stream_output }
            };
            println!("{}", request);
            
            let request_parts: Vec<&str> = request
                .as_slice()
                .split(' ')
                .collect();
            let full_path: &str = request_parts[1];
            
            let split_path: Vec<&str> = full_path
                .split('?')
                .collect();
            let route = split_path[0];
            let query_string = match split_path.len() {
               1  => "",
                _ => split_path[1]
            };
            

            let mut content = "404";
            for (r, callback) in self.routes.iter() {
                if *r == route {
                    println!("found");
                    let call_func = *callback;
                    content = call_func();
                }
            }

            let with_headers = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\ncontent-length: {}\r\n\r\n{}",content.len(),content);

            let outcome = stream.write_str(with_headers.as_slice());
            println!("-> {}", outcome);
    }

    pub fn run(&self, address:&str, port:u16) -> () {

        let mut acceptor = TcpListener::bind(address, port).listen();

        println!("||Starting server||{}:{}||", address, port);

        for stream in acceptor.incoming() {
            match stream {
                Err(e) => { println!("error: {}", e) }
                Ok(stream) => {
                    //spawn(proc() {
                        let mut buf_stream = BufferedStream::new(stream);
                        self.handle_client(&mut buf_stream);
                    //})
                }
            }
        }
    }
}
