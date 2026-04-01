#![allow(clippy::unwrap_used, clippy::float_cmp)]

use engine_core::types::TextureId;
use engine_render::atlas::{
    AtlasBuilder, AtlasError, AtlasUploaded, normalize_uv_rect, upload_atlas_system,
};
use engine_render::renderer::RendererRes;
use engine_render::testing::SpyRenderer;
use engine_render::testing::helpers::minimal_atlas;

#[test]
fn when_building_empty_atlas_then_pixel_buffer_is_all_zeros() {
    // Arrange
    let builder = AtlasBuilder::new(4, 4);

    // Act
    let atlas = builder.build();

    // Assert
    assert_eq!(atlas.data, vec![0u8; 4 * 4 * 4]);
}

#[test]
fn when_building_atlas_with_image_then_buffer_size_matches_atlas() {
    // Arrange
    let mut builder = AtlasBuilder::new(64, 128);
    let pixel_data = vec![255u8; 2 * 2 * 4];
    builder.add_image(2, 2, &pixel_data).unwrap();

    // Act
    let atlas = builder.build();

    // Assert
    assert_eq!(atlas.data.len(), 64 * 128 * 4);
}

#[test]
fn when_builder_created_then_reports_matching_dimensions() {
    // Arrange
    let builder = AtlasBuilder::new(512, 256);

    // Act
    let w = builder.width();
    let h = builder.height();

    // Assert
    assert_eq!(w, 512);
    assert_eq!(h, 256);
}

#[test]
fn when_adding_single_image_then_returns_handle_with_valid_texture_id() {
    // Arrange
    let mut builder = AtlasBuilder::new(512, 512);

    // Act
    let result = builder.add_image(1, 1, &[255, 0, 0, 255]);

    // Assert
    assert!(result.is_ok());
}

/// @doc: Atlas UV normalization maps pixel rects to 0..1 -- shader sampling requires normalized coordinates
#[test]
fn when_adding_image_then_uv_rect_is_normalized_to_zero_one() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);

    // Act
    let handle = builder.add_image(2, 2, &[255; 16]).unwrap();

    // Assert
    let [u0, v0, u1, v1] = handle.uv_rect;
    assert!((0.0..=1.0).contains(&u0));
    assert!((0.0..=1.0).contains(&v0));
    assert!((0.0..=1.0).contains(&u1));
    assert!((0.0..=1.0).contains(&v1));
    assert!(u1 > u0, "uv_rect must have positive width");
    assert!(v1 > v0, "uv_rect must have positive height");
}

#[test]
fn when_adding_image_that_fills_atlas_then_uv_rect_is_full_range() {
    // Arrange
    let mut builder = AtlasBuilder::new(4, 4);

    // Act
    let handle = builder.add_image(4, 4, &[255; 64]).unwrap();

    // Assert
    assert_eq!(handle.uv_rect, [0.0, 0.0, 1.0, 1.0]);
}

/// @doc: Each allocation must have unique `TextureId` -- id collisions break texture lookups and render incorrect sprites
#[test]
fn when_adding_two_images_then_each_has_distinct_texture_id() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);

    // Act
    let h1 = builder.add_image(2, 2, &[255; 16]).unwrap();
    let h2 = builder.add_image(2, 2, &[0; 16]).unwrap();

    // Assert
    assert_ne!(h1.texture_id, h2.texture_id);
}

/// @doc: Guillotiere allocator must produce non-overlapping rects -- overlaps corrupt texture sampling
#[test]
fn when_adding_two_images_then_uv_rects_do_not_overlap() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);

    // Act
    let h1 = builder.add_image(4, 4, &[255; 64]).unwrap();
    let h2 = builder.add_image(4, 4, &[0; 64]).unwrap();

    // Assert -- convert to pixel rects and check no overlap
    let [u0a, v0a, u1a, v1a] = h1.uv_rect;
    let [u0b, v0b, u1b, v1b] = h2.uv_rect;
    let no_overlap = u1a <= u0b || u1b <= u0a || v1a <= v0b || v1b <= v0a;
    assert!(
        no_overlap,
        "uv_rects overlap: {:?} vs {:?}",
        h1.uv_rect, h2.uv_rect
    );
}

