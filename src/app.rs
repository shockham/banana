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
    
    fn handle_client(&self, mut stream: BufferedStream<TcpStream>) -> () {
            let request = stream.read_line().ok().unwrap();
            println!("{}", request);
            let request_parts: Vec<&str> = request.as_slice().split(' ').collect();
            let full_path: &str = request_parts[1];

            let split_path: Vec<&str> = full_path.split('?').collect();

            let route = split_path[0];
            let query_string;
            if split_path.len() > 1{
                query_string = split_path[1];
            }else{
                query_string = "";
            }
        
            let callback = self.routes.find(&route).unwrap();
            //let content = callback();
            let content = format!("<h1>TEST</h1><p>route:{}</p><p>query string:{}</p>",route,query_string);

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
                Ok(stream) => spawn(proc() {
                    let buf_stream = BufferedStream::new(stream);
                    self.handle_client(buf_stream);
                })
            }
        }
    }
}
