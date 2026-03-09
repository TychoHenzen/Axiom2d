use axiom2d::prelude::*;

fn main() {
    let rect = Rect {
        x: Pixels(490.0),
        y: Pixels(260.0),
        width: Pixels(300.0),
        height: Pixels(200.0),
        color: Color::WHITE,
    };

    App::new()
        .set_window_config(WindowConfig {
            title: "Axiom2d Demo",
            ..Default::default()
        })
        .on_render(move |renderer| {
            renderer.clear(Color::new(0.392, 0.584, 0.929, 1.0));
            renderer.draw_rect(rect);
        })
        .run();
}
