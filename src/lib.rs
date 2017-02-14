extern crate regex;
extern crate num_cpus;
#[macro_use] extern crate lazy_static;

use regex::Regex;

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::str;

/// Data related to a request1
pub struct Request {
    pub method: String,
    pub query_string: HashMap<String, String>,
    pub route: String,
}

/// HTTP response codes
pub enum ResponseCode {
    // 200
    Ok,
    // 400
    BadRequest,
    Unauthorised,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    // 500
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
}

/// Converts ResponseCode enum unto the str required in a http response
fn response_code_str(code:&ResponseCode) -> &'static str {
    match *code {
        ResponseCode::Ok => "200 OK",
        ResponseCode::BadRequest => "400 Bad Request",
        ResponseCode::Unauthorised => "401 Unauthorized",
        ResponseCode::Forbidden => "403 Forbidden",
        ResponseCode::NotFound => "404 Not Found",
        ResponseCode::MethodNotAllowed => "405 Method Not Allowed",
        ResponseCode::InternalServerError => "500 Internal Server Error",
        ResponseCode::NotImplemented => "501 Not Implemented",
        ResponseCode::BadGateway => "502 Bad Gateway",
        ResponseCode::ServiceUnavailable => "503 Service Unavailable",
    }
}

/// Data related to a response
pub struct Response {
    pub code: ResponseCode,
    pub content: String,
    pub mimetype: &'static str,
}

impl Response {
    /// Helper function for creating an Ok response with html mimetype
    pub fn ok_html(content: String) -> Response {
        Response {
            code: ResponseCode::Ok,
            content: content,
            mimetype: "text/html; charset=utf-8",
        }
    }
}

impl Response {
    /// Renders the Response to a string for sending
    pub fn create(&self) -> String {
        format!("HTTP/1.1 {}\r\nContent-Type: {}\r\ncontent-length: {}\r\n\r\n{}", 
                response_code_str(&self.code),
                self.mimetype,
                self.content.len(),
                self.content)
    }
}

/// Definition of an app
pub struct App {
    pub routes: HashMap<&'static str,fn(req:Request) -> Response>,
}

impl Clone for App{
    /// Creates a copy of the app 
    fn clone(&self) -> App{
        App{
            routes: self.routes.clone()
        }
    }
}

impl App  {
    /// Creates a new instance of an app
    pub fn new() -> App {
        App{
            routes : HashMap::new()
        }
    }

    /// Converts a query string into a HashMap
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

    /// Processes a request into a Request
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

    /// Routes a request to the correct callback based on Regex
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

    /// Handles the connecting client dealing with the Request and writing the Response
    fn handle_client(&self, stream: &mut TcpStream) { 
        let mut byte_req: [u8; 1024] = [0; 1024];
        let _ = stream.read(&mut byte_req).unwrap();

        let request:String = str::from_utf8(&byte_req).unwrap().to_string();
        let req:Request = App::process_request(request); 

        stream.write_all(self.route(req).create().as_bytes()).unwrap();
    }

    /// Starts the app at a particular address
    pub fn run(&self, address:&str) -> () {
        let worker_no = num_cpus::get();

        let listener = TcpListener::bind(address).unwrap();
        println!("||Starting server: http://{}\n||workers: {}\npress Ctrl-c to close", address, worker_no);

        //vec of the incoming streams
        let streams:Vec<TcpStream> = Vec::new();
        let data = Arc::new(Mutex::new(streams));

        //spawn some worker threads
        for _ in 0..worker_no {
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
                            let mut stream = s;
                            l_app.handle_client(&mut stream);
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
