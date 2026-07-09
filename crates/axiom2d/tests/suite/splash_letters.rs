#![allow(clippy::unwrap_used)]

use axiom2d::prelude::*;

/// @doc: Verifies all five "AXIOM" letter shape functions exist and return non-empty path data.
#[test]
fn when_letters_defined_then_all_five_present() {
    // Arrange & Act
    let a = axiom2d::splash::letter_a();
    let x = axiom2d::splash::letter_x();
    let i = axiom2d::splash::letter_i();
    let o = axiom2d::splash::letter_o();
    let m = axiom2d::splash::letter_m();

    // Assert
    assert!(!a.is_empty(), "letter_a should have at least one command");
    assert!(!x.is_empty(), "letter_x should have at least one command");
    assert!(!i.is_empty(), "letter_i should have at least one command");
    assert!(!o.is_empty(), "letter_o should have at least one command");
    assert!(!m.is_empty(), "letter_m should have at least one command");
}

/// @doc: Verifies each letter shape opens with a `MoveTo` command, confirming valid contour start.
#[test]
fn when_letter_shapes_then_non_empty_vertices() {
    // Arrange
    let letters = [
        ('A', axiom2d::splash::letter_a()),
        ('X', axiom2d::splash::letter_x()),
        ('I', axiom2d::splash::letter_i()),
        ('O', axiom2d::splash::letter_o()),
        ('M', axiom2d::splash::letter_m()),
    ];

    // Act & Assert
    for (name, commands) in &letters {
        assert!(
            !commands.is_empty(),
            "letter {name} should have non-empty commands"
        );
        assert!(
            matches!(commands.first(), Some(PathCommand::MoveTo(_))),
            "letter {name} should start with MoveTo, got {first:?}",
            first = commands.first()
        );
    }
}

/// @doc: Verifies the letter-associated colors have all RGBA components in valid [0, 1] range.
#[test]
fn when_letter_colors_then_all_valid_colors() {
    // Arrange
    let colors: [(&str, Color); 2] = [
        ("LOGO_COLOR", axiom2d::splash::LOGO_COLOR),
        ("ACCENT_COLOR", axiom2d::splash::ACCENT_COLOR),
    ];

    // Act & Assert
    for (name, color) in &colors {
        assert!(
            (0.0..=1.0).contains(&color.r),
            "{name}.r = {} is outside [0, 1]",
            color.r
        );
        assert!(
            (0.0..=1.0).contains(&color.g),
            "{name}.g = {} is outside [0, 1]",
            color.g
        );
        assert!(
            (0.0..=1.0).contains(&color.b),
            "{name}.b = {} is outside [0, 1]",
            color.b
        );
        assert!(
            (0.0..=1.0).contains(&color.a),
            "{name}.a = {} is outside [0, 1]",
            color.a
        );
    }
}
