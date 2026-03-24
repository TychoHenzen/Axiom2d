use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;

/// Element: Solidum
/// Aspect: Solid
/// Signature: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
pub fn armor1() -> Vec<Shape> {
    vec![
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-98.5, 87.5)),
                    PathCommand::LineTo(Vec2::new(82.5, 68.5)),
                    PathCommand::LineTo(Vec2::new(82.5, 36.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(76.24864, 11.494579),
                        control2: Vec2::new(50.574776, -5.4252243),
                        to: Vec2::new(32.5, -23.5),
                    },
                    PathCommand::LineTo(Vec2::new(-29.5, -27.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-71.69275, -22.354544),
                        control2: Vec2::new(-59.044403, -30.137064),
                        to: Vec2::new(-73.5, -20.5),
                    },
                    PathCommand::LineTo(Vec2::new(-88.5, 11.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-96.81583, 44.763317),
                        control2: Vec2::new(-93.72713, 30.044672),
                        to: Vec2::new(-98.5, 55.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23555121, 0.21347854, 0.3467792, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-104.5, 96.5)),
                    PathCommand::LineTo(Vec2::new(87.5, 80.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(98.4584, 79.1302),
                        control2: Vec2::new(93.096306, 80.70184),
                        to: Vec2::new(103.5, 75.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(102.512115, 58.705914),
                        control2: Vec2::new(100.28216, 42.889343),
                        to: Vec2::new(96.5, 26.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(89.419075, 4.0770664),
                        control2: Vec2::new(92.96649, 13.666225),
                        to: Vec2::new(86.5, -2.5),
                    },
                    PathCommand::LineTo(Vec2::new(76.5, -22.5)),
                    PathCommand::LineTo(Vec2::new(63.5, -42.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(43.654434, -66.59818),
                        control2: Vec2::new(54.822144, -54.443962),
                        to: Vec2::new(29.5, -78.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-1.575119, -101.28842),
                        control2: Vec2::new(11.900707, -100.5),
                        to: Vec2::new(-4.5, -100.5),
                    },
                    PathCommand::LineTo(Vec2::new(-34.5, -76.5)),
                    PathCommand::LineTo(Vec2::new(-55.5, -55.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-74.81027, -29.374344),
                        control2: Vec2::new(-67.59513, -40.67478),
                        to: Vec2::new(-78.5, -22.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-89.37894, -0.7421341),
                        control2: Vec2::new(-84.33888, -12.057169),
                        to: Vec2::new(-93.5, 11.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-103.53554, 51.642147),
                        control2: Vec2::new(-100.19004, 32.583664),
                        to: Vec2::new(-104.5, 68.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.18116479, 0.16808034, 0.23536284, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-115.5, 121.5)),
                    PathCommand::LineTo(Vec2::new(46.5, 91.5)),
                    PathCommand::LineTo(Vec2::new(25.5, 84.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-9.119801, 80.65336),
                        control2: Vec2::new(7.5554085, 81.5),
                        to: Vec2::new(-24.5, 81.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-72.392624, 84.7654),
                        control2: Vec2::new(-54.684364, 79.476135),
                        to: Vec2::new(-79.5, 88.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-92.72491, 95.11245),
                        control2: Vec2::new(-86.399124, 91.43275),
                        to: Vec2::new(-98.5, 99.5),
                    },
                    PathCommand::LineTo(Vec2::new(-112.5, 111.5)),
                    PathCommand::LineTo(Vec2::new(-115.5, 120.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7052679, 0.50283647, 0.25217187, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(0.5, -109.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(6.571213, -67.00151),
                        control2: Vec2::new(54.56529, -42.665176),
                        to: Vec2::new(90.5, -23.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(85.16671, -36.83323),
                        control2: Vec2::new(89.432945, -26.967304),
                        to: Vec2::new(74.5, -51.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(61.63042, -69.19567),
                        control2: Vec2::new(69.419716, -59.032833),
                        to: Vec2::new(50.5, -81.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(25.259228, -101.426926),
                        control2: Vec2::new(38.265503, -92.102425),
                        to: Vec2::new(11.5, -109.5),
                    },
                    PathCommand::LineTo(Vec2::new(0.5, -114.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.43563366, 0.33529556, 0.5057598, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-113.5, 39.5)),
                    PathCommand::LineTo(Vec2::new(-1.5, -104.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-1.4032352, -105.564415),
                        control2: Vec2::new(1.2087545, -117.791245),
                        to: Vec2::new(-1.5, -120.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-67.87663, -60.546913),
                        control2: Vec2::new(-44.61074, -84.96336),
                        to: Vec2::new(-75.5, -51.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-93.53114, -21.877413),
                        control2: Vec2::new(-85.64128, -36.60331),
                        to: Vec2::new(-99.5, -7.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-110.82764, 23.965647),
                        control2: Vec2::new(-106.35188, 8.907518),
                        to: Vec2::new(-113.5, 37.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6877908, 0.46955734, 0.26710322, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-47.5, -32.5)),
                    PathCommand::LineTo(Vec2::new(59.5, 11.5)),
                    PathCommand::LineTo(Vec2::new(59.5, 2.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(55.74776, -15.010454),
                        control2: Vec2::new(58.272514, -7.068719),
                        to: Vec2::new(52.5, -21.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(44.131905, -34.05214),
                        control2: Vec2::new(48.752655, -27.68418),
                        to: Vec2::new(38.5, -40.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(29.14885, -46.7341),
                        control2: Vec2::new(34.688114, -43.5054),
                        to: Vec2::new(21.5, -49.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.0152645, -54.962597),
                        control2: Vec2::new(11.665726, -53.196133),
                        to: Vec2::new(-10.5, -54.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-24.850159, -49.374943),
                        control2: Vec2::new(-36.501556, -44.498444),
                        to: Vec2::new(-47.5, -33.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9804942, 0.97984827, 0.99978375, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-19.5, 65.5)),
                    PathCommand::LineTo(Vec2::new(101.5, 94.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(112.05702, 62.82894),
                        control2: Vec2::new(69.99853, 46.148674),
                        to: Vec2::new(51.5, 29.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33918387, 0.29262298, 0.49500984, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-28.5, 78.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(4.2660065, 111.26601),
                        control2: Vec2::new(60.771446, 103.39302),
                        to: Vec2::new(105.5, 115.5),
                    },
                    PathCommand::LineTo(Vec2::new(100.5, 108.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(74.84261, 88.87965),
                        control2: Vec2::new(90.04559, 98.336426),
                        to: Vec2::new(53.5, 82.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(25.017845, 74.36224),
                        control2: Vec2::new(39.672264, 77.74735),
                        to: Vec2::new(9.5, 72.5),
                    },
                    PathCommand::LineTo(Vec2::new(-11.5, 72.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8142867, 0.7005127, 0.3058407, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-67.5, -23.5)),
                    PathCommand::LineTo(Vec2::new(-66.5, -0.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(61.435493, 16.625223),
                        control2: Vec2::new(30.298801, 46.7012),
                        to: Vec2::new(62.5, 14.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(68.307915, -2.9237366),
                        control2: Vec2::new(64.32489, 10.601349),
                        to: Vec2::new(67.5, -27.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(12.65797, -86.56065),
                        control2: Vec2::new(37.862892, -71.302055),
                        to: Vec2::new(4.5, -89.5),
                    },
                    PathCommand::LineTo(Vec2::new(-3.5, -89.5)),
                    PathCommand::LineTo(Vec2::new(-50.5, -45.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.32407084, 0.28134507, 0.46346346, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-117.5, 63.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-74.343475, 2.3615952),
                        control2: Vec2::new(-109.14284, 52.414246),
                        to: Vec2::new(-20.5, -91.5),
                    },
                    PathCommand::LineTo(Vec2::new(-24.5, -90.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-45.172142, -74.2576),
                        control2: Vec2::new(-34.303413, -83.69659),
                        to: Vec2::new(-56.5, -61.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-69.10466, -46.374405),
                        control2: Vec2::new(-63.51418, -53.765457),
                        to: Vec2::new(-73.5, -39.5),
                    },
                    PathCommand::LineTo(Vec2::new(-103.5, 17.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-116.94974, 48.067596),
                        control2: Vec2::new(-113.00326, 33.519566),
                        to: Vec2::new(-117.5, 60.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.75127614, 0.5819182, 0.29225698, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-67.5, -61.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-35.073414, -85.17656),
                        control2: Vec2::new(-2.5, -86.72708),
                        to: Vec2::new(-2.5, -120.5),
                    },
                    PathCommand::LineTo(Vec2::new(-12.5, -115.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-26.702038, -105.35569),
                        control2: Vec2::new(-39.194206, -93.80579),
                        to: Vec2::new(-51.5, -81.5),
                    },
                    PathCommand::LineTo(Vec2::new(-67.5, -62.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5616191, 0.3941358, 0.24042934, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(99.5, 22.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(110.9552, 89.148445),
                        control2: Vec2::new(91.70589, 75.760506),
                        to: Vec2::new(117.5, 90.5),
                    },
                    PathCommand::LineTo(Vec2::new(117.5, 70.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(115.75379, 54.784115),
                        control2: Vec2::new(112.87566, 40.02805),
                        to: Vec2::new(109.5, 24.5),
                    },
                    PathCommand::LineTo(Vec2::new(99.5, 19.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.43327963, 0.32370135, 0.28063178, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-66.5, 85.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-19.31622, 97.295944),
                        control2: Vec2::new(-63.851974, 86.37802),
                        to: Vec2::new(70.5, 98.5),
                    },
                    PathCommand::LineTo(Vec2::new(49.5, 87.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(26.619696, 79.87323),
                        control2: Vec2::new(4.8254414, 77.89425),
                        to: Vec2::new(-19.5, 74.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-56.042152, 79.21512),
                        control2: Vec2::new(-40.555054, 75.392204),
                        to: Vec2::new(-66.5, 83.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.74601865, 0.5819916, 0.24990974, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-28.5, -67.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.362038, -55.362038),
                        control2: Vec2::new(-27.442791, -65.47807),
                        to: Vec2::new(16.5, -66.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(16.5, -77.699585),
                        control2: Vec2::new(3.4059768, -84.468925),
                        to: Vec2::new(-4.5, -89.5),
                    },
                    PathCommand::LineTo(Vec2::new(-14.5, -85.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-19.40708, -81.41077),
                        control2: Vec2::new(-25.588684, -77.32263),
                        to: Vec2::new(-28.5, -71.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23500161, 0.21092683, 0.3428197, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(76.5, -30.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(100.0922, 22.046272),
                        control2: Vec2::new(83.147385, 9.28843),
                        to: Vec2::new(108.5, 24.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(106.09402, 7.658121),
                        control2: Vec2::new(98.31522, -7.0108624),
                        to: Vec2::new(91.5, -22.5),
                    },
                    PathCommand::LineTo(Vec2::new(76.5, -31.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4399163, 0.33303753, 0.39690787, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-119.5, 98.5)),
                    PathCommand::LineTo(Vec2::new(-118.5, 106.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-108.51708, 91.52562),
                        control2: Vec2::new(-116.7784, 104.66928),
                        to: Vec2::new(-106.5, 61.5),
                    },
                    PathCommand::LineTo(Vec2::new(-104.5, 44.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-111.833336, 44.5),
                        control2: Vec2::new(-107.69999, 43.433323),
                        to: Vec2::new(-114.5, 52.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-119.8758, 76.69112),
                        control2: Vec2::new(-118.17695, 64.362076),
                        to: Vec2::new(-119.5, 89.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.80120933, 0.6702945, 0.32798478, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-38.5, 21.5)),
                    PathCommand::LineTo(Vec2::new(-1.5, 52.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(20.42181, 14.745773),
                        control2: Vec2::new(25.67945, 29.149542),
                        to: Vec2::new(-1.5, 6.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.98856515, 0.9896353, 0.9997997, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-63.5, 1.5)),
                    PathCommand::LineTo(Vec2::new(-11.5, -53.5)),
                    PathCommand::LineTo(Vec2::new(-14.5, -57.5)),
                    PathCommand::LineTo(Vec2::new(-21.5, -57.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-31.788418, -54.927895),
                        control2: Vec2::new(-38.008427, -49.03198),
                        to: Vec2::new(-46.5, -42.5),
                    },
                    PathCommand::LineTo(Vec2::new(-57.5, -28.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-62.251797, -17.412472),
                        control2: Vec2::new(-59.929222, -23.973373),
                        to: Vec2::new(-63.5, -8.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.52073854, 0.5308183, 0.6652141, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(15.5, -77.5)),
                    PathCommand::LineTo(Vec2::new(23.5, -64.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(61.148643, -29.988743),
                        control2: Vec2::new(44.939705, -38.780148),
                        to: Vec2::new(65.5, -28.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(65.5, -40.01274),
                        control2: Vec2::new(31.180882, -70.81912),
                        to: Vec2::new(26.5, -75.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(18.865782, -81.22566),
                        control2: Vec2::new(19.473755, -84.46063),
                        to: Vec2::new(15.5, -78.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2275924, 0.20510699, 0.3316825, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-96.5, 88.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-95.20185, 91.0963),
                        control2: Vec2::new(-96.24536, 90.5),
                        to: Vec2::new(-93.5, 90.5),
                    },
                    PathCommand::LineTo(Vec2::new(-33.5, 73.5)),
                    PathCommand::LineTo(Vec2::new(-39.5, 68.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-55.797653, 68.5),
                        control2: Vec2::new(-44.240906, 67.91431),
                        to: Vec2::new(-73.5, 75.5),
                    },
                    PathCommand::LineTo(Vec2::new(-88.5, 83.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.36778018, 0.29953134, 0.5599483, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-57.5, 75.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-54.833332, 75.5),
                        control2: Vec2::new(-55.5, 76.166664),
                        to: Vec2::new(-55.5, 73.5),
                    },
                    PathCommand::LineTo(Vec2::new(-57.5, 73.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.36778018, 0.29953134, 0.5599483, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-18.5, -32.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-4.906969, -18.90697),
                        control2: Vec2::new(11.291388, -24.32525),
                        to: Vec2::new(29.5, -25.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(23.71116, -29.841629),
                        control2: Vec2::new(26.990688, -27.754656),
                        to: Vec2::new(19.5, -31.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(8.133981, -34.341503),
                        control2: Vec2::new(-1.8509073, -34.470757),
                        to: Vec2::new(-13.5, -33.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23529409, 0.21336584, 0.34438008, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(20.5, 69.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(48.126793, 90.22009),
                        control2: Vec2::new(22.937103, 71.91627),
                        to: Vec2::new(107.5, 103.5),
                    },
                    PathCommand::LineTo(Vec2::new(108.5, 95.5)),
                    PathCommand::LineTo(Vec2::new(60.5, 76.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(39.94026, 71.0174),
                        control2: Vec2::new(50.595783, 73.399254),
                        to: Vec2::new(28.5, 69.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.32262713, 0.22805731, 0.22665147, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-29.5, -55.5)),
                    PathCommand::LineTo(Vec2::new(54.5, -20.5)),
                    PathCommand::LineTo(Vec2::new(49.5, -31.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(41.534508, -43.448242),
                        control2: Vec2::new(45.874252, -37.78657),
                        to: Vec2::new(36.5, -48.5),
                    },
                    PathCommand::LineTo(Vec2::new(17.5, -58.5)),
                    PathCommand::LineTo(Vec2::new(7.5, -61.5)),
                    PathCommand::LineTo(Vec2::new(-7.5, -62.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-22.032148, -59.85779),
                        control2: Vec2::new(-15.039823, -61.88407),
                        to: Vec2::new(-28.5, -56.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.38913932, 0.38454387, 0.5071497, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-42.5, 70.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-27.034912, 75.13953),
                        control2: Vec2::new(-35.956215, 73.09338),
                        to: Vec2::new(-15.5, 75.5),
                    },
                    PathCommand::LineTo(Vec2::new(15.5, 71.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-2.4851513, 62.507423),
                        control2: Vec2::new(11.724808, 68.72248),
                        to: Vec2::new(-30.5, 64.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33649334, 0.25886524, 0.39914134, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(10.5, 72.5)),
                    PathCommand::LineTo(Vec2::new(86.5, 96.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(78.2941, 88.2941),
                        control2: Vec2::new(83.40621, 92.88634),
                        to: Vec2::new(70.5, 83.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(52.97669, 77.6589),
                        control2: Vec2::new(62.60508, 80.465904),
                        to: Vec2::new(41.5, 75.5),
                    },
                    PathCommand::LineTo(Vec2::new(13.5, 72.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8742736, 0.80395323, 0.36778116, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(75.5, 88.5)),
                    PathCommand::LineTo(Vec2::new(105.5, 117.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(115.05474, 123.86983),
                        control2: Vec2::new(110.894424, 123.5),
                        to: Vec2::new(116.5, 123.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(105.84145, 103.24876),
                        control2: Vec2::new(111.56071, 109.56071),
                        to: Vec2::new(103.5, 101.5),
                    },
                    PathCommand::LineTo(Vec2::new(93.5, 94.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(76.90304, 86.201515),
                        control2: Vec2::new(83.46294, 86.5),
                        to: Vec2::new(75.5, 86.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.92258465, 0.88389206, 0.4167483, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-11.5, -98.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(45.787075, -64.328766),
                        control2: Vec2::new(23.376633, -64.5),
                        to: Vec2::new(46.5, -64.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(56.729588, -84.959175),
                        control2: Vec2::new(15.674398, -98.3256),
                        to: Vec2::new(-0.5, -114.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.79094344, 0.67005366, 0.29408512, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(63.5, 62.5)),
                    PathCommand::LineTo(Vec2::new(65.5, 69.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(70.714386, 74.714386),
                        control2: Vec2::new(76.31802, 77.5),
                        to: Vec2::new(83.5, 77.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(83.5, 70.3826),
                        control2: Vec2::new(77.97745, 66.53713),
                        to: Vec2::new(73.5, 61.5),
                    },
                    PathCommand::LineTo(Vec2::new(64.5, 55.5)),
                    PathCommand::LineTo(Vec2::new(63.5, 58.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.25787175, 0.23107743, 0.38513494, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-62.5, 10.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-57.05883, 1.5220737),
                        control2: Vec2::new(-33.273415, -18.273415),
                        to: Vec2::new(-46.5, -31.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-56.74568, -21.254318),
                        control2: Vec2::new(-58.20022, -9.627859),
                        to: Vec2::new(-62.5, 4.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6621968, 0.6637089, 0.76784813, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-1.5, -122.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(4.1263833, -102.80766),
                        control2: Vec2::new(18.564709, -86.5),
                        to: Vec2::new(45.5, -86.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(31.812542, -100.18746),
                        control2: Vec2::new(17.36802, -112.86113),
                        to: Vec2::new(0.5, -122.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41768992, 0.3270892, 0.5495071, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(27.5, -55.5)),
                    PathCommand::LineTo(Vec2::new(60.5, 6.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(63.741035, 0.0179286),
                        control2: Vec2::new(61.549583, 5.2437725),
                        to: Vec2::new(60.5, -10.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(57.37394, -23.004236),
                        control2: Vec2::new(59.486774, -15.934384),
                        to: Vec2::new(53.5, -31.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(49.256855, -38.289032),
                        control2: Vec2::new(45.138893, -43.861107),
                        to: Vec2::new(39.5, -49.5),
                    },
                    PathCommand::LineTo(Vec2::new(28.5, -57.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4455219, 0.3877178, 0.5788194, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(5.5, 7.5)),
                    PathCommand::LineTo(Vec2::new(6.5, 11.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(12.200863, 17.200863),
                        control2: Vec2::new(21.23627, 16.5),
                        to: Vec2::new(28.5, 16.5),
                    },
                    PathCommand::LineTo(Vec2::new(41.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(38.435043, 0.8376117),
                        control2: Vec2::new(40.543392, 4.9607506),
                        to: Vec2::new(34.5, -3.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(27.98785, -8.92679),
                        control2: Vec2::new(31.103418, -8.5),
                        to: Vec2::new(26.5, -8.5),
                    },
                    PathCommand::LineTo(Vec2::new(14.5, -2.5)),
                    PathCommand::LineTo(Vec2::new(5.5, 5.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.31063646, 0.28507677, 0.44445652, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-41.5, -49.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-18.406479, -51.173443),
                        control2: Vec2::new(27.5, -34.345924),
                        to: Vec2::new(27.5, -57.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(23.075905, -61.924095),
                        control2: Vec2::new(15.308201, -63.04795),
                        to: Vec2::new(9.5, -64.5),
                    },
                    PathCommand::LineTo(Vec2::new(-4.5, -65.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.998856, -63.93764),
                        control2: Vec2::new(-9.931355, -65.29035),
                        to: Vec2::new(-25.5, -60.5),
                    },
                    PathCommand::LineTo(Vec2::new(-37.5, -53.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.48903084, 0.39483997, 0.63668114, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(109.5, 105.5)),
                    PathCommand::LineTo(Vec2::new(116.5, 120.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(120.24069, 109.277916),
                        control2: Vec2::new(117.5, 118.555916),
                        to: Vec2::new(117.5, 91.5),
                    },
                    PathCommand::LineTo(Vec2::new(113.5, 88.5)),
                    PathCommand::LineTo(Vec2::new(110.5, 87.5)),
                    PathCommand::LineTo(Vec2::new(109.5, 103.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.44341806, 0.33038262, 0.28056, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-31.5, 59.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-24.117325, 64.42178),
                        control2: Vec2::new(-11.655701, 62.5),
                        to: Vec2::new(-3.5, 62.5),
                    },
                    PathCommand::LineTo(Vec2::new(-6.5, 50.5)),
                    PathCommand::LineTo(Vec2::new(-8.5, 47.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.80093, 44.733025),
                        control2: Vec2::new(-10.493963, 46.185616),
                        to: Vec2::new(-27.5, 51.5),
                    },
                    PathCommand::LineTo(Vec2::new(-31.5, 57.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.31770483, 0.27747986, 0.4883519, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-118.5, 114.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-114.56399, 114.5),
                        control2: Vec2::new(-115.12998, 112.57496),
                        to: Vec2::new(-113.5, 108.5),
                    },
                    PathCommand::LineTo(Vec2::new(-108.5, 85.5)),
                    PathCommand::LineTo(Vec2::new(-107.5, 64.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-114.85, 86.549995),
                        control2: Vec2::new(-110.57125, 72.38698),
                        to: Vec2::new(-118.5, 107.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.85637355, 0.76457036, 0.37194142, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-47.5, 67.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-21.163387, 70.13366),
                        control2: Vec2::new(-40.83263, 68.840065),
                        to: Vec2::new(11.5, 63.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-13.561565, 49.830055),
                        control2: Vec2::new(4.585251, 58.3692),
                        to: Vec2::new(-46.5, 47.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5100666, 0.42623883, 0.63662744, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-29.5, 20.5)),
                    PathCommand::LineTo(Vec2::new(-3.5, 19.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-1.9887003, -0.14689636),
                        control2: Vec2::new(-2.5, 10.846035),
                        to: Vec2::new(-2.5, -13.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7137658, 0.7257556, 0.88204783, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-75.5, -29.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-43.207928, -29.5),
                        control2: Vec2::new(-29.333942, -74.66606),
                        to: Vec2::new(-6.5, -97.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-8.245348, -99.245346),
                        control2: Vec2::new(-7.052801, -98.64906),
                        to: Vec2::new(-10.5, -97.5),
                    },
                    PathCommand::LineTo(Vec2::new(-24.5, -87.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-39.689808, -74.84183),
                        control2: Vec2::new(-53.140194, -60.949757),
                        to: Vec2::new(-65.5, -45.5),
                    },
                    PathCommand::LineTo(Vec2::new(-75.5, -30.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2984517, 0.20449995, 0.13696606, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(46.5, 62.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(52.705887, 68.70589),
                        control2: Vec2::new(48.14836, 65.084114),
                        to: Vec2::new(62.5, 69.5),
                    },
                    PathCommand::LineTo(Vec2::new(61.5, 50.5)),
                    PathCommand::LineTo(Vec2::new(59.5, 48.5)),
                    PathCommand::LineTo(Vec2::new(47.5, 60.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2570504, 0.23131017, 0.3843355, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-27.5, -28.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-11.287554, -3.0232964),
                        control2: Vec2::new(-24.805756, -16.909014),
                        to: Vec2::new(30.5, -24.5),
                    },
                    PathCommand::LineTo(Vec2::new(-22.5, -30.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.36641845, 0.31561843, 0.51782715, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-77.5, 70.5)),
                    PathCommand::LineTo(Vec2::new(-75.5, 70.5)),
                    PathCommand::LineTo(Vec2::new(-45.5, 57.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-45.5, 46.38289),
                        control2: Vec2::new(-45.171658, 55.443985),
                        to: Vec2::new(-53.5, 31.5),
                    },
                    PathCommand::LineTo(Vec2::new(-77.5, 69.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.30657798, 0.26917255, 0.48003274, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-59.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(10.786591, -34.67605),
                        control2: Vec2::new(10.5, -7.1158504),
                        to: Vec2::new(10.5, -35.5),
                    },
                    PathCommand::LineTo(Vec2::new(-11.5, -35.5)),
                    PathCommand::LineTo(Vec2::new(-25.5, -32.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-37.517387, -24.989132),
                        control2: Vec2::new(-31.066236, -29.49126),
                        to: Vec2::new(-44.5, -18.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-54.58243, -4.096528),
                        control2: Vec2::new(-49.481102, -12.03307),
                        to: Vec2::new(-59.5, 5.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.48963597, 0.42060688, 0.6599189, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(102.5, 44.5)),
                    PathCommand::LineTo(Vec2::new(104.5, 94.5)),
                    PathCommand::LineTo(Vec2::new(108.5, 94.5)),
                    PathCommand::LineTo(Vec2::new(106.5, 60.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(104.43772, 45.032925),
                        control2: Vec2::new(108.06682, 49.06682),
                        to: Vec2::new(103.5, 44.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41537294, 0.31521535, 0.19503276, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-22.5, 58.5)),
                    PathCommand::LineTo(Vec2::new(-4.5, 61.5)),
                    PathCommand::LineTo(Vec2::new(-4.5, 57.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-7.233905, 52.032192),
                        control2: Vec2::new(-15.361164, 51.641964),
                        to: Vec2::new(-20.5, 50.5),
                    },
                    PathCommand::LineTo(Vec2::new(-22.5, 54.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.28235272, 0.25053295, 0.4270829, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(46.5, -63.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(71.11781, -32.46015),
                        control2: Vec2::new(60.021786, -42.39851),
                        to: Vec2::new(75.5, -29.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(71.36744, -41.89767),
                        control2: Vec2::new(58.87822, -53.72541),
                        to: Vec2::new(50.5, -63.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(48.264297, -65.7357),
                        control2: Vec2::new(49.442802, -65.5),
                        to: Vec2::new(47.5, -65.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.741724, 0.62270606, 0.2761275, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(11.5, 29.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(27.701708, 50.76474),
                        control2: Vec2::new(18.699757, 50.5),
                        to: Vec2::new(28.5, 50.5),
                    },
                    PathCommand::LineTo(Vec2::new(42.5, 21.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(31.058315, 15.779158),
                        control2: Vec2::new(26.746386, 22.782606),
                        to: Vec2::new(11.5, 28.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.311594, 0.2860754, 0.43873298, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-13.5, -57.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-3.355033, -51.41302),
                        control2: Vec2::new(-10.44368, -54.5),
                        to: Vec2::new(9.5, -54.5),
                    },
                    PathCommand::LineTo(Vec2::new(6.5, -59.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-0.5608883, -61.26522),
                        control2: Vec2::new(-6.7764354, -61.189426),
                        to: Vec2::new(-13.5, -58.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.52422214, 0.5322716, 0.66946745, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-30.5, -40.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-26.389652, -29.53907),
                        control2: Vec2::new(-30.201853, -31.599382),
                        to: Vec2::new(-18.5, -35.5),
                    },
                    PathCommand::LineTo(Vec2::new(-17.5, -36.5)),
                    PathCommand::LineTo(Vec2::new(-22.5, -47.5)),
                    PathCommand::LineTo(Vec2::new(-30.5, -43.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8868187, 0.8864868, 0.9822325, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-118.5, 117.5)),
                    PathCommand::LineTo(Vec2::new(-117.5, 123.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-107.26727, 123.5),
                        control2: Vec2::new(-115.85066, 124.06646),
                        to: Vec2::new(-102.5, 102.5),
                    },
                    PathCommand::LineTo(Vec2::new(-108.5, 86.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-116.43217, 94.43217),
                        control2: Vec2::new(-109.35611, 87.05233),
                        to: Vec2::new(-118.5, 115.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9430445, 0.91658956, 0.44961137, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(37.5, -23.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(51.263397, -11.854049),
                        control2: Vec2::new(44.823532, -12.5),
                        to: Vec2::new(53.5, -12.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(53.5, -19.554276),
                        control2: Vec2::new(47.559086, -24.426144),
                        to: Vec2::new(43.5, -29.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(39.81345, -27.656725),
                        control2: Vec2::new(42.027046, -29.027046),
                        to: Vec2::new(37.5, -24.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8991069, 0.8802831, 0.96043974, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 10.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(5.020239, 13.260119),
                        control2: Vec2::new(2.03823, 12.207646),
                        to: Vec2::new(8.5, 13.5),
                    },
                    PathCommand::LineTo(Vec2::new(-0.5, -14.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5693044, 0.5797143, 0.7275934, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-38.5, -35.5)),
                    PathCommand::LineTo(Vec2::new(-35.5, -28.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-29.624655, -30.850138),
                        control2: Vec2::new(-31.769283, -29.230717),
                        to: Vec2::new(-28.5, -32.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-28.5, -38.038597),
                        control2: Vec2::new(-28.106083, -34.58086),
                        to: Vec2::new(-31.5, -42.5),
                    },
                    PathCommand::LineTo(Vec2::new(-38.5, -38.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.82345504, 0.82158756, 0.92145723, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(11.5, -34.5)),
                    PathCommand::LineTo(Vec2::new(57.5, 14.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(57.5, 5.282503),
                        control2: Vec2::new(58.0963, 11.692594),
                        to: Vec2::new(50.5, -3.5),
                    },
                    PathCommand::LineTo(Vec2::new(43.5, -14.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(30.80033, -27.19967),
                        control2: Vec2::new(36.814552, -22.62363),
                        to: Vec2::new(26.5, -29.5),
                    },
                    PathCommand::LineTo(Vec2::new(14.5, -34.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4788505, 0.41157454, 0.64683074, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-107.5, 101.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-105.97295, 103.02705),
                        control2: Vec2::new(-106.9216, 102.7108),
                        to: Vec2::new(-104.5, 101.5),
                    },
                    PathCommand::LineTo(Vec2::new(-92.5, 93.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-95.98103, 86.53793),
                        control2: Vec2::new(-92.873184, 91.902954),
                        to: Vec2::new(-107.5, 82.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4459828, 0.3485199, 0.7774703, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(88.5, 81.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(94.52762, 87.52762),
                        control2: Vec2::new(90.96558, 84.40399),
                        to: Vec2::new(99.5, 90.5),
                    },
                    PathCommand::LineTo(Vec2::new(99.5, 75.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(88.786095, 78.71417),
                        control2: Vec2::new(91.05634, 75.38731),
                        to: Vec2::new(88.5, 80.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23529427, 0.21295312, 0.34581116, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(91.5, 9.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(98.11332, 31.54441),
                        control2: Vec2::new(93.151985, 24.064978),
                        to: Vec2::new(101.5, 34.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(107.98617, 28.01383),
                        control2: Vec2::new(99.26787, 17.035736),
                        to: Vec2::new(95.5, 9.5),
                    },
                    PathCommand::LineTo(Vec2::new(91.5, 7.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.53651196, 0.40169275, 0.29907134, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-45.5, -31.5)),
                    PathCommand::LineTo(Vec2::new(-42.5, -23.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-38.365627, -23.5),
                        control2: Vec2::new(-41.02058, -23.083534),
                        to: Vec2::new(-35.5, -27.5),
                    },
                    PathCommand::LineTo(Vec2::new(-39.5, -37.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-43.186527, -35.656734),
                        control2: Vec2::new(-40.972992, -37.027008),
                        to: Vec2::new(-45.5, -32.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.75901276, 0.7572839, 0.8603422, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(102.5, 37.5)),
                    PathCommand::LineTo(Vec2::new(108.5, 105.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(111.11692, 95.03233),
                        control2: Vec2::new(109.5, 102.874596),
                        to: Vec2::new(109.5, 81.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(108.60172, 65.33101),
                        control2: Vec2::new(107.65106, 49.953194),
                        to: Vec2::new(102.5, 34.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6550728, 0.44893473, 0.2639811, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(7.5, -58.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(9.578144, -48.10928),
                        control2: Vec2::new(6.8510704, -53.91223),
                        to: Vec2::new(24.5, -49.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(22.051317, -56.846046),
                        control2: Vec2::new(14.536594, -58.327232),
                        to: Vec2::new(7.5, -59.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.46869224, 0.47140715, 0.602327, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(98.5, 33.5)),
                    PathCommand::LineTo(Vec2::new(103.5, 65.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(110.706856, 58.29314),
                        control2: Vec2::new(105.58538, 64.43919),
                        to: Vec2::new(103.5, 41.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(99.394554, 29.18367),
                        control2: Vec2::new(103.71641, 29.5),
                        to: Vec2::new(98.5, 29.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.46832275, 0.3649674, 0.24610028, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-44.5, 20.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-35.41324, 25.04338),
                        control2: Vec2::new(-43.2212, 21.688139),
                        to: Vec2::new(-19.5, 15.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.429039, 9.358077),
                        control2: Vec2::new(-18.124447, 14.143805),
                        to: Vec2::new(-25.5, 1.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-26.833334, 0.16666663),
                        control2: Vec2::new(-26.166666, 0.16666663),
                        to: Vec2::new(-27.5, 1.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.39673206, 0.34028968, 0.5431837, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-92.5, 91.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-86.54464, 97.45536),
                        control2: Vec2::new(-91.925804, 92.7017),
                        to: Vec2::new(-72.5, 83.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-80.834946, 77.943375),
                        control2: Vec2::new(-74.43776, 80.93764),
                        to: Vec2::new(-92.5, 90.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.42628482, 0.32962394, 0.6933763, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-22.5, 44.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-10.672339, 38.04855),
                        control2: Vec2::new(-14.453339, 41.45334),
                        to: Vec2::new(-9.5, 36.5),
                    },
                    PathCommand::LineTo(Vec2::new(-10.5, 32.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-12.554089, 30.445911),
                        control2: Vec2::new(-11.078385, 31.289192),
                        to: Vec2::new(-15.5, 33.5),
                    },
                    PathCommand::LineTo(Vec2::new(-22.5, 43.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9842594, 0.98556656, 1.0, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(1.5, 19.5)),
                    PathCommand::LineTo(Vec2::new(20.5, 19.5)),
                    PathCommand::LineTo(Vec2::new(19.5, 16.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(11.286399, 13.762133),
                        control2: Vec2::new(15.608191, 14.5),
                        to: Vec2::new(6.5, 14.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4534314, 0.45354033, 0.5967863, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-24.5, 46.5)),
                    PathCommand::LineTo(Vec2::new(-3.5, 57.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-6.673755, 40.573303),
                        control2: Vec2::new(-2.663041, 45.33696),
                        to: Vec2::new(-8.5, 39.5),
                    },
                    PathCommand::LineTo(Vec2::new(-22.5, 45.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4364879, 0.367775, 0.5786871, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-50.5, 46.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-48.782303, 53.027256),
                        control2: Vec2::new(-50.249485, 65.5),
                        to: Vec2::new(-43.5, 65.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-36.95911, 58.95911),
                        control2: Vec2::new(-44.314228, 48.623848),
                        to: Vec2::new(-50.5, 44.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.34155002, 0.29511365, 0.54366606, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(81.5, -13.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(88.81518, 2.1753817),
                        control2: Vec2::new(83.56488, -0.96756005),
                        to: Vec2::new(90.5, 2.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(88.93039, -8.487253),
                        control2: Vec2::new(90.47538, -2.3541574),
                        to: Vec2::new(84.5, -15.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5800066, 0.4698812, 0.39961419, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(70.5, -33.5)),
                    PathCommand::LineTo(Vec2::new(81.5, -14.5)),
                    PathCommand::LineTo(Vec2::new(83.5, -15.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(80.621605, -22.695986),
                        control2: Vec2::new(76.96457, -28.035433),
                        to: Vec2::new(71.5, -33.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6555449, 0.5418834, 0.24525881, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 43.5)),
                    PathCommand::LineTo(Vec2::new(3.5, 40.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(5.8233805, 30.044786),
                        control2: Vec2::new(6.8743687, 34.248737),
                        to: Vec2::new(3.5, 27.5),
                    },
                    PathCommand::LineTo(Vec2::new(0.5, 24.5)),
                    PathCommand::LineTo(Vec2::new(-0.5, 40.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.720654, 0.73392177, 0.92202675, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(8.5, 27.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(20.727173, 27.5),
                        control2: Vec2::new(12.572998, 28.035278),
                        to: Vec2::new(32.5, 22.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(17.71024, 21.362326),
                        control2: Vec2::new(23.535286, 19.885887),
                        to: Vec2::new(14.5, 23.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7338237, 0.739636, 0.8362744, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-31.5, 48.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-22.457949, 48.5),
                        control2: Vec2::new(-20.5, 34.911194),
                        to: Vec2::new(-20.5, 28.5),
                    },
                    PathCommand::LineTo(Vec2::new(-31.5, 47.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.44226575, 0.3904867, 0.59041417, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-23.5, -4.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.366055, 4.0607357),
                        control2: Vec2::new(-21.103409, 1.5),
                        to: Vec2::new(-8.5, 1.5),
                    },
                    PathCommand::LineTo(Vec2::new(-5.5, -6.5)),
                    PathCommand::LineTo(Vec2::new(-5.5, -8.5)),
                    PathCommand::LineTo(Vec2::new(-6.5, -8.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3287582, 0.29629633, 0.46252733, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-54.5, -33.5)),
                    PathCommand::LineTo(Vec2::new(-34.5, -52.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-49.389877, -45.05506),
                        control2: Vec2::new(-42.227764, -50.453907),
                        to: Vec2::new(-54.5, -34.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41250455, 0.407473, 0.5364408, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-107.5, 81.5)),
                    PathCommand::LineTo(Vec2::new(-105.5, 83.5)),
                    PathCommand::LineTo(Vec2::new(-104.5, 65.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-104.5, 63.028618),
                        control2: Vec2::new(-104.0286, 63.971394),
                        to: Vec2::new(-105.5, 62.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-108.14652, 67.793045),
                        control2: Vec2::new(-106.43253, 63.7579),
                        to: Vec2::new(-107.5, 75.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.42868125, 0.33171844, 0.70003855, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-50.5, 77.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-36.43779, 77.5),
                        control2: Vec2::new(-46.484184, 77.748024),
                        to: Vec2::new(-20.5, 74.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-27.237453, 71.13127),
                        control2: Vec2::new(-42.544544, 75.87546),
                        to: Vec2::new(-46.5, 76.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7943865, 0.6660514, 0.29604015, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(4.5, 54.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(16.10685, 59.77584),
                        control2: Vec2::new(11.472326, 59.5),
                        to: Vec2::new(17.5, 59.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(17.5, 52.283615),
                        control2: Vec2::new(18.448664, 56.816223),
                        to: Vec2::new(5.5, 52.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.28235313, 0.25098038, 0.42745087, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-88.5, 31.5)),
                    PathCommand::LineTo(Vec2::new(-80.5, 30.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-80.5, 26.239283),
                        control2: Vec2::new(-85.223915, 24.592028),
                        to: Vec2::new(-88.5, 23.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.18156035, 0.16971214, 0.24939507, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(58.5, -20.5)),
                    PathCommand::LineTo(Vec2::new(61.5, 12.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(64.552895, 0.28841972),
                        control2: Vec2::new(62.342125, -10.97363),
                        to: Vec2::new(58.5, -22.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2163537, 0.20025024, 0.3105549, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(14.5, 6.5)),
                    PathCommand::LineTo(Vec2::new(16.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(21.99568, 8.5),
                        control2: Vec2::new(24.5, 0.676918),
                        to: Vec2::new(24.5, -4.5),
                    },
                    PathCommand::LineTo(Vec2::new(14.5, 5.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.61415184, 0.62318844, 0.73981255, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(48.5, -9.5)),
                    PathCommand::LineTo(Vec2::new(58.5, 12.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(59.64907, 14.79814),
                        control2: Vec2::new(58.75465, 14.5),
                        to: Vec2::new(60.5, 14.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(57.47521, 6.181823),
                        control2: Vec2::new(54.923042, -4.076956),
                        to: Vec2::new(48.5, -10.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2271895, 0.20836595, 0.33403045, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-63.5, 10.5)),
                    PathCommand::LineTo(Vec2::new(-56.5, -17.5)),
                    PathCommand::LineTo(Vec2::new(-57.5, -17.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-62.47192, -2.584239),
                        control2: Vec2::new(-60.585293, -9.616185),
                        to: Vec2::new(-63.5, 3.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.77559894, 0.788671, 0.8809585, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-100.5, 84.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-88.36312, 78.431564),
                        control2: Vec2::new(-98.5, 84.24573),
                        to: Vec2::new(-98.5, 49.5),
                    },
                    PathCommand::LineTo(Vec2::new(-99.5, 54.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5782036, 0.47897866, 0.68691295, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-13.5, 76.5)),
                    PathCommand::LineTo(Vec2::new(8.5, 76.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(0.11488247, 73.70496),
                        control2: Vec2::new(6.5520153, 75.56188),
                        to: Vec2::new(-11.5, 74.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7563155, 0.60018265, 0.2575467, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(3.5, 25.5)),
                    PathCommand::LineTo(Vec2::new(5.5, 28.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(12.481699, 25.707321),
                        control2: Vec2::new(9.098318, 27.621346),
                        to: Vec2::new(15.5, 22.5),
                    },
                    PathCommand::LineTo(Vec2::new(8.5, 22.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.81203806, 0.82006365, 0.94254464, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-16.5, 2.5)),
                    PathCommand::LineTo(Vec2::new(-10.5, 12.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-8.052787, 7.605574),
                        control2: Vec2::new(-8.5, 9.990711),
                        to: Vec2::new(-8.5, 5.5),
                    },
                    PathCommand::LineTo(Vec2::new(-13.5, 2.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.38613772, 0.38668495, 0.5191974, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(6.5, 36.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(12.506941, 42.506943),
                        control2: Vec2::new(7.6681204, 38.05633),
                        to: Vec2::new(23.5, 46.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(15.426812, 38.42681),
                        control2: Vec2::new(20.488544, 42.765438),
                        to: Vec2::new(7.5, 34.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7622191, 0.76776654, 0.8678142, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(98.5, 94.5)),
                    PathCommand::LineTo(Vec2::new(102.5, 97.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(110.83478, 97.5),
                        control2: Vec2::new(107.76944, 80.76944),
                        to: Vec2::new(102.5, 75.5),
                    },
                    PathCommand::LineTo(Vec2::new(98.5, 93.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23036708, 0.20995478, 0.33624944, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-105.5, 61.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-100.06743, 66.93257),
                        control2: Vec2::new(-102.5, 48.676643),
                        to: Vec2::new(-102.5, 44.5),
                    },
                    PathCommand::LineTo(Vec2::new(-103.5, 43.5)),
                    PathCommand::LineTo(Vec2::new(-105.5, 55.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4049271, 0.3082957, 0.5980895, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-39.5, 64.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-37.150043, 66.84996),
                        control2: Vec2::new(-35.455776, 65.45577),
                        to: Vec2::new(-33.5, 63.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-33.5, 60.25968),
                        control2: Vec2::new(-37.17333, 60.08167),
                        to: Vec2::new(-39.5, 59.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.1636739, 0.15221883, 0.19060886, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(13.5, 18.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(24.807629, 19.913454),
                        control2: Vec2::new(18.812542, 19.5),
                        to: Vec2::new(31.5, 19.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(20.24998, 16.687496),
                        control2: Vec2::new(26.235199, 17.77352),
                        to: Vec2::new(13.5, 16.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3971104, 0.3971104, 0.52033013, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(10.5, 68.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(18.441452, 73.794304),
                        control2: Vec2::new(14.096295, 72.5),
                        to: Vec2::new(23.5, 72.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(18.656757, 68.86757),
                        control2: Vec2::new(21.081123, 70.02704),
                        to: Vec2::new(16.5, 68.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33704284, 0.2426074, 0.30429265, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-50.5, -41.5)),
                    PathCommand::LineTo(Vec2::new(-32.5, -57.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-40.488766, -54.83708),
                        control2: Vec2::new(-45.15756, -48.605644),
                        to: Vec2::new(-50.5, -42.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.20261437, 0.18758173, 0.2821351, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-102.5, 43.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-97.91338, 38.913387),
                        control2: Vec2::new(-96.661316, 31.177364),
                        to: Vec2::new(-99.5, 25.5),
                    },
                    PathCommand::LineTo(Vec2::new(-102.5, 38.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.38638985, 0.28973478, 0.49653974, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 39.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(2.5467672, 34.929848),
                        control2: Vec2::new(2.5467758, 31.070164),
                        to: Vec2::new(-0.5, 26.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.77104944, 0.7853517, 0.98546726, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-19.5, 34.5)),
                    PathCommand::LineTo(Vec2::new(-9.5, 31.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-11.406765, 27.68647),
                        control2: Vec2::new(-9.764787, 29.622536),
                        to: Vec2::new(-16.5, 28.5),
                    },
                    PathCommand::LineTo(Vec2::new(-19.5, 32.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.55282587, 0.5547866, 0.7557093, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-39.5, 54.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-31.918861, 57.027046),
                        control2: Vec2::new(-34.764915, 58.186554),
                        to: Vec2::new(-30.5, 52.5),
                    },
                    PathCommand::LineTo(Vec2::new(-30.5, 51.5)),
                    PathCommand::LineTo(Vec2::new(-39.5, 52.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33891857, 0.2936423, 0.5209744, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-21.5, 4.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.976845, 10.530874),
                        control2: Vec2::new(-20.031914, 7.843465),
                        to: Vec2::new(-11.5, 11.5),
                    },
                    PathCommand::LineTo(Vec2::new(-21.5, 1.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.78633386, 0.7902554, 0.89031494, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-12.5, -36.5)),
                    PathCommand::LineTo(Vec2::new(10.5, -36.5)),
                    PathCommand::LineTo(Vec2::new(-5.5, -37.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.22388588, 0.20570418, 0.32382652, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(87.5, 94.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(91.93531, 99.67453),
                        control2: Vec2::new(92.407745, 102.5),
                        to: Vec2::new(98.5, 102.5),
                    },
                    PathCommand::LineTo(Vec2::new(88.5, 94.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8707108, 0.7975488, 0.36397064, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-86.5, -9.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-79.833374, -9.5),
                        control2: Vec2::new(-85.65683, -9.205939),
                        to: Vec2::new(-77.5, -24.5),
                    },
                    PathCommand::LineTo(Vec2::new(-75.5, -28.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-77.701866, -28.5),
                        control2: Vec2::new(-76.315094, -28.777363),
                        to: Vec2::new(-78.5, -25.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3132353, 0.21850494, 0.19313726, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-98.5, 25.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-94.46279, 21.462793),
                        control2: Vec2::new(-93.5, 15.16144),
                        to: Vec2::new(-93.5, 9.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-98.1428, 9.5),
                        control2: Vec2::new(-97.13462, 18.062513),
                        to: Vec2::new(-98.5, 22.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3606536, 0.2648366, 0.3947713, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 14.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.2418313, 17.983662),
                        control2: Vec2::new(3.1278627, 16.872137),
                        to: Vec2::new(5.5, 14.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(5.5, 11.553225),
                        control2: Vec2::new(1.731605, 11.5),
                        to: Vec2::new(-0.5, 11.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6258825, 0.6373856, 0.79986924, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-93.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-86.80715, 8.5),
                        control2: Vec2::new(-86.5, -2.3374457),
                        to: Vec2::new(-86.5, -8.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-91.07534, -1.6369901),
                        control2: Vec2::new(-88.068596, -6.621643),
                        to: Vec2::new(-93.5, 7.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33084968, 0.23869282, 0.29490197, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(61.5, -26.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(67.68485, -17.222717),
                        control2: Vec2::new(63.894478, -17.5),
                        to: Vec2::new(68.5, -17.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(64.0719, -27.463224),
                        control2: Vec2::new(67.61447, -25.461842),
                        to: Vec2::new(61.5, -27.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2288889, 0.2066667, 0.3364706, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-66.5, 4.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-65.840866, 9.773048),
                        control2: Vec2::new(-66.99371, 13.253141),
                        to: Vec2::new(-62.5, 15.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-58.842888, 11.842889),
                        control2: Vec2::new(-65.26042, 0.2786175),
                        to: Vec2::new(-65.5, -0.5),
                    },
                    PathCommand::LineTo(Vec2::new(-66.5, 2.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.47450978, 0.40743744, 0.6423258, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(0.5, -14.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.9216375, -4.5485373),
                        control2: Vec2::new(0.4459057, -9.608189),
                        to: Vec2::new(5.5, 0.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(4.0527873, -9.630487),
                        control2: Vec2::new(5.2453547, -4.263936),
                        to: Vec2::new(1.5, -15.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3081812, 0.27085873, 0.43867475, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(12.5, 4.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(18.271236, 4.5),
                        control2: Vec2::new(14.028598, 4.971402),
                        to: Vec2::new(22.5, -3.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(16.584518, -1.5281731),
                        control2: Vec2::new(20.300335, -3.1860008),
                        to: Vec2::new(12.5, 3.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.52843124, 0.5372551, 0.67507005, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(23.5, -60.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(37.78017, -51.319893),
                        control2: Vec2::new(31.952257, -51.5),
                        to: Vec2::new(38.5, -51.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(31.546679, -58.453323),
                        control2: Vec2::new(36.15533, -54.59709),
                        to: Vec2::new(23.5, -61.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.18879552, 0.17563027, 0.25994402, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(9.5, 31.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(21.735054, 43.735054),
                        control2: Vec2::new(15.843199, 43.5),
                        to: Vec2::new(22.5, 43.5),
                    },
                    PathCommand::LineTo(Vec2::new(10.5, 31.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5253394, 0.53604823, 0.6745099, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-18.5, 46.5)),
                    PathCommand::LineTo(Vec2::new(-8.5, 46.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-9.58084, 42.17664),
                        control2: Vec2::new(-8.125634, 42.5),
                        to: Vec2::new(-10.5, 42.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.21317647, 0.19419606, 0.3052549, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(8.5, 34.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(9.904879, 38.714638),
                        control2: Vec2::new(12.247007, 40.5),
                        to: Vec2::new(17.5, 40.5),
                    },
                    PathCommand::LineTo(Vec2::new(11.5, 34.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.83218956, 0.83643794, 0.9330065, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-6.5, 4.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-5.1666665, 5.8333335),
                        control2: Vec2::new(-5.8333335, 5.8333335),
                        to: Vec2::new(-4.5, 4.5),
                    },
                    PathCommand::LineTo(Vec2::new(-4.5, -5.5)),
                    PathCommand::LineTo(Vec2::new(-6.5, 1.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6545752, 0.66503286, 0.78905237, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(79.5, 103.5)),
                    PathCommand::LineTo(Vec2::new(94.5, 110.5)),
                    PathCommand::LineTo(Vec2::new(81.5, 103.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.913555, 0.8668372, 0.40784317, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(6.5, 33.5)),
                    PathCommand::LineTo(Vec2::new(14.5, 34.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(13.406554, 31.219664),
                        control2: Vec2::new(10.01169, 26.98831),
                        to: Vec2::new(6.5, 30.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.37391302, 0.37647063, 0.5137255, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-66.5, -32.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-62.833393, -34.944405),
                        control2: Vec2::new(-60.90006, -37.299828),
                        to: Vec2::new(-59.5, -41.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-62.779514, -41.5),
                        control2: Vec2::new(-65.26277, -35.56205),
                        to: Vec2::new(-66.5, -33.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.26035804, 0.22711, 0.36930948, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(3.5, 45.5)),
                    PathCommand::LineTo(Vec2::new(4.5, 46.5)),
                    PathCommand::LineTo(Vec2::new(9.5, 40.5)),
                    PathCommand::LineTo(Vec2::new(5.5, 38.5)),
                    PathCommand::LineTo(Vec2::new(3.5, 44.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.35133693, 0.30409986, 0.4905526, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(19.5, 26.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(25.443285, 26.5),
                        control2: Vec2::new(30.925646, 24.358118),
                        to: Vec2::new(36.5, 22.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(29.699448, 22.5),
                        control2: Vec2::new(34.464676, 22.236906),
                        to: Vec2::new(22.5, 25.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.22941174, 0.20891269, 0.33636367, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-12.5, 2.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-5.659924, 7.0600505),
                        control2: Vec2::new(-7.3768387, 3.8841934),
                        to: Vec2::new(-5.5, -5.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23475936, 0.21336901, 0.34527633, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-100.5, 94.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-97.83332, 94.5),
                        control2: Vec2::new(-99.76681, 94.7001),
                        to: Vec2::new(-95.5, 91.5),
                    },
                    PathCommand::LineTo(Vec2::new(-96.5, 89.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-99.701836, 89.5),
                        control2: Vec2::new(-98.13021, 88.94531),
                        to: Vec2::new(-100.5, 92.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33520073, 0.2883287, 0.5411765, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-20.5, -9.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.486135, -9.5),
                        control2: Vec2::new(-19.5, -17.486135),
                        to: Vec2::new(-19.5, -21.5),
                    },
                    PathCommand::LineTo(Vec2::new(-20.5, -16.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3309057, 0.285901, 0.47861814, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-5.5, -9.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-1.5665283, -9.5),
                        control2: Vec2::new(-2.5, -17.287876),
                        to: Vec2::new(-2.5, -19.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-5.442809, -15.085787),
                        control2: Vec2::new(-4.264298, -17.678509),
                        to: Vec2::new(-5.5, -11.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.30065364, 0.264239, 0.43211952, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-58.5, -27.5)),
                    PathCommand::LineTo(Vec2::new(-51.5, -40.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-55.926685, -36.073315),
                        control2: Vec2::new(-53.04217, -39.324093),
                        to: Vec2::new(-58.5, -29.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.20728293, 0.19103643, 0.29094303, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(41.5, 92.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(55.474472, 95.72488),
                        control2: Vec2::new(50.052773, 95.5),
                        to: Vec2::new(57.5, 95.5),
                    },
                    PathCommand::LineTo(Vec2::new(46.5, 92.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.90745085, 0.85627455, 0.40313727, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(0.5, 56.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(4.8247156, 56.5),
                        control2: Vec2::new(6.34243, 49.34243),
                        to: Vec2::new(3.5, 46.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(0.7840576, 49.215942),
                        control2: Vec2::new(2.629522, 47.046673),
                        to: Vec2::new(0.5, 54.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.39078432, 0.33529413, 0.53411764, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 47.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(2.7403233, 47.5),
                        control2: Vec2::new(2.9183304, 43.82668),
                        to: Vec2::new(3.5, 41.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.085787, 41.5),
                        control2: Vec2::new(2.7356987, 41.2643),
                        to: Vec2::new(-0.5, 44.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.672549, 0.6845099, 0.859804, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(40.5, -49.5)),
                    PathCommand::LineTo(Vec2::new(48.5, -40.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(44.754635, -47.990726),
                        control2: Vec2::new(47.288864, -45.15835),
                        to: Vec2::new(41.5, -49.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.18254906, 0.17039219, 0.24980395, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(88.5, 16.5)),
                    PathCommand::LineTo(Vec2::new(91.5, 29.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(91.5, 21.019947),
                        control2: Vec2::new(91.978874, 26.09625),
                        to: Vec2::new(88.5, 14.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3729618, 0.29659447, 0.48586178, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-32.5, 25.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-29.09628, 28.90372),
                        control2: Vec2::new(-31.962517, 26.407497),
                        to: Vec2::new(-21.5, 28.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-21.5, 25.651999),
                        control2: Vec2::new(-26.737034, 26.190742),
                        to: Vec2::new(-29.5, 25.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40433434, 0.34158924, 0.5426213, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-55.5, -3.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-48.309044, -13.772794),
                        control2: Vec2::new(-48.5, -9.431177),
                        to: Vec2::new(-48.5, -14.5),
                    },
                    PathCommand::LineTo(Vec2::new(-55.5, -4.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.24086687, 0.21795672, 0.35479882, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-74.5, 74.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-69.61751, 74.5),
                        control2: Vec2::new(-65.1006, 73.03353),
                        to: Vec2::new(-60.5, 71.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-66.248726, 71.5),
                        control2: Vec2::new(-62.52994, 71.257484),
                        to: Vec2::new(-71.5, 73.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5708061, 0.47320262, 0.68061, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-22.5, 36.5)),
                    PathCommand::LineTo(Vec2::new(-17.5, 27.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-23.24874, 27.5),
                        control2: Vec2::new(-20.25747, 26.529877),
                        to: Vec2::new(-22.5, 35.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23943359, 0.2163399, 0.35446626, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-19.5, 9.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.328497, 14.785838),
                        control2: Vec2::new(-18.443647, 14.5),
                        to: Vec2::new(-15.5, 14.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-14.028594, 10.085783),
                        control2: Vec2::new(-13.557188, 11.971406),
                        to: Vec2::new(-18.5, 9.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6381264, 0.64553386, 0.78126365, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(9.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(12.429851, 11.429851),
                        control2: Vec2::new(12.7438135, 8.524748),
                        to: Vec2::new(13.5, 5.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(10.656723, 4.0783615),
                        control2: Vec2::new(12.027044, 3.9729555),
                        to: Vec2::new(9.5, 6.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.47973862, 0.48649243, 0.61830074, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(89.5, -2.5)),
                    PathCommand::LineTo(Vec2::new(94.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(94.5, 2.7732449),
                        control2: Vec2::new(94.79807, 7.096142),
                        to: Vec2::new(89.5, -3.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6405229, 0.45773423, 0.30196083, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-23.5, -1.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.2643, 5.7357006),
                        control2: Vec2::new(-19.799839, 5.5),
                        to: Vec2::new(-15.5, 5.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-15.5, 1.4268241),
                        control2: Vec2::new(-15.246951, 4.7168994),
                        to: Vec2::new(-22.5, -1.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.53376913, 0.5442266, 0.68366027, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-21.5, -46.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-20.3151, -40.575493),
                        control2: Vec2::new(-21.701845, -42.701847),
                        to: Vec2::new(-18.5, -39.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.015043, -41.98496),
                        control2: Vec2::new(-17.82219, -47.5),
                        to: Vec2::new(-21.5, -47.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.92004377, 0.9213509, 1.0, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-5.5, -66.5)),
                    PathCommand::LineTo(Vec2::new(9.5, -65.5)),
                    PathCommand::LineTo(Vec2::new(4.5, -66.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.21873638, 0.19912852, 0.3122005, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-10.5, -115.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-2.2333298, -121.700005),
                        control2: Vec2::new(-2.5, -118.16669),
                        to: Vec2::new(-2.5, -122.5),
                    },
                    PathCommand::LineTo(Vec2::new(-10.5, -116.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.68671024, 0.46688452, 0.2666667, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-64.5, 8.5)),
                    PathCommand::LineTo(Vec2::new(-64.5, -7.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.20899653, 0.1928489, 0.29665515, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(18.5, 46.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(27.065361, 51.85335),
                        control2: Vec2::new(23.355333, 51.5),
                        to: Vec2::new(28.5, 51.5),
                    },
                    PathCommand::LineTo(Vec2::new(19.5, 46.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4997549, 0.42034316, 0.62696075, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-37.5, 20.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-29.557196, 20.5),
                        control2: Vec2::new(-32.149914, 22.149916),
                        to: Vec2::new(-28.5, 18.5),
                    },
                    PathCommand::LineTo(Vec2::new(-30.5, 18.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.657353, 0.66838235, 0.84411776, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-21.5, 20.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-12.028645, 20.5),
                        control2: Vec2::new(-14.621317, 22.621317),
                        to: Vec2::new(-11.5, 19.5),
                    },
                    PathCommand::LineTo(Vec2::new(-17.5, 19.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7642156, 0.7784313, 0.9764706, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 19.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.4428142, 21.442814),
                        control2: Vec2::new(0.028582096, 20.5),
                        to: Vec2::new(4.5, 20.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.8333361, 16.500004),
                        control2: Vec2::new(3.4999962, 18.166664),
                        to: Vec2::new(-0.5, 15.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6705883, 0.6833334, 0.857598, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-4.5, 15.5)),
                    PathCommand::LineTo(Vec2::new(-3.5, 16.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-1.8675424, 11.602627),
                        control2: Vec2::new(-2.5, 14.55411),
                        to: Vec2::new(-2.5, 7.5),
                    },
                    PathCommand::LineTo(Vec2::new(-3.5, 9.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8044118, 0.814951, 0.93946075, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(89.5, 4.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(91.03749, 7.574975),
                        control2: Vec2::new(89.87981, 6.0865383),
                        to: Vec2::new(93.5, 8.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(91.22318, 2.8079643),
                        control2: Vec2::new(93.10554, 4.302768),
                        to: Vec2::new(89.5, 2.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.50196075, 0.39803922, 0.34705883, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-26.5, -2.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-22.3151, 3.777351),
                        control2: Vec2::new(-24.903702, 3.5),
                        to: Vec2::new(-21.5, 3.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-22.947214, -0.8416438),
                        control2: Vec2::new(-21.754639, 1.2453604),
                        to: Vec2::new(-25.5, -2.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.74460787, 0.7492647, 0.8414215, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-56.5, -18.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-54.76004, -21.283937),
                        control2: Vec2::new(-50.1786, -25.178602),
                        to: Vec2::new(-52.5, -27.5),
                    },
                    PathCommand::LineTo(Vec2::new(-56.5, -19.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8480392, 0.8664216, 0.95392156, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-14.5, -86.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-6.1402016, -88.88851),
                        control2: Vec2::new(-9.215938, -86.784065),
                        to: Vec2::new(-4.5, -91.5),
                    },
                    PathCommand::LineTo(Vec2::new(-13.5, -87.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.410049, 0.32352942, 0.53651965, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(71.5, 100.5)),
                    PathCommand::LineTo(Vec2::new(78.5, 102.5)),
                    PathCommand::LineTo(Vec2::new(71.5, 98.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9027451, 0.8478432, 0.3963399, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(50.5, 69.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(60.132362, 72.710785),
                        control2: Vec2::new(56.33774, 72.5),
                        to: Vec2::new(61.5, 72.5),
                    },
                    PathCommand::LineTo(Vec2::new(53.5, 69.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.53751636, 0.4481046, 0.65359473, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-91.5, 84.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-84.656746, 81.07837),
                        control2: Vec2::new(-87.08113, 83.08113),
                        to: Vec2::new(-83.5, 79.5),
                    },
                    PathCommand::LineTo(Vec2::new(-89.5, 82.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.57226896, 0.47478992, 0.68235296, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-27.5, 50.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-24.17492, 50.5),
                        control2: Vec2::new(-21.421877, 48.960938),
                        to: Vec2::new(-18.5, 47.5),
                    },
                    PathCommand::LineTo(Vec2::new(-19.5, 46.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.24033618, 0.21512607, 0.34005603, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(52.5, -8.5)),
                    PathCommand::LineTo(Vec2::new(55.5, -2.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(54.377464, -9.235203),
                        control2: Vec2::new(56.313522, -7.593239),
                        to: Vec2::new(52.5, -9.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8436974, 0.8187675, 0.8946778, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-110.5, 108.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-107.425026, 106.96251),
                        control2: Vec2::new(-108.913445, 108.12016),
                        to: Vec2::new(-106.5, 104.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-109.70185, 104.5),
                        control2: Vec2::new(-108.13021, 103.94532),
                        to: Vec2::new(-110.5, 107.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6865762, 0.46666673, 0.26666668, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-98.5, 79.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-97.43632, 76.30896),
                        control2: Vec2::new(-96.97327, 74.553474),
                        to: Vec2::new(-98.5, 71.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23227753, 0.20995478, 0.3420815, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(92.5, 34.5)),
                    PathCommand::LineTo(Vec2::new(94.5, 40.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(94.5, 34.3918),
                        control2: Vec2::new(94.92165, 37.764954),
                        to: Vec2::new(92.5, 30.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.37315235, 0.29743597, 0.48808444, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(23.5, 20.5)),
                    PathCommand::LineTo(Vec2::new(35.5, 20.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.42473605, 0.42895928, 0.55173457, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-31.5, -5.5)),
                    PathCommand::LineTo(Vec2::new(-27.5, 0.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-30.631308, -6.806383),
                        control2: Vec2::new(-27.961407, -6.5),
                        to: Vec2::new(-31.5, -6.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5013575, 0.4226245, 0.63227767, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(38.5, -21.5)),
                    PathCommand::LineTo(Vec2::new(44.5, -15.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(44.5, -18.857016),
                        control2: Vec2::new(44.73567, -16.264332),
                        to: Vec2::new(39.5, -21.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.27843142, 0.24615386, 0.41116136, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(43.5, -54.5)),
                    PathCommand::LineTo(Vec2::new(50.5, -52.5)),
                    PathCommand::LineTo(Vec2::new(47.5, -55.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.37224737, 0.2983409, 0.4901961, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-20.5, 67.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, 67.5)),
                    PathCommand::LineTo(Vec2::new(-19.5, 66.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.25686276, 0.2297386, 0.38137263, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(2.5, 60.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(6.1324544, 59.28918),
                        control2: Vec2::new(5.5, 60.554096),
                        to: Vec2::new(5.5, 57.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(2.7546496, 57.5),
                        control2: Vec2::new(3.7981367, 56.903725),
                        to: Vec2::new(2.5, 59.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.28202617, 0.2509804, 0.4267974, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(15.5, 29.5)),
                    PathCommand::LineTo(Vec2::new(17.5, 32.5)),
                    PathCommand::LineTo(Vec2::new(18.5, 27.5)),
                    PathCommand::LineTo(Vec2::new(15.5, 27.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23431376, 0.21405232, 0.34477127, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-1.5, -5.5)),
                    PathCommand::LineTo(Vec2::new(-1.5, -16.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.79673207, 0.8026144, 0.90228766, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(96.5, 104.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(99.73579, 107.73579),
                        control2: Vec2::new(98.085785, 107.5),
                        to: Vec2::new(100.5, 107.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(100.5, 104.75466),
                        control2: Vec2::new(101.0963, 105.79815),
                        to: Vec2::new(98.5, 104.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8670232, 0.79001784, 0.36007133, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 51.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(1.7445331, 51.5),
                        control2: Vec2::new(1.5, 49.195137),
                        to: Vec2::new(1.5, 47.5),
                    },
                    PathCommand::LineTo(Vec2::new(-0.5, 48.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6, 0.6128343, 0.77005357, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(8.5, 41.5)),
                    PathCommand::LineTo(Vec2::new(15.5, 44.5)),
                    PathCommand::LineTo(Vec2::new(10.5, 41.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.45561498, 0.38680926, 0.5864528, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-30.5, 38.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-27.317833, 41.682167),
                        control2: Vec2::new(-29.9386, 34.6228),
                        to: Vec2::new(-30.5, 33.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.29447418, 0.25918007, 0.45383248, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-1.5, 5.5)),
                    PathCommand::LineTo(Vec2::new(-1.5, -4.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8541889, 0.85811055, 0.9468805, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-51.5, -27.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-48.299984, -31.766685),
                        control2: Vec2::new(-48.5, -29.833326),
                        to: Vec2::new(-48.5, -32.5),
                    },
                    PathCommand::LineTo(Vec2::new(-51.5, -29.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.88983965, 0.90944743, 0.9907308, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, -34.5)),
                    PathCommand::LineTo(Vec2::new(9.5, -34.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33262032, 0.2894831, 0.46951872, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-3.5, -92.5)),
                    PathCommand::LineTo(Vec2::new(4.5, -90.5)),
                    PathCommand::LineTo(Vec2::new(0.5, -92.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40499112, 0.32014266, 0.5294118, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(95.5, 111.5)),
                    PathCommand::LineTo(Vec2::new(101.5, 114.5)),
                    PathCommand::LineTo(Vec2::new(96.5, 111.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.91411763, 0.867451, 0.4082353, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(118.5, 98.5)),
                    PathCommand::LineTo(Vec2::new(118.5, 89.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.45215687, 0.34705883, 0.52745104, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(23.5, 90.5)),
                    PathCommand::LineTo(Vec2::new(32.5, 90.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.862353, 0.77882355, 0.36862746, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-98.5, 70.5)),
                    PathCommand::LineTo(Vec2::new(-96.5, 77.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-96.5, 71.75461),
                        control2: Vec2::new(-95.75464, 73.99072),
                        to: Vec2::new(-97.5, 70.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.609804, 0.5035295, 0.71960783, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(107.5, 75.5)),
                    PathCommand::LineTo(Vec2::new(107.5, 66.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.57254905, 0.39686278, 0.23137255, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-32.5, 49.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-30.874304, 51.125698),
                        control2: Vec2::new(-29.125696, 51.125694),
                        to: Vec2::new(-27.5, 49.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.52862746, 0.44392157, 0.6556863, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-45.5, 21.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-39.881004, 23.747599),
                        control2: Vec2::new(-42.295055, 23.5),
                        to: Vec2::new(-38.5, 23.5),
                    },
                    PathCommand::LineTo(Vec2::new(-43.5, 21.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5117647, 0.4301961, 0.64, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(21.5, 21.5)),
                    PathCommand::LineTo(Vec2::new(30.5, 21.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.58745104, 0.5952942, 0.70509803, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(17.5, 10.5)),
                    PathCommand::LineTo(Vec2::new(20.5, 4.5)),
                    PathCommand::LineTo(Vec2::new(17.5, 9.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.22784315, 0.20745099, 0.33450982, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(12.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(14.027045, 10.027045),
                        control2: Vec2::new(13.0783615, 9.710819),
                        to: Vec2::new(15.5, 8.5),
                    },
                    PathCommand::LineTo(Vec2::new(13.5, 6.5)),
                    PathCommand::LineTo(Vec2::new(12.5, 7.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.57215685, 0.5780393, 0.6972549, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(33.5, -25.5)),
                    PathCommand::LineTo(Vec2::new(37.5, -21.5)),
                    PathCommand::LineTo(Vec2::new(38.5, -22.5)),
                    PathCommand::LineTo(Vec2::new(34.5, -25.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.34, 0.29529414, 0.4803922, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-20.5, -81.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-18.306822, -81.5),
                        control2: Vec2::new(-12.8217945, -85.5),
                        to: Vec2::new(-16.5, -85.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4047059, 0.31960785, 0.52941173, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-4.5, -90.5)),
                    PathCommand::LineTo(Vec2::new(1.5, -90.5)),
                    PathCommand::LineTo(Vec2::new(-2.5, -91.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.1709804, 0.1592157, 0.21568628, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-21.5, 73.5)),
                    PathCommand::LineTo(Vec2::new(-16.5, 76.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-16.5, 74.5572),
                        control2: Vec2::new(-16.264305, 75.735695),
                        to: Vec2::new(-18.5, 73.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8579522, 0.77472764, 0.35206977, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-59.5, 70.5)),
                    PathCommand::LineTo(Vec2::new(-52.5, 69.5)),
                    PathCommand::LineTo(Vec2::new(-56.5, 69.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.55729854, 0.46230936, 0.671024, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-22.5, 37.5)),
                    PathCommand::LineTo(Vec2::new(-21.5, 39.5)),
                    PathCommand::LineTo(Vec2::new(-19.5, 35.5)),
                    PathCommand::LineTo(Vec2::new(-20.5, 35.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.49586058, 0.5023965, 0.6762528, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(24.5, 17.5)),
                    PathCommand::LineTo(Vec2::new(31.5, 18.5)),
                    PathCommand::LineTo(Vec2::new(28.5, 17.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23398693, 0.21394338, 0.3429194, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-69.5, -29.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-67.2643, -27.2643),
                        control2: Vec2::new(-68.44281, -27.5),
                        to: Vec2::new(-66.5, -27.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(-67.60541, -30.816233),
                        control2: Vec2::new(-66.44591, -30.5),
                        to: Vec2::new(-68.5, -30.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.37342048, 0.31939, 0.52592593, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(58.5, -39.5)),
                    PathCommand::LineTo(Vec2::new(61.5, -35.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(60.20186, -39.39443),
                        control2: Vec2::new(61.245358, -37.754642),
                        to: Vec2::new(58.5, -40.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4061002, 0.31938997, 0.530719, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-38.5, -63.5)),
                    PathCommand::LineTo(Vec2::new(-33.5, -65.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-35.027042, -67.02705),
                        control2: Vec2::new(-34.078365, -66.710815),
                        to: Vec2::new(-36.5, -65.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.37908497, 0.30283228, 0.49978215, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-100.5, 112.5)),
                    PathCommand::LineTo(Vec2::new(-95.5, 110.5)),
                    PathCommand::LineTo(Vec2::new(-99.5, 111.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9377451, 0.9098039, 0.45196077, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-80.5, 102.5)),
                    PathCommand::LineTo(Vec2::new(-74.5, 101.5)),
                    PathCommand::LineTo(Vec2::new(-77.5, 101.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9406863, 0.91372555, 0.45196077, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(64.5, 98.5)),
                    PathCommand::LineTo(Vec2::new(70.5, 99.5)),
                    PathCommand::LineTo(Vec2::new(67.5, 98.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.90980387, 0.85931385, 0.40441176, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(33.5, 91.5)),
                    PathCommand::LineTo(Vec2::new(40.5, 91.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.88676465, 0.81960785, 0.3877451, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(7.5, 89.5)),
                    PathCommand::LineTo(Vec2::new(14.5, 89.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.81813735, 0.70000005, 0.33382353, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-82.5, 78.5)),
                    PathCommand::LineTo(Vec2::new(-77.5, 76.5)),
                    PathCommand::LineTo(Vec2::new(-81.5, 77.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.56666666, 0.47009802, 0.6769608, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(8.5, 33.5)),
                    PathCommand::LineTo(Vec2::new(10.5, 35.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(10.5, 32.557194),
                        control2: Vec2::new(10.971403, 33.971405),
                        to: Vec2::new(8.5, 31.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.879902, 0.8857844, 0.9906863, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-35.5, 23.5)),
                    PathCommand::LineTo(Vec2::new(-29.5, 24.5)),
                    PathCommand::LineTo(Vec2::new(-32.5, 23.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.25539216, 0.23088236, 0.3794118, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(76.5, -13.5)),
                    PathCommand::LineTo(Vec2::new(77.5, -10.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(79.376564, -12.376565),
                        control2: Vec2::new(78.44139, -12.558608),
                        to: Vec2::new(76.5, -14.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3740196, 0.29803923, 0.48529416, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-72.5, -21.5)),
                    PathCommand::LineTo(Vec2::new(-68.5, -23.5)),
                    PathCommand::LineTo(Vec2::new(-72.5, -23.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.35735297, 0.30637255, 0.4955882, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(62.5, -33.5)),
                    PathCommand::LineTo(Vec2::new(65.5, -30.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(65.5, -32.914215),
                        control2: Vec2::new(65.73571, -31.264294),
                        to: Vec2::new(62.5, -34.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.402451, 0.31666666, 0.52696085, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(42.5, -59.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(45.7357, -56.2643),
                        control2: Vec2::new(44.08579, -56.5),
                        to: Vec2::new(46.5, -56.5),
                    },
                    PathCommand::LineTo(Vec2::new(43.5, -59.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4, 0.31617647, 0.5240196, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(34.5, -67.5)),
                    PathCommand::LineTo(Vec2::new(37.5, -64.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(37.5, -66.91422),
                        control2: Vec2::new(37.735714, -65.26428),
                        to: Vec2::new(34.5, -68.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40735292, 0.32107845, 0.5328431, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(99.5, 72.5)),
                    PathCommand::LineTo(Vec2::new(101.5, 73.5)),
                    PathCommand::LineTo(Vec2::new(101.5, 70.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.27675074, 0.23753504, 0.37871152, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(13.5, 64.5)),
                    PathCommand::LineTo(Vec2::new(19.5, 64.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.55910367, 0.46554622, 0.6733895, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-0.5, 56.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-0.015897006, 55.04769),
                        control2: Vec2::new(1.7198453, 52.5),
                        to: Vec2::new(-0.5, 52.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5456583, 0.55238104, 0.697479, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(96.5, 52.5)),
                    PathCommand::LineTo(Vec2::new(96.5, 46.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.34453785, 0.27955186, 0.45546222, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-27.5, 48.5)),
                    PathCommand::LineTo(Vec2::new(-26.5, 49.5)),
                    PathCommand::LineTo(Vec2::new(-23.5, 47.5)),
                    PathCommand::LineTo(Vec2::new(-24.5, 47.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.48011208, 0.40336138, 0.6207284, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-17.5, 47.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, 47.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-13.5, 46.166668),
                        control2: Vec2::new(-13.166667, 46.5),
                        to: Vec2::new(-14.5, 46.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4162465, 0.35238096, 0.5708684, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(5.5, 36.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(7.245357, 34.754642),
                        control2: Vec2::new(6.649065, 35.947193),
                        to: Vec2::new(5.5, 32.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.42184874, 0.4229692, 0.5775911, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(10.5, 20.5)),
                    PathCommand::LineTo(Vec2::new(16.5, 20.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.55182076, 0.56190485, 0.70588243, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-63.5, 12.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-61.649548, 14.3504505),
                        control2: Vec2::new(-61.5, 12.259331),
                        to: Vec2::new(-61.5, 10.5),
                    },
                    PathCommand::LineTo(Vec2::new(-63.5, 11.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.24257705, 0.21904764, 0.35910365, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-15.5, 6.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, 8.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-13.5, 6.0285997),
                        control2: Vec2::new(-13.028597, 6.9714026),
                        to: Vec2::new(-14.5, 5.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.50644267, 0.5114846, 0.65714294, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(4.5, 3.5)),
                    PathCommand::LineTo(Vec2::new(6.5, 4.5)),
                    PathCommand::LineTo(Vec2::new(4.5, 0.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23249301, 0.21120448, 0.34117648, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(21.5, 3.5)),
                    PathCommand::LineTo(Vec2::new(23.5, -0.5)),
                    PathCommand::LineTo(Vec2::new(21.5, 2.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23249301, 0.21176472, 0.34117648, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(7.5, 1.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(9.027045, 3.0270455),
                        control2: Vec2::new(8.078363, 2.7108185),
                        to: Vec2::new(10.5, 1.5),
                    },
                    PathCommand::CubicTo {
                        control1: Vec2::new(10.5, 0.16666663),
                        control2: Vec2::new(10.833333, 0.5),
                        to: Vec2::new(9.5, 0.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.37703082, 0.3271709, 0.5243698, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(80.5, -4.5)),
                    PathCommand::LineTo(Vec2::new(81.5, -2.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(82.711655, -4.923313),
                        control2: Vec2::new(83.48629, -5.5),
                        to: Vec2::new(80.5, -5.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.36526614, 0.2929972, 0.4745098, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-32.5, -7.5)),
                    PathCommand::LineTo(Vec2::new(-28.5, -7.5)),
                    PathCommand::LineTo(Vec2::new(-32.5, -8.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.34845942, 0.3036415, 0.49131656, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(45.5, -13.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(47.735703, -11.264297),
                        control2: Vec2::new(46.55719, -11.5),
                        to: Vec2::new(48.5, -11.5),
                    },
                    PathCommand::LineTo(Vec2::new(45.5, -14.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.26666668, 0.2386555, 0.39607847, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-15.5, -28.5)),
                    PathCommand::LineTo(Vec2::new(-11.5, -26.5)),
                    PathCommand::LineTo(Vec2::new(-14.5, -28.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.43081236, 0.37254903, 0.59383756, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(55.5, -44.5)),
                    PathCommand::LineTo(Vec2::new(57.5, -41.5)),
                    PathCommand::LineTo(Vec2::new(55.5, -45.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40784317, 0.3210084, 0.5338936, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-12.5, -65.5)),
                    PathCommand::LineTo(Vec2::new(-6.5, -65.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.21400563, 0.1971989, 0.3042017, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-108.5, 118.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-107.23879, 118.5),
                        control2: Vec2::new(-103.701454, 116.5),
                        to: Vec2::new(-106.5, 116.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9418301, 0.91699356, 0.45294118, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-104.5, 115.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-103.238785, 115.5),
                        control2: Vec2::new(-99.70148, 113.5),
                        to: Vec2::new(-102.5, 113.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9359478, 0.90326804, 0.4490196, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-70.5, 99.5)),
                    PathCommand::LineTo(Vec2::new(-66.5, 98.5)),
                    PathCommand::LineTo(Vec2::new(-68.5, 98.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.93202615, 0.9006536, 0.44509804, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-3.5, 51.5)),
                    PathCommand::LineTo(Vec2::new(-2.5, 52.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-2.5, 50.02861),
                        control2: Vec2::new(-2.028596, 50.971405),
                        to: Vec2::new(-3.5, 49.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.86339873, 0.8666667, 0.9666667, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-10.5, 38.5)),
                    PathCommand::LineTo(Vec2::new(-7.5, 39.5)),
                    PathCommand::LineTo(Vec2::new(-8.5, 37.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.24248368, 0.21830067, 0.36143792, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(31.5, 21.5)),
                    PathCommand::LineTo(Vec2::new(36.5, 21.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6843138, 0.6888889, 0.77647066, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(17.5, 20.5)),
                    PathCommand::LineTo(Vec2::new(22.5, 20.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.50522876, 0.51307195, 0.6464053, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(57.5, 15.5)),
                    PathCommand::LineTo(Vec2::new(59.5, 16.5)),
                    PathCommand::LineTo(Vec2::new(58.5, 13.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4575163, 0.39411768, 0.6215687, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(78.5, -8.5)),
                    PathCommand::LineTo(Vec2::new(80.5, -8.5)),
                    PathCommand::LineTo(Vec2::new(79.5, -10.5)),
                    PathCommand::LineTo(Vec2::new(78.5, -9.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.36666667, 0.29346406, 0.4764706, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-9.5, -50.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-8.350926, -48.20185),
                        control2: Vec2::new(-9.2453575, -48.5),
                        to: Vec2::new(-7.5, -48.5),
                    },
                    PathCommand::LineTo(Vec2::new(-8.5, -50.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6529412, 0.66405237, 0.7947712, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(15.5, -63.5)),
                    PathCommand::LineTo(Vec2::new(19.5, -62.5)),
                    PathCommand::LineTo(Vec2::new(18.5, -63.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.24379086, 0.21568629, 0.35751638, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(23.5, -78.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(25.735725, -76.264275),
                        control2: Vec2::new(24.55718, -76.5),
                        to: Vec2::new(26.5, -76.5),
                    },
                    PathCommand::LineTo(Vec2::new(24.5, -78.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41045755, 0.32352942, 0.53594774, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(19.5, -81.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(21.735662, -79.264336),
                        control2: Vec2::new(20.557207, -79.5),
                        to: Vec2::new(22.5, -79.5),
                    },
                    PathCommand::LineTo(Vec2::new(20.5, -81.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40457517, 0.32026145, 0.5300654, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(109.5, 108.5)),
                    PathCommand::LineTo(Vec2::new(110.5, 109.5)),
                    PathCommand::LineTo(Vec2::new(109.5, 106.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6760785, 0.46117648, 0.28392157, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-56.5, 94.5)),
                    PathCommand::LineTo(Vec2::new(-52.5, 94.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8980393, 0.8407844, 0.4164706, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-89.5, 92.5)),
                    PathCommand::LineTo(Vec2::new(-86.5, 91.5)),
                    PathCommand::LineTo(Vec2::new(-88.5, 91.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9270589, 0.89960784, 0.4407843, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(72.5, 86.5)),
                    PathCommand::LineTo(Vec2::new(73.5, 88.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(73.5, 86.0286),
                        control2: Vec2::new(73.971405, 86.971405),
                        to: Vec2::new(72.5, 85.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9231373, 0.88000005, 0.4117647, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(65.5, 83.5)),
                    PathCommand::LineTo(Vec2::new(69.5, 83.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9192157, 0.87764704, 0.41098043, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-14.5, 74.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-12.754639, 74.5),
                        control2: Vec2::new(-13.649086, 74.79817),
                        to: Vec2::new(-12.5, 72.5),
                    },
                    PathCommand::LineTo(Vec2::new(-13.5, 72.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8564707, 0.7686275, 0.34666666, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-51.5, 68.5)),
                    PathCommand::LineTo(Vec2::new(-47.5, 68.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.54588234, 0.45411763, 0.667451, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(36.5, 66.5)),
                    PathCommand::LineTo(Vec2::new(40.5, 66.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.55529416, 0.45803925, 0.6682353, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-35.5, 51.5)),
                    PathCommand::LineTo(Vec2::new(-32.5, 50.5)),
                    PathCommand::LineTo(Vec2::new(-34.5, 50.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.48156863, 0.40235296, 0.6211765, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(95.5, 45.5)),
                    PathCommand::LineTo(Vec2::new(95.5, 41.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.35607848, 0.28549024, 0.46666667, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-96.5, 43.5)),
                    PathCommand::LineTo(Vec2::new(-96.5, 39.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5545098, 0.46039215, 0.66431373, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(36.5, 20.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(37.971405, 21.971405),
                        control2: Vec2::new(37.0286, 21.5),
                        to: Vec2::new(39.5, 21.5),
                    },
                    PathCommand::LineTo(Vec2::new(37.5, 20.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23137255, 0.2101961, 0.34274513, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(5.5, 20.5)),
                    PathCommand::LineTo(Vec2::new(9.5, 20.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.614902, 0.6266667, 0.7858824, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-14.5, 13.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, 13.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, 11.5)),
                    PathCommand::LineTo(Vec2::new(-14.5, 12.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5317647, 0.53490204, 0.6705882, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-61.5, 9.5)),
                    PathCommand::LineTo(Vec2::new(-60.5, 11.5)),
                    PathCommand::LineTo(Vec2::new(-60.5, 8.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41098043, 0.3647059, 0.5592157, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-20.5, 7.5)),
                    PathCommand::LineTo(Vec2::new(-19.5, 8.5)),
                    PathCommand::LineTo(Vec2::new(-19.5, 6.5)),
                    PathCommand::LineTo(Vec2::new(-20.5, 6.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7105883, 0.7168628, 0.8313727, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(84.5, 3.5)),
                    PathCommand::LineTo(Vec2::new(85.5, 5.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(85.5, 3.028578),
                        control2: Vec2::new(85.97142, 3.971417),
                        to: Vec2::new(84.5, 2.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.38588235, 0.3058824, 0.50431377, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(82.5, -0.5)),
                    PathCommand::LineTo(Vec2::new(83.5, 1.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(83.5, -0.9714105),
                        control2: Vec2::new(83.971405, -0.028591871),
                        to: Vec2::new(82.5, -1.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.38431373, 0.30509806, 0.49960786, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-21.5, -17.5)),
                    PathCommand::LineTo(Vec2::new(-21.5, -21.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.3121569, 0.26980394, 0.45960784, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-62.5, -33.5)),
                    PathCommand::LineTo(Vec2::new(-61.5, -36.5)),
                    PathCommand::LineTo(Vec2::new(-62.5, -35.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.22431374, 0.20470588, 0.32705885, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-17.5, -35.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, -35.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.227451, 0.20784314, 0.3254902, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-60.5, -42.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-58.75465, -42.5),
                        control2: Vec2::new(-59.64907, -42.20186),
                        to: Vec2::new(-58.5, -44.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23843138, 0.21411765, 0.34352943, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-12.5, -52.5)),
                    PathCommand::LineTo(Vec2::new(-9.5, -51.5)),
                    PathCommand::LineTo(Vec2::new(-10.5, -52.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.58274513, 0.59450984, 0.7301961, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-50.5, -52.5)),
                    PathCommand::CubicTo {
                        control1: Vec2::new(-48.754654, -52.5),
                        control2: Vec2::new(-49.649063, -52.201874),
                        to: Vec2::new(-48.5, -54.5),
                    },
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33411765, 0.2729412, 0.43843135, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(10.5, -64.5)),
                    PathCommand::LineTo(Vec2::new(14.5, -64.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.24000001, 0.21333335, 0.3529412, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(29.5, -72.5)),
                    PathCommand::LineTo(Vec2::new(31.5, -71.5)),
                    PathCommand::LineTo(Vec2::new(29.5, -73.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.39686275, 0.3145098, 0.51843137, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(7.5, -88.5)),
                    PathCommand::LineTo(Vec2::new(10.5, -87.5)),
                    PathCommand::LineTo(Vec2::new(9.5, -88.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.39921567, 0.31607842, 0.52156866, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-111.5, 120.5)),
                    PathCommand::LineTo(Vec2::new(-109.5, 119.5)),
                    PathCommand::LineTo(Vec2::new(-110.5, 119.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9401961, 0.9147059, 0.45294118, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(105.5, 114.5)),
                    PathCommand::LineTo(Vec2::new(106.5, 115.5)),
                    PathCommand::LineTo(Vec2::new(105.5, 113.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.87843144, 0.8088235, 0.3735294, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-101.5, 100.5)),
                    PathCommand::LineTo(Vec2::new(-99.5, 99.5)),
                    PathCommand::LineTo(Vec2::new(-100.5, 99.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.93921566, 0.9078432, 0.4490196, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-97.5, 97.5)),
                    PathCommand::LineTo(Vec2::new(-95.5, 96.5)),
                    PathCommand::LineTo(Vec2::new(-96.5, 96.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.94803923, 0.92647064, 0.45784315, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-60.5, 95.5)),
                    PathCommand::LineTo(Vec2::new(-57.5, 95.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.90588236, 0.85490197, 0.42254904, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-51.5, 93.5)),
                    PathCommand::LineTo(Vec2::new(-48.5, 93.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.87745094, 0.8019608, 0.39901963, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-32.5, 90.5)),
                    PathCommand::LineTo(Vec2::new(-29.5, 90.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8264706, 0.7137255, 0.3509804, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-99.5, 89.5)),
                    PathCommand::LineTo(Vec2::new(-97.5, 88.5)),
                    PathCommand::LineTo(Vec2::new(-99.5, 88.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.58431375, 0.48333332, 0.6950981, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-94.5, 86.5)),
                    PathCommand::LineTo(Vec2::new(-92.5, 85.5)),
                    PathCommand::LineTo(Vec2::new(-93.5, 85.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5568628, 0.46176472, 0.6666667, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-108.5, 83.5)),
                    PathCommand::LineTo(Vec2::new(-108.5, 80.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.83137256, 0.7254902, 0.34313726, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-74.5, 81.5)),
                    PathCommand::LineTo(Vec2::new(-72.5, 82.5)),
                    PathCommand::LineTo(Vec2::new(-73.5, 81.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41568628, 0.31960785, 0.6431373, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-54.5, 78.5)),
                    PathCommand::LineTo(Vec2::new(-51.5, 78.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.79509807, 0.66176474, 0.30098042, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(71.5, 77.5)),
                    PathCommand::LineTo(Vec2::new(73.5, 78.5)),
                    PathCommand::LineTo(Vec2::new(72.5, 77.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5029412, 0.42156863, 0.62941176, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(42.5, 67.5)),
                    PathCommand::LineTo(Vec2::new(45.5, 67.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.5411765, 0.45, 0.65882355, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(12.5, 63.5)),
                    PathCommand::LineTo(Vec2::new(15.5, 63.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.41568628, 0.3529412, 0.56078434, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-1.5, 59.5)),
                    PathCommand::LineTo(Vec2::new(-1.5, 56.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8303922, 0.827451, 0.9, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-97.5, 48.5)),
                    PathCommand::LineTo(Vec2::new(-97.5, 45.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.56078434, 0.46470588, 0.67058825, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(16.5, 44.5)),
                    PathCommand::LineTo(Vec2::new(18.5, 45.5)),
                    PathCommand::LineTo(Vec2::new(18.5, 44.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33137256, 0.29313728, 0.46078432, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-25.5, 41.5)),
                    PathCommand::LineTo(Vec2::new(-24.5, 39.5)),
                    PathCommand::LineTo(Vec2::new(-25.5, 40.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.25686276, 0.22941177, 0.3882353, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-95.5, 38.5)),
                    PathCommand::LineTo(Vec2::new(-95.5, 35.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.53431374, 0.44607845, 0.6450981, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-19.5, 38.5)),
                    PathCommand::LineTo(Vec2::new(-18.5, 36.5)),
                    PathCommand::LineTo(Vec2::new(-19.5, 37.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.78431374, 0.80196077, 0.99215686, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-9.5, 34.5)),
                    PathCommand::LineTo(Vec2::new(-8.5, 36.5)),
                    PathCommand::LineTo(Vec2::new(-8.5, 34.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.6882353, 0.69705886, 0.89509803, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-14.5, 32.5)),
                    PathCommand::LineTo(Vec2::new(-12.5, 31.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, 31.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.8382353, 0.8519608, 1.0, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(17.5, 25.5)),
                    PathCommand::LineTo(Vec2::new(20.5, 25.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.67745095, 0.68725497, 0.78627455, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(25.5, 23.5)),
                    PathCommand::LineTo(Vec2::new(28.5, 23.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.7009804, 0.70490193, 0.7931373, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-26.5, 16.5)),
                    PathCommand::LineTo(Vec2::new(-23.5, 16.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.25980392, 0.23725492, 0.38627452, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(87.5, 13.5)),
                    PathCommand::LineTo(Vec2::new(87.5, 10.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.39313725, 0.3107843, 0.51274514, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(86.5, 9.5)),
                    PathCommand::LineTo(Vec2::new(86.5, 6.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.39607844, 0.3127451, 0.5156863, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(6.5, 2.5)),
                    PathCommand::LineTo(Vec2::new(7.5, 2.5)),
                    PathCommand::LineTo(Vec2::new(6.5, 0.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.49313727, 0.41862747, 0.62352943, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-17.5, -0.5)),
                    PathCommand::LineTo(Vec2::new(-15.5, 0.5)),
                    PathCommand::LineTo(Vec2::new(-16.5, -0.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.2784314, 0.24411765, 0.41862747, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(3.5, -0.5)),
                    PathCommand::LineTo(Vec2::new(3.5, -3.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.23725492, 0.2127451, 0.34509805, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(82.5, -2.5)),
                    PathCommand::LineTo(Vec2::new(83.5, -1.5)),
                    PathCommand::LineTo(Vec2::new(83.5, -3.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.29509807, 0.24411765, 0.377451, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-81.5, -2.5)),
                    PathCommand::LineTo(Vec2::new(-80.5, -2.5)),
                    PathCommand::LineTo(Vec2::new(-81.5, -4.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.25490198, 0.22647059, 0.36176473, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(51.5, -7.5)),
                    PathCommand::LineTo(Vec2::new(52.5, -6.5)),
                    PathCommand::LineTo(Vec2::new(51.5, -8.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.9794118, 0.96862745, 1.0, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(46.5, -10.5)),
                    PathCommand::LineTo(Vec2::new(47.5, -9.5)),
                    PathCommand::LineTo(Vec2::new(46.5, -11.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.33333334, 0.2901961, 0.47058824, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(71.5, -21.5)),
                    PathCommand::LineTo(Vec2::new(72.5, -20.5)),
                    PathCommand::LineTo(Vec2::new(71.5, -22.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40098038, 0.31764707, 0.5235294, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(68.5, -26.5)),
                    PathCommand::LineTo(Vec2::new(69.5, -25.5)),
                    PathCommand::LineTo(Vec2::new(68.5, -27.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40588236, 0.31960785, 0.5284314, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(53.5, -47.5)),
                    PathCommand::LineTo(Vec2::new(54.5, -46.5)),
                    PathCommand::LineTo(Vec2::new(53.5, -48.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40588236, 0.31862748, 0.53039217, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(51.5, -50.5)),
                    PathCommand::LineTo(Vec2::new(52.5, -49.5)),
                    PathCommand::LineTo(Vec2::new(51.5, -51.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.4009804, 0.3156863, 0.52156866, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-16.5, -64.5)),
                    PathCommand::LineTo(Vec2::new(-13.5, -64.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.20784314, 0.19215687, 0.29411766, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(16.5, -83.5)),
                    PathCommand::LineTo(Vec2::new(18.5, -82.5)),
                    PathCommand::LineTo(Vec2::new(17.5, -83.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40686274, 0.32058823, 0.53333336, 1.0),
        },
        Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(13.5, -85.5)),
                    PathCommand::LineTo(Vec2::new(15.5, -84.5)),
                    PathCommand::LineTo(Vec2::new(14.5, -85.5)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(0.40980393, 0.32352942, 0.5362745, 1.0),
        },
    ]
}
