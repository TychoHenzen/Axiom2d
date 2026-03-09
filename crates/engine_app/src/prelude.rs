pub use crate::app::{App, Plugin};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_prelude_imported_then_app_and_plugin_resolve() {
        // Act
        let _app = App::new();

        // Assert — compilation is the assertion
    }
}
