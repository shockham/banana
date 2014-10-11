use app::App;
pub mod app;

fn main() -> () {
    let mut a = App::new();

    fn this_handler() -> &'static str{
        "TESTTESTTEST"
    }

    a.routes.insert("/", this_handler);

    a.run("127.0.0.1", 8080);
}
