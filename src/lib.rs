extern crate regex;
extern crate bufstream;
#[macro_use] extern crate lazy_static;

use regex::Regex;
use bufstream::BufStream;

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::str;

pub struct App {
    pub routes: HashMap<&'static str,fn(req:Request) -> Response>,
}

pub struct Request {
    pub method: String,
    pub query_string: HashMap<String, String>,
    pub route: String,
}

pub enum ResponseCode {
    Ok,
    NotFound,
}

fn response_code_str(code:&ResponseCode) -> &'static str {
    match *code {
        ResponseCode::Ok => "200 OK",
        ResponseCode::NotFound => "404 Not Found",
    }
}

pub struct Response {
    pub code: ResponseCode,
    pub content: String,
    pub mimetype: &'static str,
}

impl Response {
    pub fn ok_html(content: String) -> Response {
        Response {
            code: ResponseCode::Ok,
            content: content,
            mimetype: "text/plain; charset=utf-8",
        }
    }
}

impl Response {
    pub fn create(&self) -> String {
        format!("HTTP/1.1 {}\r\nContent-Type: {}\r\ncontent-length: {}\r\n\r\n{}", 
                response_code_str(&self.code),
                self.mimetype,
                self.content.len(),
                self.content)
    }
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

    fn create_query_map(query_string:&str) -> HashMap<String, String> {
        let query_pairs: Vec<&str> = query_string.split('&').collect();
        let mut query_map: HashMap<String, String> = HashMap::new();
        for pairs in query_pairs.iter() {
            let split_pairs: Vec<&str> = pairs.split('=').collect();
            if split_pairs.len() < 2 {
                continue;
            }
            query_map.insert(split_pairs[0].to_string(), split_pairs[1].to_string());
        }

        query_map
    }

    fn process_request(request:String) -> Request {
        lazy_static! {
            static ref REQ_RE: Regex = Regex::new("(?P<type>[A-Z^']+) (?P<route>[^']+) HTTP/(?P<http>[^']+)").unwrap();
        }
        let (full_path, req_type) = match REQ_RE.captures(request.as_str()) {
            Some(caps) => {
                let full_path = caps.name("route").unwrap().as_str();
                let req_type = caps.name("type").unwrap().as_str();
                (full_path, req_type)
            },
            None => {
                println!("req:{}", request);
                println!("err:falling back to default");
                ("/", "GET")
            }
        };

        let split_path: Vec<&str> = full_path.split('?').collect();
        let route = split_path[0];
        let query_string = match split_path.len() {
            1  => "",
            _ => split_path[1]
        };

        Request {
            method: req_type.to_string(),
            query_string: App::create_query_map(query_string),
            route: route.to_string(),
        }
    }

    fn route(&self, req:Request) -> Response {
        for (r, callback) in self.routes.iter() {
            let re = Regex::new(*r).unwrap();
            if re.is_match(req.route.as_str()) {
                let call_func = *callback;
                return call_func(req);
            }
        }
        
        Response {
            code: ResponseCode::NotFound,
            content: "NOT FOUND".to_string(),
            mimetype: "text/plain; charset=utf-8",
        }
    }

    fn handle_client(&self, stream: &mut BufStream<TcpStream>) { 
        let mut byte_req: [u8; 1024] = [0; 1024];
        let _ = stream.read(&mut byte_req).unwrap();

        let request:String = str::from_utf8(&byte_req).unwrap().to_string();
        let req:Request = App::process_request(request); 

        stream.write_all(self.route(req).create().as_bytes()).unwrap();
    }

    pub fn run(&self, address:&str) -> () {
        const WORKER_NO:i32 = 8;

        let listener = TcpListener::bind(address).unwrap();
        println!("||Starting server||:press Ctrl-c to close\nhttp://{}", address);

        //vec of the incoming streams
        let streams:Vec<TcpStream> = Vec::new();
        let data = Arc::new(Mutex::new(streams));

        //spawn some worker threads
        for _ in 0..WORKER_NO {
            let data = data.clone();
            let l_app = self.clone();
            thread::spawn(move || {
                loop {
                    let mut data = match data.lock() {
                        Ok(data) => data,
                        Err(e) => {
                            println!("{}", e);
                            break;
                        },
                    };

                    let _ = match data.pop() {
                        Some(s) => {
                            let mut stream = BufStream::new(s);
                            l_app.handle_client(&mut stream)
                        },
                        None => (),
                    };
                }
            });
        }    

        //loop for accepting requests
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let data = data.clone();
                    let mut data = match data.lock() {
                        Ok(data) => data,
                        Err(e) => {
                            println!("{}", e);
                            break;
                        },
                    };
                    data.push(stream);
                },
                Err(e) => println!("error: {}", e),
            }
        }
        
        drop(listener);
    }
}
