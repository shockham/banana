extern crate regex;

use self::regex::Regex;

use std::collections::HashMap;

use std::io::{TcpListener, TcpStream, BufferedStream};
use std::io::{Acceptor, Listener};

pub struct App {
    pub routes: HashMap<&'static str,fn(req:Request) -> String>,
}

pub struct Request {
    pub method: String,
    pub query_string: String,
}

impl Clone for App{
    fn clone(&self) -> App{
        App{
            routes: self.routes.clone()
        }
    }
}

impl App  {
    pub fn new() -> App {
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
            
            let req_re = match Regex::new("(?P<type>[^']+) (?P<route>[^']+) (?P<http>[^']+)"){
                Ok(re) => re,
                Err(err) => panic!("{}", err),
            };
            let caps = req_re.captures(request.as_slice()).unwrap();
            let full_path: &str = caps.name("route");
            let req_type: &str = caps.name("type");
            
            let split_path: Vec<&str> = full_path
                .split('?')
                .collect();
            let route = split_path[0];
            let query_string = match split_path.len() {
               1  => "",
                _ => split_path[1]
            };
            println!("{}", query_string);

            let req = Request {
                method: String::from_str(req_type),
                query_string: String::from_str(query_string),
            };
            

            let mut content = String::from_str("Route does not exist");
            for (r, callback) in self.routes.iter() {
                let re = match Regex::new(*r){
                    Ok(re) => re,
                    Err(err) => panic!("{}", err),
                };
                let matched = re.is_match(route);
                if matched {
                    let call_func = *callback;
                    content = call_func(req);
                    break;
                }
            }

            let with_headers = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\ncontent-length: {}\r\n\r\n{}",content.len(),content);

            let outcome = stream.write_str(with_headers.as_slice());
            println!("-> {}", outcome);
    }

    pub fn run(&self, address:&str) -> () {

        let mut acceptor = TcpListener::bind(address).listen();

        println!("||Starting server||\n{}", address);

        for stream in acceptor.incoming() {
            match stream {
                Err(e) => { println!("error: {}", e) }
                Ok(stream) => {
                    let l_app = self.clone();
                    spawn(proc() {
                        let mut buf_stream = BufferedStream::new(stream);
                        l_app.handle_client(&mut buf_stream);
                    })
                }
            }
        }
    }
}
