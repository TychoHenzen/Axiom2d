use bytemuck::{Pod, Zeroable};

/// Unique terrain type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerrainId(pub u8);

/// Determines which sub-shader branch evaluates this terrain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TerrainKind {
    Grass = 0,
    Stone = 1,
    Water = 2,
    Sand = 3,
    Lava = 4,
    Snow = 5,
}

impl TerrainKind {
    pub const ALL: [Self; 6] = [
        Self::Grass,
        Self::Stone,
        Self::Water,
        Self::Sand,
        Self::Lava,
        Self::Snow,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Stone => "Stone",
            Self::Water => "Water",
            Self::Sand => "Sand",
            Self::Lava => "Lava",
            Self::Snow => "Snow",
        }
    }
}

/// Describes a terrain type's visual parameters. Colors and floats are
/// runtime-adjustable via the uniform buffer without recompiling the shader.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainMaterial {
    pub id: TerrainId,
    pub kind: TerrainKind,
    pub color_a: [f32; 3],
    pub color_b: [f32; 3],
    /// frequency, amplitude, warp strength, scale.
    pub params: [f32; 4],
    /// Type-specific tunables (wind direction, strata angle, etc.).
    pub extra: [f32; 4],
}

/// GPU-compatible material parameters. Matches the WGSL `MaterialParams` struct.
/// 64 bytes, 16-byte aligned.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MaterialParams {
    pub color_a: [f32; 4], // rgb + padding
    pub color_b: [f32; 4], // rgb + padding
    pub params: [f32; 4],  // frequency, amplitude, warp, scale
    pub extra: [f32; 4],   // type-specific
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl TerrainMaterial {
    /// Pack into GPU-compatible `MaterialParams`.
    #[must_use]
    pub fn to_gpu_params(&self) -> MaterialParams {
        MaterialParams {
            color_a: [self.color_a[0], self.color_a[1], self.color_a[2], 0.0],
            color_b: [self.color_b[0], self.color_b[1], self.color_b[2], 0.0],
            params: self.params,
            extra: self.extra,
        }
    }
}

/// Default material definitions for the initial terrain set.
#[must_use]
pub fn default_materials() -> Vec<TerrainMaterial> {
    vec![
        TerrainMaterial {
            id: TerrainId(0),
            kind: TerrainKind::Grass,
            color_a: [0.18, 0.42, 0.12],
            color_b: [0.30, 0.58, 0.20],
            params: [6.0, 0.4, 0.3, 1.0],
            extra: [0.3, 0.0, 0.0, 0.0], // wind direction
        },
        TerrainMaterial {
            id: TerrainId(1),
            kind: TerrainKind::Stone,
            color_a: [0.45, 0.42, 0.38],
            color_b: [0.58, 0.55, 0.50],
            params: [4.0, 0.3, 0.1, 8.0],
            extra: [0.0, 0.0, 0.0, 0.0],
        },
        TerrainMaterial {
            id: TerrainId(2),
            kind: TerrainKind::Water,
            color_a: [0.10, 0.25, 0.55],
            color_b: [0.20, 0.45, 0.70],
            params: [3.0, 0.5, 0.6, 6.0],
            extra: [1.0, 0.0, 0.0, 0.0], // animation speed
        },
        TerrainMaterial {
            id: TerrainId(3),
            kind: TerrainKind::Sand,
            color_a: [0.76, 0.65, 0.42],
            color_b: [0.85, 0.75, 0.52],
            params: [8.0, 0.2, 0.1, 1.0],
            extra: [0.5, 0.0, 0.0, 0.0], // ripple direction
        },
        TerrainMaterial {
            id: TerrainId(4),
            kind: TerrainKind::Lava,
            color_a: [0.25, 0.05, 0.02],
            color_b: [1.0, 0.35, 0.05],
            params: [5.0, 0.6, 0.4, 6.0],
            extra: [0.3, 0.0, 0.0, 0.0], // drift speed
        },
        TerrainMaterial {
            id: TerrainId(5),
            kind: TerrainKind::Snow,
            color_a: [0.90, 0.92, 0.95],
            color_b: [0.80, 0.85, 0.92],
            params: [4.0, 0.08, 0.05, 1.0],
            extra: [0.0, 0.0, 0.0, 0.0],
        },
    ]
}
