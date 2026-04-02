use axiom2d::prelude::*;

use crate::types::{CelestialDef, Moon, MoonDef, OrbitalSpeed, SUN_COLOR, SUN_POSITION, Sun};

pub fn planets() -> [CelestialDef; 4] {
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

pub fn spawn_sun(world: &mut World) {
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

pub fn spawn_planets(world: &mut World) {
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

pub fn spawn_nebula(world: &mut World) {
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

pub fn spawn_camera(world: &mut World) {
    world.spawn(Camera2D {
        position: SUN_POSITION,
        zoom: 0.5,
    });
}
