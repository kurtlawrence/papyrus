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
    let app = App::new(MyApp, AppConfig::default());

    let window = Window::new(WindowCreateOptions::default(), css::native()).unwrap();

    app.run(window).unwrap();
}
