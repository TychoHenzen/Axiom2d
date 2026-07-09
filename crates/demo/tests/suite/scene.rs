#![allow(clippy::unwrap_used)]

use axiom2d::prelude::*;
use demo::{
    setup,
    types::{Earth, Moon, OrbitalSpeed, SUN_COLOR, Sun, SynodicFrame, action},
};

const PLANET_COUNT: usize = 8;
const MOON_COUNT: usize = 1;

#[test]
fn when_setup_called_then_all_celestial_entities_exist() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut shape_query = world.query::<(&Shape, &RenderLayer)>();
    let world_shapes = shape_query
        .iter(world)
        .filter(|(_, layer)| **layer == RenderLayer::World)
        .count();
    let mut sprite_query = world.query::<&Sprite>();
    assert_eq!(
        world_shapes + sprite_query.iter(world).count(),
        1 + PLANET_COUNT + MOON_COUNT,
        "total world shapes plus sprites should equal sun + planets + moon"
    );
}

#[test]
fn when_setup_called_then_camera2d_entity_exists() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<&Camera2D>();
    assert_eq!(query.iter(world).count(), 1, "exactly one Camera2D entity should exist after setup");
}

#[test]
fn when_setup_called_then_action_map_has_all_bindings() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let action_map = world.get_resource::<ActionMap>().unwrap();
    assert!(
        !action_map.bindings_for(action::ZOOM_IN).is_empty(),
        "ZOOM_IN should have at least one binding in action map"
    );
    assert!(
        !action_map.bindings_for(action::ZOOM_OUT).is_empty(),
        "ZOOM_OUT should have at least one binding in action map"
    );
}

#[test]
fn when_setup_called_then_exactly_one_synodic_frame_exists() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<&SynodicFrame>();
    assert_eq!(query.iter(world).count(), 1, "exactly one SynodicFrame entity should exist after setup");
}

#[test]
fn when_setup_called_then_exactly_one_sun_entity_exists() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<&Sun>();
    assert_eq!(query.iter(world).count(), 1, "exactly one Sun entity should exist after setup");
}

#[test]
fn when_setup_called_then_exactly_one_earth_entity_exists() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<&Earth>();
    assert_eq!(query.iter(world).count(), 1, "exactly one Earth entity should exist after setup");
}

#[test]
fn when_setup_called_then_sun_has_yellow_color() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);

    // Act
    let world = app.world_mut();
    let mut query = world.query::<(&Sun, &Shape)>();
    let (_, shape) = query.single(world).unwrap();

    // Assert
    assert_eq!(shape.color, SUN_COLOR, "sun shape should have the configured SUN_COLOR");
}

#[test]
fn when_setup_called_then_correct_number_of_orbiting_body_pivots() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let frame = world
        .query::<(Entity, &SynodicFrame)>()
        .single(world)
        .unwrap()
        .0;
    let mut query = world.query::<(Entity, &OrbitalSpeed, &ChildOf)>();
    let pivots: Vec<Entity> = query
        .iter(world)
        .filter(|(_, _, parent)| parent.0 == frame)
        .map(|(entity, _, _)| entity)
        .collect();
    assert_eq!(pivots.len(), PLANET_COUNT, "should have exactly {PLANET_COUNT} orbiting body pivots");
}

#[test]
fn when_setup_called_then_each_pivot_has_one_body_child() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);
    let world = app.world_mut();
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);

    // Act / Assert
    let frame = world
        .query::<(Entity, &SynodicFrame)>()
        .single(world)
        .unwrap()
        .0;
    let mut pivot_query = world.query::<(Entity, &OrbitalSpeed, &ChildOf)>();
    let pivots: Vec<Entity> = pivot_query
        .iter(world)
        .filter(|(_, _, parent)| parent.0 == frame)
        .map(|(e, _, _)| e)
        .collect();
    assert_eq!(pivots.len(), PLANET_COUNT, "should have {PLANET_COUNT} orbital pivots after hierarchy maintenance");
    for pivot in pivots {
        let children = world.get::<Children>(pivot).unwrap();
        assert_eq!(children.0.len(), 1, "each orbital pivot should have exactly one child body");
        let child = children.0[0];
        assert!(world.get::<Shape>(child).is_some(), "orbital body child should have a Shape component");
    }
}

#[test]
fn when_setup_called_then_all_planets_on_world_render_layer() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);
    let world = app.world_mut();
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);

    // Act / Assert
    let frame = world
        .query::<(Entity, &SynodicFrame)>()
        .single(world)
        .unwrap()
        .0;
    let mut pivot_query = world.query::<(Entity, &OrbitalSpeed, &ChildOf)>();
    let pivots: Vec<Entity> = pivot_query
        .iter(world)
        .filter(|(_, _, parent)| parent.0 == frame)
        .map(|(e, _, _)| e)
        .collect();
    for pivot in pivots {
        let children = world.get::<Children>(pivot).unwrap();
        let layer = world.get::<RenderLayer>(children.0[0]).unwrap();
        assert_eq!(*layer, RenderLayer::World, "all planet bodies should be on World render layer");
    }
}

#[test]
fn when_setup_called_then_moon_exists_with_moon_marker() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<&Moon>();
    assert_eq!(query.iter(world).count(), MOON_COUNT, "exactly {MOON_COUNT} entity should have Moon marker");
}

