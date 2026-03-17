pub(crate) const SHADER_SRC: &str = include_str!("quad.wgsl");
pub(crate) const SHAPE_SHADER_SRC: &str = include_str!("shape.wgsl");
pub(super) const BLOOM_PREAMBLE: &str = include_str!("bloom_preamble.wgsl");
pub(super) const BLOOM_SHADER_FRAG: &str = include_str!("bloom_frag.wgsl");
pub(super) const COMPOSITE_SHADER_FRAG: &str = include_str!("bloom_composite.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_shape_shader_parsed_then_no_error() {
        // Act
        let result = naga::front::wgsl::parse_str(SHAPE_SHADER_SRC);

        // Assert
        assert!(result.is_ok(), "WGSL parse error: {result:?}");
    }
}
