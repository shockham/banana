#![feature(convert)]

extern crate regex;
extern crate bufstream;

use regex::Regex;
use bufstream::BufStream;

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
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
        let caps = req_re.captures(request.as_str()).unwrap();
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
            query_map.insert(split_pairs[0].to_string(), split_pairs[1].to_string());
        }

        Request {
            method: req_type.to_string(),
            query_string: query_map.clone(),
            route: route.to_string(),
        }
    }

    fn handle_client(&self, stream: &mut BufStream<TcpStream>) -> () { 
        let mut byte_req: [u8; 1024] = [0; 1024];
        let _ = stream.read(&mut byte_req).unwrap();

        let request:String = str::from_utf8(&byte_req).unwrap().to_string();
        let req:Request = App::process_request(request); 

        let mut content = "Route does not exist".to_string();
        for (r, callback) in self.routes.iter() {
            let re = match Regex::new(*r){
                Ok(re) => re,
                Err(err) => panic!("{}", err),
            };
            let matched = re.is_match(req.route.as_str());
            if matched {
                let call_func = *callback;
                content = call_func(req);
                break;
            }
        }

        let with_headers = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\ncontent-length: {}\r\n\r\n{}",content.len(),content);

        match stream.write_all(with_headers.as_str().as_bytes()){
            Ok(_) => return,
            Err(e) => panic!("{}", e),
        }
    }

    pub fn run(&self, address:&str) -> () {
        let worker_no = 4;

        let listener = match TcpListener::bind(address){
            Ok(acc) => acc,
            Err(e) => panic!("{}", e),
        };

        println!("||Starting server||\nhttp://{}", address);

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
