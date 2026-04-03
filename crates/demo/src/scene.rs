use axiom2d::prelude::*;

use crate::types::{
    EARTH_COLOR, Earth, MOON_COLOR, Moon, OrbitalSpeed, SUN_COLOR, Sun, SynodicFrame,
};

#[derive(Clone, Copy)]
struct PlanetDef {
    orbit_radius: f32,
    angular_speed: f32,
    phase: f32,
    color: Color,
    size: f32,
    moon: Option<MoonDef>,
    is_earth: bool,
}

#[derive(Clone, Copy)]
struct MoonDef {
    orbit_radius: f32,
    angular_speed: f32,
    phase: f32,
    size: f32,
}

fn planets() -> [PlanetDef; 8] {
    [
        PlanetDef {
            orbit_radius: 90.0,
            angular_speed: 1.2,
            phase: 0.25,
            color: Color::from_u8(155, 155, 165, 255),
            size: 8.0,
            moon: None,
            is_earth: false,
        },
        PlanetDef {
            orbit_radius: 130.0,
            angular_speed: 0.85,
            phase: 1.1,
            color: Color::from_u8(220, 190, 120, 255),
            size: 13.0,
            moon: None,
            is_earth: false,
        },
        PlanetDef {
            orbit_radius: 180.0,
            angular_speed: 0.5,
            phase: 2.0,
            color: EARTH_COLOR,
            size: 20.0,
            moon: Some(MoonDef {
                orbit_radius: 40.0,
                angular_speed: 1.0,
                phase: 0.75,
                size: 8.0,
            }),
            is_earth: true,
        },
        PlanetDef {
            orbit_radius: 240.0,
            angular_speed: 0.42,
            phase: 2.8,
            color: Color::from_u8(210, 95, 75, 255),
            size: 11.0,
            moon: None,
            is_earth: false,
        },
        PlanetDef {
            orbit_radius: 300.0,
            angular_speed: 0.22,
            phase: 3.5,
            color: Color::from_u8(205, 145, 85, 255),
            size: 28.0,
            moon: None,
            is_earth: false,
        },
        PlanetDef {
            orbit_radius: 380.0,
            angular_speed: 0.16,
            phase: 4.2,
            color: Color::from_u8(235, 215, 150, 255),
            size: 24.0,
            moon: None,
            is_earth: false,
        },
        PlanetDef {
            orbit_radius: 460.0,
            angular_speed: 0.12,
            phase: 4.9,
            color: Color::from_u8(120, 205, 220, 255),
            size: 18.0,
            moon: None,
            is_earth: false,
        },
        PlanetDef {
            orbit_radius: 540.0,
            angular_speed: 0.09,
            phase: 5.6,
            color: Color::from_u8(85, 120, 230, 255),
            size: 18.0,
            moon: None,
            is_earth: false,
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

pub fn spawn_synodic_frame(world: &mut World) -> Entity {
    world.spawn((Transform2D::default(), SynodicFrame)).id()
}

pub fn spawn_sun(world: &mut World, parent: Entity) {
    world.spawn_child(
        parent,
        (
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Circle { radius: 40.0 },
                color: SUN_COLOR,
            },
            Sun,
            RenderLayer::World,
        ),
    );
}

fn spawn_planet(world: &mut World, parent: Entity, planet_def: PlanetDef) {
    let pivot = world.spawn_child(
        parent,
        (
            Transform2D {
                rotation: planet_def.phase,
                ..Transform2D::default()
            },
            OrbitalSpeed(planet_def.angular_speed),
        ),
    );
    let planet = if planet_def.is_earth {
        world.spawn_child(
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
                Earth,
                RenderLayer::World,
            ),
        )
    } else {
        world.spawn_child(
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
        )
    };

    if let Some(moon_def) = planet_def.moon {
        let moon_pivot = world.spawn_child(
            planet,
            (
                Transform2D {
                    rotation: moon_def.phase,
                    ..Transform2D::default()
                },
                OrbitalSpeed(moon_def.angular_speed),
            ),
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
                    color: MOON_COLOR,
                    width: Pixels(moon_def.size),
                    height: Pixels(moon_def.size),
                },
                Moon,
                RenderLayer::World,
            ),
        );
    }
}

pub fn spawn_planets(world: &mut World, parent: Entity) {
    for planet_def in planets() {
        spawn_planet(world, parent, planet_def);
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
    world.spawn((
        Camera2D {
            position: Vec2::ZERO,
            zoom: 0.5,
        },
        CameraRotation::default(),
    ));
}
