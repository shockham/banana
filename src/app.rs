extern crate regex;

use self::regex::Regex;

use std::collections::HashMap;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::str;

pub struct App {
    pub routes: HashMap<&'static str,fn(req:Request) -> String>,
}

pub struct Request {
    pub method: String,
    pub query_string: HashMap<String, String>,
    pub route: String,
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

    fn process_request(request:String) -> Request {
            let req_re = match Regex::new("(?P<type>[A-Z^']+) (?P<route>[^']+) HTTP/(?P<http>[^']+)"){
                Ok(re) => re,
                Err(err) => panic!("{}", err),
            };
            let caps = req_re.captures(request.as_slice()).unwrap();
            let full_path: &str = caps.name("route").unwrap();
            let req_type: &str = caps.name("type").unwrap();
            
            let split_path: Vec<&str> = full_path.split('?').collect();
            let route = split_path[0];
            let query_string = match split_path.len() {
               1  => "",
                _ => split_path[1]
            };

            let query_pairs: Vec<&str> = query_string.split('&').collect();
            let mut query_map: HashMap<String, String> = HashMap::new();
            for pairs in query_pairs.iter() {
                let split_pairs: Vec<&str> = pairs.split('=').collect();
                if split_pairs.len() < 2 {
                    continue;
                }
                query_map.insert(String::from_str(split_pairs[0]), String::from_str(split_pairs[1]));
            }

            Request {
                method: String::from_str(req_type),
                query_string: query_map.clone(),
                route: String::from_str(route),
            }
    }
    
    fn handle_client(&self, stream: &mut TcpStream) -> () {
            //let mut request:String = String::from_str("GET /test?name=jef HTTP/1.1");
            //let _ = stream.read_to_string(&mut request).unwrap();
            
            let mut byte_req: [u8; 1024] = [0; 1024];
            let _ = stream.read(&mut byte_req).unwrap();
            
            let request:String = str::from_utf8(&byte_req).unwrap().to_string();

            println!("\n{}\n", request);
            
            let req:Request = App::process_request(request); 

            let mut content = String::from_str("Route does not exist");
            for (r, callback) in self.routes.iter() {
                let re = match Regex::new(*r){
                    Ok(re) => re,
                    Err(err) => panic!("{}", err),
                };
                let matched = re.is_match(req.route.as_slice());
                if matched {
                    let call_func = *callback;
                    content = call_func(req);
                    break;
                }
            }

            let with_headers = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\ncontent-length: {}\r\n\r\n{}",content.len(),content);

            println!("\n{}", with_headers);

            match stream.write_all(with_headers.as_slice().as_bytes()){
                Ok(_) => println!("ok"),
                Err(e) => panic!("{}", e),
            }
    }

    pub fn run(&self, address:&str) -> () {

        let acceptor = TcpListener::bind(address).unwrap();

        println!("||Starting server||\n{}", address);

        for stream in acceptor.incoming() {
            match stream {
                Err(e) => { println!("error: {}", e) }
                Ok(stream) => {
                    let l_app = self.clone();
                    thread::spawn(move || {
                        let mut mut_stream = stream;
                        //mut_stream.write_all("YO!".as_bytes());
                        l_app.handle_client(&mut mut_stream);
                    });
                }
            }
        }
    }
}