/// @doc: Pairwise non-overlapping invariant must hold under load -- overlaps cause texture corruption across multiple draw calls
#[test]
fn when_adding_many_images_then_all_uv_rects_are_non_overlapping() {
    // Arrange
    let mut builder = AtlasBuilder::new(512, 512);
    let pixel_data = [128u8; 32 * 32 * 4];

    // Act
    let handles: Vec<_> = (0..16)
        .map(|_| builder.add_image(32, 32, &pixel_data).unwrap())
        .collect();

    // Assert -- pairwise non-overlap
    for i in 0..handles.len() {
        for j in (i + 1)..handles.len() {
            let [u0a, v0a, u1a, v1a] = handles[i].uv_rect;
            let [u0b, v0b, u1b, v1b] = handles[j].uv_rect;
            let no_overlap = u1a <= u0b || u1b <= u0a || v1a <= v0b || v1b <= v0a;
            assert!(no_overlap, "handles {i} and {j} overlap");
        }
    }
}

#[test]
fn when_adding_image_larger_than_atlas_then_returns_no_space_error() {
    // Arrange
    let mut builder = AtlasBuilder::new(8, 8);

    // Act
    let result = builder.add_image(16, 16, &[0; 16 * 16 * 4]);

    // Assert
    assert!(matches!(result, Err(AtlasError::NoSpace)));
}

#[test]
fn when_atlas_full_then_returns_no_space_error() {
    // Arrange
    let mut builder = AtlasBuilder::new(4, 4);
    builder.add_image(4, 4, &[255; 64]).unwrap();

    // Act
    let result = builder.add_image(1, 1, &[0; 4]);

    // Assert
    assert!(matches!(result, Err(AtlasError::NoSpace)));
}

#[test]
fn when_data_length_mismatches_then_returns_error() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);

    // Act
    let result = builder.add_image(1, 1, &[255, 0, 0]);

    // Assert
    assert!(matches!(
        result,
        Err(AtlasError::DataLengthMismatch {
            expected: 4,
            actual: 3
        })
    ));
}

#[test]
fn when_adding_zero_width_image_then_returns_invalid_dimensions() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);

    // Act
    let result = builder.add_image(0, 4, &[]);

    // Assert
    assert!(matches!(result, Err(AtlasError::InvalidDimensions)));
}

#[test]
fn when_adding_zero_height_image_then_returns_invalid_dimensions() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);

    // Act
    let result = builder.add_image(4, 0, &[]);

    // Assert
    assert!(matches!(result, Err(AtlasError::InvalidDimensions)));
}

/// @doc: `TextureAtlas::lookup` must return exact `handle.uv_rect` from `add_image` -- mismatch breaks sprite sampling
#[test]
fn when_looking_up_known_texture_id_then_returns_matching_uv_rect() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);
    let handle = builder.add_image(2, 2, &[255; 16]).unwrap();
    let atlas = builder.build();

    // Act
    let result = atlas.lookup(handle.texture_id);

    // Assert
    assert_eq!(result, Some(handle.uv_rect));
}

/// @doc: Lookup must return None for non-existent `TextureId` -- catching invalid IDs prevents shader sampling garbage
#[test]
fn when_looking_up_unknown_texture_id_then_returns_none() {
    // Arrange
    let mut builder = AtlasBuilder::new(256, 256);
    builder.add_image(2, 2, &[255; 16]).unwrap();
    let atlas = builder.build();

    // Act
    let result = atlas.lookup(TextureId(99));

    // Assert
    assert_eq!(result, None);
}

/// @doc: Multiple lookups must each return their unique UV rects -- aliasing or collision breaks multi-sprite rendering
#[test]
fn when_looking_up_multiple_textures_then_each_returns_its_own_uv_rect() {
    // Arrange
    let mut builder = AtlasBuilder::new(64, 64);
    let h1 = builder.add_image(4, 4, &[255; 64]).unwrap();
    let h2 = builder.add_image(4, 4, &[128; 64]).unwrap();
    let h3 = builder.add_image(4, 4, &[0; 64]).unwrap();
    let atlas = builder.build();

    // Act + Assert
    assert_eq!(atlas.lookup(h1.texture_id), Some(h1.uv_rect));
    assert_eq!(atlas.lookup(h2.texture_id), Some(h2.uv_rect));
    assert_eq!(atlas.lookup(h3.texture_id), Some(h3.uv_rect));
    assert_ne!(h1.uv_rect, h2.uv_rect);
    assert_ne!(h2.uv_rect, h3.uv_rect);
}

