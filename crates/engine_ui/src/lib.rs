pub mod interaction;
pub mod layout;
pub mod prelude;
pub mod render;
pub mod theme;
pub mod ui_event;
pub mod widget;

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::World;
    use engine_render::prelude::RendererRes;
    use engine_render::testing::{RectCallLog, SpyRenderer};

    pub fn make_spy_world() -> (World, Arc<Mutex<Vec<String>>>, RectCallLog) {
        let mut world = World::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let rect_cap: RectCallLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log)).with_rect_capture(Arc::clone(&rect_cap));
        world.insert_resource(RendererRes::new(Box::new(spy)));
        (world, log, rect_cap)
    }
}
