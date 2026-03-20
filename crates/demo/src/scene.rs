use axiom2d::prelude::*;

use crate::types::{CelestialDef, Moon, MoonDef, OrbitalSpeed, SUN_COLOR, SUN_POSITION, Sun};

pub(crate) fn planets() -> [CelestialDef; 4] {
    [
        CelestialDef {
            orbit_radius: 120.0,
            speed: 1.5,
            color: Color::from_u8(180, 120, 60, 255),
            size: 20.0,
            moon: None,
        },
        CelestialDef {
            orbit_radius: 200.0,
            speed: 1.0,
            color: Color::from_u8(60, 130, 200, 255),
            size: 30.0,
            moon: Some(MoonDef {
                orbit_radius: 40.0,
                speed: 3.0,
                color: Color::from_u8(200, 200, 200, 255),
                size: 8.0,
            }),
        },
        CelestialDef {
            orbit_radius: 300.0,
            speed: 0.6,
            color: Color::from_u8(50, 180, 80, 255),
            size: 35.0,
            moon: Some(MoonDef {
                orbit_radius: 50.0,
                speed: 2.0,
                color: Color::from_u8(180, 160, 140, 255),
                size: 10.0,
            }),
        },
        CelestialDef {
            orbit_radius: 420.0,
            speed: 0.35,
            color: Color::RED,
            size: 25.0,
            moon: None,
        },
    ]
}

fn nebula_circles() -> [(Vec2, f32, Color); 4] {
    [
        (
            Vec2::new(-200.0, 150.0),
            180.0,
            Color::from_u8(30, 10, 60, 40),
        ),
        (
            Vec2::new(250.0, -100.0),
            220.0,
            Color::from_u8(10, 20, 50, 35),
        ),
        (
            Vec2::new(0.0, -250.0),
            160.0,
            Color::from_u8(40, 15, 45, 30),
        ),
        (
            Vec2::new(350.0, 200.0),
            140.0,
            Color::from_u8(20, 10, 55, 25),
        ),
    ]
}

fn nebula_polygons() -> [(Vec2, Vec<Vec2>, Color); 2] {
    [
        (
            Vec2::new(-150.0, 50.0),
            vec![
                Vec2::new(-40.0, -20.0),
                Vec2::new(0.0, -35.0),
                Vec2::new(30.0, -10.0),
                Vec2::new(25.0, 20.0),
                Vec2::new(-10.0, 30.0),
                Vec2::new(-35.0, 10.0),
            ],
            Color::from_u8(50, 20, 80, 20),
        ),
        (
            Vec2::new(180.0, -200.0),
            vec![
                Vec2::new(-30.0, -25.0),
                Vec2::new(10.0, -40.0),
                Vec2::new(35.0, -5.0),
                Vec2::new(20.0, 30.0),
                Vec2::new(-20.0, 25.0),
            ],
            Color::from_u8(25, 15, 60, 25),
        ),
    ]
}

pub(crate) fn spawn_sun(world: &mut World) {
    world.spawn((
        Transform2D {
            position: SUN_POSITION,
            ..Transform2D::default()
        },
        Shape {
            variant: ShapeVariant::Circle { radius: 40.0 },
            color: SUN_COLOR,
        },
        Sun,
        RenderLayer::World,
    ));
}

pub(crate) fn spawn_planets(world: &mut World) {
    for planet_def in planets() {
        let pivot = world
            .spawn((
                Transform2D {
                    position: SUN_POSITION,
                    ..Transform2D::default()
                },
                OrbitalSpeed(planet_def.speed),
            ))
            .id();
        let planet = world.spawn_child(
            pivot,
            (
                Transform2D {
                    position: Vec2::new(planet_def.orbit_radius, 0.0),
                    ..Transform2D::default()
                },
                Shape {
                    variant: ShapeVariant::Circle {
                        radius: planet_def.size / 2.0,
                    },
                    color: planet_def.color,
                },
                RenderLayer::World,
            ),
        );
        if let Some(moon_def) = planet_def.moon {
            let moon_pivot = world.spawn_child(
                planet,
                (Transform2D::default(), OrbitalSpeed(moon_def.speed)),
            );
            world.spawn_child(
                moon_pivot,
                (
                    Transform2D {
                        position: Vec2::new(moon_def.orbit_radius, 0.0),
                        ..Transform2D::default()
                    },
                    Sprite {
                        texture: TextureId(0),
                        uv_rect: [0.0, 0.0, 1.0, 1.0],
                        color: moon_def.color,
                        width: Pixels(moon_def.size),
                        height: Pixels(moon_def.size),
                    },
                    Moon,
                    RenderLayer::World,
                ),
            );
        }
    }
}

pub(crate) fn spawn_nebula(world: &mut World) {
    let additive_material = Material2d {
        blend_mode: BlendMode::Additive,
        ..Material2d::default()
    };
    for (pos, radius, color) in nebula_circles() {
        world.spawn((
            Transform2D {
                position: pos,
                ..Transform2D::default()
            },
            Shape {
                variant: ShapeVariant::Circle { radius },
                color,
            },
            RenderLayer::Background,
            additive_material.clone(),
        ));
    }
    for (pos, points, color) in nebula_polygons() {
        world.spawn((
            Transform2D {
                position: pos,
                ..Transform2D::default()
            },
            Shape {
                variant: ShapeVariant::Polygon { points },
                color,
            },
            RenderLayer::Background,
            additive_material.clone(),
        ));
    }
}

