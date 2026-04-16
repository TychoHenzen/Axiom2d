use terrain::material::{
    MaterialParams, TerrainId, TerrainKind, TerrainMaterial, default_materials,
};

#[test]
fn when_packing_material_to_gpu_then_colors_are_padded_to_vec4() {
    // Arrange
    let mat = TerrainMaterial {
        id: TerrainId(0),
        kind: TerrainKind::Grass,
        color_a: [0.1, 0.2, 0.3],
        color_b: [0.4, 0.5, 0.6],
        params: [1.0, 2.0, 3.0, 4.0],
        extra: [5.0, 0.0, 0.0, 0.0],
    };

    // Act
    let gpu = mat.to_gpu_params();

    // Assert
    assert_eq!(gpu.color_a, [0.1, 0.2, 0.3, 0.0]);
    assert_eq!(gpu.color_b, [0.4, 0.5, 0.6, 0.0]);
    assert_eq!(gpu.params, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn when_calling_default_materials_then_returns_six_distinct_types() {
    // Act
    let materials = default_materials();

    // Assert
    assert_eq!(materials.len(), 6);
    for (i, mat) in materials.iter().enumerate() {
        assert_eq!(mat.id.0, i as u8);
    }
}

#[test]
fn when_gpu_params_size_then_is_64_bytes() {
    // Assert — validates WGSL alignment assumption
    assert_eq!(std::mem::size_of::<MaterialParams>(), 64);
}
