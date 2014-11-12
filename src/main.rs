use app::{App, Request};
use html::{html, elem};
pub mod app;
pub mod html;

fn main() -> () {
    let mut a = App::new();

    fn this_handler(req:Request) -> String{
        String::from_str("TESTTESTTEST")
    }

    fn another_handler(req:Request) -> String{
        let name:String = match req.query_string.get(&String::from_str("name")) {
            Some(n) => n.clone(),
            None => String::from_str("anonymous"),
        };

        html("test",
            elem("h1", "title", String::from_str("Hello!")) +
            elem("div", "container",
                elem("p", "", req.method) +
                elem("p", "", name) 
            )
        )
    }

    a.routes.insert("^/$", this_handler);
    a.routes.insert("/(?P<title>[^']+)", another_handler);

    a.run("127.0.0.1:8080");
}
