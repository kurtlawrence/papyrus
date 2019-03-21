extern crate azul;
use azul::prelude::*;

struct MyApp;

impl Layout for MyApp {
    fn layout(&self, _: LayoutInfo<Self>) -> Dom<Self> {
        println!("called layout()",);

        Dom::label("hello")
    }
}

fn main() {
    let mut app = App::new(MyApp, AppConfig::default()).unwrap();

    let window = app
        .create_window(WindowCreateOptions::default(), css::native())
        .unwrap();

    app.run(window).unwrap();
}