#[test]
fn when_setup_called_then_moon_is_child_of_earth() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);
    let world = app.world_mut();
    let earth = world.query::<(Entity, &Earth)>().single(world).unwrap().0;

    // Act
    let moon = world.query::<(Entity, &Moon)>().single(world).unwrap().0;

    // Assert
    let parent = world.get::<ChildOf>(moon).unwrap().0;
    assert!(world.get::<OrbitalSpeed>(parent).is_some(), "moon's direct parent should be an orbital pivot (has OrbitalSpeed)");
    let grandparent = world.get::<ChildOf>(parent).unwrap().0;
    assert_eq!(grandparent, earth, "moon's grandparent should be the Earth entity");
}

#[test]
fn when_setup_called_then_bloom_settings_exist() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    assert!(world.get_resource::<BloomSettings>().is_some(), "BloomSettings resource should be present after setup");
}

#[test]
fn when_bloom_settings_queried_then_enabled() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);

    // Act
    let world = app.world_mut();
    let bloom = world.get_resource::<BloomSettings>().unwrap();

    // Assert
    assert!(bloom.enabled, "BloomSettings.enabled should be true after setup");
}

#[test]
fn when_setup_called_then_sun_is_circle_shape() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<(&Sun, &Shape)>();
    let (_, shape) = query.single(world).unwrap();
    assert!(matches!(shape.variant, ShapeVariant::Circle { .. }), "sun shape variant should be Circle");
}

#[test]
fn when_orbiting_bodies_queried_then_each_has_shape() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);
    let world = app.world_mut();
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);

    // Act
    let frame = world
        .query::<(Entity, &SynodicFrame)>()
        .single(world)
        .unwrap()
        .0;
    let mut pivot_query = world.query::<(Entity, &OrbitalSpeed, &ChildOf)>();
    let pivots: Vec<Entity> = pivot_query
        .iter(world)
        .filter(|(_, _, parent)| parent.0 == frame)
        .map(|(e, _, _)| e)
        .collect();

    // Assert
    assert_eq!(pivots.len(), PLANET_COUNT, "should have {PLANET_COUNT} orbital pivots before checking shapes");
    for pivot in pivots {
        let children = world.get::<Children>(pivot).unwrap();
        let body = children.0[0];
        assert!(
            world.get::<Shape>(body).is_some(),
            "orbiting body child should have a Shape component"
        );
    }
}

#[test]
fn when_orbiting_body_shapes_queried_then_distinct_colors() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);
    let world = app.world_mut();
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);

    // Act
    let frame = world
        .query::<(Entity, &SynodicFrame)>()
        .single(world)
        .unwrap()
        .0;
    let mut pivot_query = world.query::<(Entity, &OrbitalSpeed, &ChildOf)>();
    let pivots: Vec<Entity> = pivot_query
        .iter(world)
        .filter(|(_, _, parent)| parent.0 == frame)
        .map(|(e, _, _)| e)
        .collect();
    let mut colors = Vec::new();
    for pivot in pivots {
        let children = world.get::<Children>(pivot).unwrap();
        let shape = world.get::<Shape>(children.0[0]).unwrap();
        colors.push(shape.color);
    }

    // Assert
    let unique: std::collections::HashSet<u32> = colors
        .iter()
        .map(|c| {
            let r = (c.r * 255.0) as u32;
            let g = (c.g * 255.0) as u32;
            let b = (c.b * 255.0) as u32;
            (r << 16) | (g << 8) | b
        })
        .collect();
    assert_eq!(unique.len(), PLANET_COUNT, "each planet should have a distinct color");
}

#[test]
fn when_sprites_queried_then_only_moons_remain() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<&Sprite>();
    assert_eq!(query.iter(world).count(), MOON_COUNT, "exactly {MOON_COUNT} Sprite entities should exist (the moon)");
}

#[test]
fn when_shapes_queried_then_sun_plus_planets_are_circles() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<(&Shape, &RenderLayer)>();
    let world_circle_count = query
        .iter(world)
        .filter(|(s, layer)| {
            **layer == RenderLayer::World && matches!(s.variant, ShapeVariant::Circle { .. })
        })
        .count();
    assert_eq!(world_circle_count, 1 + PLANET_COUNT, "sun plus all planets should be circles on World layer");
}

#[test]
fn when_background_shapes_queried_then_at_least_one_exists() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<(&Shape, &RenderLayer)>();
    let bg_count = query
        .iter(world)
        .filter(|(_, layer)| **layer == RenderLayer::Background)
        .count();
    assert!(bg_count > 0, "expected at least one background shape");
}

#[test]
fn when_background_shapes_queried_then_all_additive_blend() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);

    // Act
    let world = app.world_mut();
    let mut query = world.query::<(&RenderLayer, &Material2d)>();

    // Assert
    for (layer, material) in query.iter(world) {
        if *layer == RenderLayer::Background {
            assert_eq!(material.blend_mode, BlendMode::Additive, "all background layer shapes should use Additive blend mode");
        }
    }
}

#[test]
fn when_background_shapes_queried_then_polygon_clusters_exist() {
    // Arrange
    let mut app = App::new();

    // Act
    setup(&mut app);

    // Assert
    let world = app.world_mut();
    let mut query = world.query::<(&Shape, &RenderLayer)>();
    let polygon_count = query
        .iter(world)
        .filter(|(s, layer)| {
            **layer == RenderLayer::Background && matches!(s.variant, ShapeVariant::Polygon { .. })
        })
        .count();
    assert!(
        polygon_count > 0,
        "expected at least one polygon on background layer"
    );
}
