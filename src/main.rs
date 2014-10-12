use app::App;
pub mod app;

fn main() -> () {
    let mut a = App::new();

    fn this_handler() -> &'static str{
        "TESTTESTTEST"
    }

    fn another_handler() -> &'static str{
        "BOSBOSBOS"
    }

    a.routes.insert("/", this_handler);
    a.routes.insert("/bos", another_handler);

    a.run("127.0.0.1", 8080);
}