/// @doc: Pixel row-major memory layout: compute offset correctly with stride to avoid data corruption
#[test]
fn when_building_atlas_then_pixel_data_appears_at_correct_offset() {
    // Arrange
    let mut builder = AtlasBuilder::new(8, 8);
    let red = [255, 0, 0, 255].repeat(2 * 2);
    let handle = builder.add_image(2, 2, &red).unwrap();

    // Act
    let atlas = builder.build();

    // Assert -- sample the top-left pixel of the allocation
    let [u0, v0, _, _] = handle.uv_rect;
    let px = (u0 * atlas.width as f32) as usize;
    let py = (v0 * atlas.height as f32) as usize;
    let offset = (py * atlas.width as usize + px) * 4;
    assert_eq!(&atlas.data[offset..offset + 4], &[255, 0, 0, 255]);
}

/// @doc: Two images must not overwrite each other's pixels in final atlas -- overlapping writes cause color bleed
#[test]
fn when_building_atlas_with_two_images_then_neither_overwrites_the_other() {
    // Arrange
    let mut builder = AtlasBuilder::new(16, 16);
    let red = [255, 0, 0, 255].repeat(2 * 2);
    let blue = [0, 0, 255, 255].repeat(2 * 2);
    let h_red = builder.add_image(2, 2, &red).unwrap();
    let h_blue = builder.add_image(2, 2, &blue).unwrap();

    // Act
    let atlas = builder.build();

    // Assert -- sample one pixel from each allocation
    let sample = |uv: [f32; 4]| -> &[u8] {
        let px = (uv[0] * atlas.width as f32) as usize;
        let py = (uv[1] * atlas.height as f32) as usize;
        let off = (py * atlas.width as usize + px) * 4;
        &atlas.data[off..off + 4]
    };
    assert_eq!(sample(h_red.uv_rect), &[255, 0, 0, 255]);
    assert_eq!(sample(h_blue.uv_rect), &[0, 0, 255, 255]);
}

/// @doc: UV normalization must preserve aspect ratio -- wrong formula causes stretched textures
#[test]
fn when_normalizing_uv_rect_then_output_is_in_zero_one_range() {
    // Act
    let uv = normalize_uv_rect(10, 20, 40, 60, (100, 100));

    // Assert
    assert_eq!(uv, [0.10, 0.20, 0.50, 0.80]);
}

#[test]
fn when_normalizing_uv_rect_at_origin_then_starts_at_zero() {
    // Act
    let uv = normalize_uv_rect(0, 0, 32, 32, (256, 256));

    // Assert
    assert_eq!(uv, [0.0, 0.0, 0.125, 0.125]);
}

/// @doc: Each pixel row must respect atlas stride -- incorrect stride calculation causes row-major layout corruption
#[test]
fn when_building_atlas_then_all_rows_of_image_are_correctly_placed() {
    // Arrange
    let mut builder = AtlasBuilder::new(4, 4);
    // Row 0: red, green; Row 1: blue, white
    #[rustfmt::skip]
    let data = [
        255, 0, 0, 255,    0, 255, 0, 255,
        0, 0, 255, 255,    255, 255, 255, 255,
    ];
    let handle = builder.add_image(2, 2, &data).unwrap();

    // Act
    let atlas = builder.build();

    // Assert
    let [u0, v0, _, _] = handle.uv_rect;
    let px = (u0 * atlas.width as f32) as usize;
    let py = (v0 * atlas.height as f32) as usize;
    let stride = atlas.width as usize * 4;
    assert_eq!(&atlas.data[py * stride + px * 4..][..4], [255, 0, 0, 255]);
    assert_eq!(
        &atlas.data[py * stride + (px + 1) * 4..][..4],
        [0, 255, 0, 255]
    );
    assert_eq!(
        &atlas.data[(py + 1) * stride + px * 4..][..4],
        [0, 0, 255, 255]
    );
    assert_eq!(
        &atlas.data[(py + 1) * stride + (px + 1) * 4..][..4],
        [255, 255, 255, 255]
    );
}

