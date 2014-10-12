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
    
    fn handle_client<'a>(&self, stream: &'a mut BufferedStream<TcpStream>) -> () {
            let request:String = match stream.read_line() {
                Err(e) => { format!("error: {}", e) }
                Ok(stream_output) => { stream_output }
            };
            println!("{}", request);
            
            let clone_slice = request.as_slice();
            let request_parts: Vec<&str> = clone_slice.split(' ').collect();
            let full_path: &str = request_parts[1];
            
            let split_path: Vec<&str> = full_path.split('?').collect();
            let route = split_path[0];
            let query_string = match split_path.len() {
               1  => "",
                _ => split_path[1]
            };

            //let route_string = String::from_str("/");
            //let route_slice = route_string.as_slice();

            //let callback: fn() -> &'static str = *self.routes.find(&route).unwrap();
            let callback: fn() -> &'static str = *self.routes.find(&("/")).unwrap();
            let content = callback();
            //let content = format!("<h1>TEST</h1><p>route:{}</p><p>query string:{}</p>",route,query_string);

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
