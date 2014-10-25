use app::App;
use html::{html, elem};
pub mod app;
pub mod html;

fn main() -> () {
    let mut a = App::new();

    fn this_handler() -> String{
        String::from_str("TESTTESTTEST")
    }

    fn another_handler() -> String{
        html("test",
            elem("div", "title", String::from_str("Hello!")) +
            elem("div", "whut",
                elem("p", "", String::from_str("Some text"))
            )
        )
    }

    a.routes.insert("/", this_handler);
    a.routes.insert("/bos", another_handler);

    a.run("127.0.0.1", 8080);
}
