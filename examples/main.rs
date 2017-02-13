extern crate banana;

use banana::{App, Request, Response};

fn main() -> () {
    let mut a = App::new();

    fn this_handler(_:Request) -> Response {
        Response::ok_html("TESTTESTTEST".to_string())
    }

    fn another_handler(req:Request) -> Response {
        let name:String = match req.query_string.get(&"name".to_string()) {
            Some(n) => n.clone(),
            None => "anonymous".to_string(),
        };

        Response::ok_html(format!("Hello {}", name))
    }

    a.routes.insert("^/$", this_handler); 
    a.routes.insert("^/(?P<title>[^']+)$", another_handler);

    a.run("127.0.0.1:8080");
}
