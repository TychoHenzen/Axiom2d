use axiom2d::prelude::App;
use demo::setup;

fn main() {
    let mut app = App::new();
    setup(&mut app);
    app.run();
}
