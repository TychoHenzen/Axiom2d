#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowConfig {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Axiom2d",
            width: 1280,
            height: 720,
            vsync: true,
            resizable: true,
        }
    }
}
