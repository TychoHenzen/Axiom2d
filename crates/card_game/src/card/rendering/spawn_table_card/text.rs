// EVOLVE-BLOCK-START
pub(crate) const TEXT_COLOR: engine_core::color::Color = engine_core::color::Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

/// Find the largest font size where the name wraps to at most 2 lines
/// and fits within both `max_width` and `max_height`.
pub(crate) fn fit_name_font_size(
    text: &str,
    base_size: f32,
    max_width: f32,
    max_height: f32,
) -> f32 {
    let full_width = engine_render::font::measure_text(text, base_size);
    if full_width <= max_width {
        return base_size;
    }

    let words: Vec<&str> = text.split(' ').collect();
    if words.len() <= 1 {
        return base_size * max_width / full_width;
    }

    let mut best_max_half = full_width;
    for split in 1..words.len() {
        let line1 = words[..split].join(" ");
        let line2 = words[split..].join(" ");
        let w1 = engine_render::font::measure_text(&line1, base_size);
        let w2 = engine_render::font::measure_text(&line2, base_size);
        let wider = w1.max(w2);
        if wider < best_max_half {
            best_max_half = wider;
        }
    }

    let width_size = if best_max_half > max_width {
        base_size * max_width / best_max_half
    } else {
        base_size
    };

    let two_line_height = width_size * 1.3 * 2.0;
    if two_line_height <= max_height {
        width_size
    } else {
        width_size * max_height / two_line_height
    }
}
// EVOLVE-BLOCK-END
