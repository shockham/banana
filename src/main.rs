#![feature(convert)]
#![feature(collections)]

extern crate temple;

use temple::{html, elem};
use app::{App, Request};

pub mod app;

fn main() -> () {
    let mut a = App::new();

    fn this_handler(_:Request) -> String{
        "TESTTESTTEST".to_string()
    }

    fn another_handler(req:Request) -> String{
        let name:String = match req.query_string.get(&"name".to_string()) {
            Some(n) => n.clone(),
            None => String::from_str("anonymous"),
        };

        html("test",
            elem("h1", "title", "Hello!".to_string()) +
            elem("div", "container",
                elem("p", "", req.method) +
                elem("p", "", name).as_str()
            ).as_str()
        )
    }

    a.routes.insert("^/$", this_handler); 
    a.routes.insert("^/(?P<title>[^']+)$", another_handler);

    a.run("127.0.0.1:8080");
}