pub(crate) fn spawn_camera(world: &mut World) {
    world.spawn(Camera2D {
        position: SUN_POSITION,
        zoom: 0.5,
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::setup;
    use crate::types::{Moon, OrbitalSpeed, SUN_COLOR, SUN_POSITION, Sun, action};

    const PLANET_COUNT: usize = 4;
    const MOON_COUNT: usize = 2;

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
            1 + PLANET_COUNT + MOON_COUNT
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
        assert_eq!(query.iter(world).count(), 1);
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
        assert!(!action_map.bindings_for(action::MOVE_RIGHT).is_empty());
        assert!(!action_map.bindings_for(action::MOVE_LEFT).is_empty());
        assert!(!action_map.bindings_for(action::MOVE_UP).is_empty());
        assert!(!action_map.bindings_for(action::MOVE_DOWN).is_empty());
        assert!(!action_map.bindings_for(action::ZOOM_IN).is_empty());
        assert!(!action_map.bindings_for(action::ZOOM_OUT).is_empty());
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
        assert_eq!(query.iter(world).count(), 1);
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
        assert_eq!(shape.color, SUN_COLOR);
    }

    #[test]
    fn when_setup_called_then_correct_number_of_planet_pivots() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let mut query = world.query_filtered::<&OrbitalSpeed, Without<ChildOf>>();
        assert_eq!(query.iter(world).count(), PLANET_COUNT);
    }

    #[test]
    fn when_setup_called_then_each_pivot_has_one_planet_child() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);

        // Act / Assert
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();
        assert_eq!(pivots.len(), PLANET_COUNT);
        for pivot in pivots {
            let children = world.get::<Children>(pivot).unwrap();
            assert_eq!(children.0.len(), 1);
            let child = children.0[0];
            assert!(world.get::<Shape>(child).is_some());
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
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();
        for pivot in pivots {
            let children = world.get::<Children>(pivot).unwrap();
            let layer = world.get::<RenderLayer>(children.0[0]).unwrap();
            assert_eq!(*layer, RenderLayer::World);
        }
    }

    #[test]
    fn when_setup_called_then_camera_centered_on_sun() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);

        // Act
        let world = app.world_mut();
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(world).unwrap();

        // Assert
        assert_eq!(camera.position, SUN_POSITION);
    }

    #[test]
    fn when_orbit_and_propagation_run_then_planet_position_on_circle() {
        // Arrange
        let mut world = World::new();
        let orbit_radius = 100.0;
        world.insert_resource(DeltaTime(Seconds(std::f32::consts::FRAC_PI_2)));
        let pivot = world
            .spawn((Transform2D::default(), OrbitalSpeed(1.0)))
            .id();
        world.spawn_child(
            pivot,
            Transform2D {
                position: Vec2::new(orbit_radius, 0.0),
                ..Transform2D::default()
            },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                crate::systems::orbit_system,
                hierarchy_maintenance_system,
                transform_propagation_system,
            )
                .chain(),
        );

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<(&GlobalTransform2D, &ChildOf)>();
        let (global, _) = query.single(&world).unwrap();
        let pos = global.0.translation;
        assert!(
            (pos - Vec2::new(0.0, orbit_radius)).length() < 1e-4,
            "expected planet near (0, {orbit_radius}), got ({}, {})",
            pos.x,
            pos.y
        );
    }

    #[test]
    fn when_setup_called_then_moons_exist_with_moon_marker() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let mut query = world.query::<&Moon>();
        assert!(query.iter(world).count() >= 1, "expected at least one moon");
    }

    #[test]
    fn when_setup_called_then_moons_are_grandchildren_of_orbit_pivots() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);
        schedule.run(world);

        // Act
        let mut moon_query = world.query::<(Entity, &Moon, &ChildOf)>();
        let moons: Vec<(Entity, Entity)> = moon_query
            .iter(world)
            .map(|(e, _, parent)| (e, parent.0))
            .collect();

        // Assert
        assert!(!moons.is_empty());
        for (_moon, moon_pivot) in &moons {
            let pivot_parent = world.get::<ChildOf>(*moon_pivot);
            assert!(
                pivot_parent.is_some(),
                "moon pivot should be a child of the planet"
            );
        }
    }

    #[test]
    fn when_setup_called_then_moon_pivots_have_orbital_speed() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();

        // Act
        let mut moon_query = world.query::<(&Moon, &ChildOf)>();
        let moon_pivots: Vec<Entity> = moon_query.iter(world).map(|(_, parent)| parent.0).collect();

        // Assert
        assert!(!moon_pivots.is_empty());
        for pivot in moon_pivots {
            assert!(
                world.get::<OrbitalSpeed>(pivot).is_some(),
                "moon pivot should have OrbitalSpeed"
            );
        }
    }

    #[test]
    fn when_setup_called_then_bloom_settings_exist() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        assert!(world.get_resource::<BloomSettings>().is_some());
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
        assert!(bloom.enabled);
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
        assert!(matches!(shape.variant, ShapeVariant::Circle { .. }));
    }

    #[test]
    fn when_planets_queried_then_each_has_shape() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);

        // Act
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();

        // Assert
        assert_eq!(pivots.len(), PLANET_COUNT);
        for pivot in pivots {
            let children = world.get::<Children>(pivot).unwrap();
            let planet = children.0[0];
            assert!(
                world.get::<Shape>(planet).is_some(),
                "planet child should have a Shape component"
            );
        }
    }

    #[test]
    fn when_planet_shapes_queried_then_distinct_colors() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);

        // Act
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();
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
        assert_eq!(unique.len(), PLANET_COUNT);
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
        assert_eq!(query.iter(world).count(), MOON_COUNT);
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
        assert_eq!(world_circle_count, 1 + PLANET_COUNT);
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
                assert_eq!(material.blend_mode, BlendMode::Additive);
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
                **layer == RenderLayer::Background
                    && matches!(s.variant, ShapeVariant::Polygon { .. })
            })
            .count();
        assert!(
            polygon_count > 0,
            "expected at least one polygon on background layer"
        );
    }
}
