pub(crate) const SHADER_SRC: &str = include_str!("quad.wgsl");
pub(crate) const SHAPE_SHADER_SRC: &str = include_str!("shape.wgsl");
pub(super) const BLOOM_PREAMBLE: &str = include_str!("bloom_preamble.wgsl");
pub(super) const BLOOM_SHADER_FRAG: &str = include_str!("bloom_frag.wgsl");
pub(super) const COMPOSITE_SHADER_FRAG: &str = include_str!("bloom_composite.wgsl");