/// @doc: Second image offset must not change UV height calculation -- UV rect is always normalized correctly
#[test]
fn when_second_image_at_nonzero_y_then_uv_height_matches_image_ratio() {
    // Arrange -- narrow atlas forces second image to y > 0
    let mut builder = AtlasBuilder::new(4, 8);
    builder.add_image(4, 4, &[0u8; 64]).unwrap();

    // Act
    let h2 = builder.add_image(2, 2, &[0u8; 16]).unwrap();

    // Assert
    let uv_height = h2.uv_rect[3] - h2.uv_rect[1];
    let expected = 2.0 / 8.0;
    assert!(
        (uv_height - expected).abs() < 1e-6,
        "UV height {uv_height} should be {expected}"
    );
}

#[test]
fn when_second_image_offset_then_handle_uv_matches_build_lookup() {
    // Arrange -- narrow atlas forces second image to y > 0
    let mut builder = AtlasBuilder::new(4, 8);
    builder.add_image(4, 4, &[0u8; 4 * 4 * 4]).unwrap();
    let data = [255u8; 2 * 2 * 4];
    let handle = builder.add_image(2, 2, &data).unwrap();

    // Act
    let atlas = builder.build();

    // Assert -- handle UV (from add_image) must match lookup (from build)
    let looked_up = atlas.lookup(handle.texture_id).unwrap();
    assert_eq!(handle.uv_rect, looked_up);

    // Also verify pixel data at the UV location
    let [u0, v0, _, _] = handle.uv_rect;
    let px = (u0 * atlas.width as f32) as usize;
    let py = (v0 * atlas.height as f32) as usize;
    let stride = atlas.width as usize * 4;
    let off = py * stride + px * 4;
    assert_eq!(&atlas.data[off..off + 4], [255, 255, 255, 255]);
}

fn insert_spy(world: &mut bevy_ecs::world::World) -> std::sync::Arc<std::sync::Mutex<Vec<String>>> {
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    log
}

fn run_system(world: &mut bevy_ecs::world::World) {
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(upload_atlas_system);
    schedule.run(world);
}

/// @doc: `upload_atlas_system` must invoke renderer when `TextureAtlas` resource exists -- GPU upload won't happen without this call
#[test]
fn when_atlas_present_then_upload_atlas_called() {
    // Arrange
    let mut world = bevy_ecs::world::World::new();
    let log = insert_spy(&mut world);
    world.insert_resource(minimal_atlas());

    // Act
    run_system(&mut world);

    // Assert
    assert!(log.lock().unwrap().contains(&"upload_atlas".to_string()));
}

/// @doc: `upload_atlas_system` must skip when no `TextureAtlas` resource -- prevents spam calls and respects resource-missing contracts
#[test]
fn when_no_atlas_then_upload_atlas_not_called() {
    // Arrange
    let mut world = bevy_ecs::world::World::new();
    let log = insert_spy(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    assert!(!log.lock().unwrap().contains(&"upload_atlas".to_string()));
}

/// @doc: `AtlasUploaded` marker prevents re-uploading -- GPU resource should only transfer once
#[test]
fn when_system_runs_twice_then_upload_atlas_called_only_once() {
    // Arrange
    let mut world = bevy_ecs::world::World::new();
    let log = insert_spy(&mut world);
    world.insert_resource(minimal_atlas());
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(upload_atlas_system);

    // Act
    schedule.run(&mut world);
    schedule.run(&mut world);

    // Assert
    let calls: Vec<_> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|s| *s == "upload_atlas")
        .cloned()
        .collect();
    assert_eq!(calls.len(), 1);
}

/// @doc: `AtlasUploaded` marker presence must prevent re-upload -- double-upload causes GPU resource leaks and redundant transfers
#[test]
fn when_atlas_uploaded_marker_present_then_upload_atlas_not_called() {
    // Arrange
    let mut world = bevy_ecs::world::World::new();
    let log = insert_spy(&mut world);
    world.insert_resource(minimal_atlas());
    world.insert_resource(AtlasUploaded);

    // Act
    run_system(&mut world);

    // Assert
    assert!(!log.lock().unwrap().contains(&"upload_atlas".to_string()));
}
