use axiom2d::prelude::*;

#[derive(Resource, Default)]
struct FrameCount(u64);

fn count_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

fn main() {
    let rect = Rect {
        x: Pixels(490.0),
        y: Pixels(260.0),
        width: Pixels(300.0),
        height: Pixels(200.0),
        color: Color::WHITE,
    };

    let mut app = App::new();
    app.world_mut().insert_resource(FrameCount::default());
    app.set_window_config(WindowConfig {
        title: "Axiom2d Demo",
        ..Default::default()
    })
    .add_systems(Phase::Update, count_frames)
    .on_render(move |renderer| {
        renderer.clear(Color::new(0.392, 0.584, 0.929, 1.0));
        renderer.draw_rect(rect);
    })
    .run();
}
