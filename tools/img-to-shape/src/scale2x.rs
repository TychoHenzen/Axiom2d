/// Apply the EPX/Scale2x algorithm to an RGBA pixel buffer.
///
/// Returns a new buffer at exactly 2x the input dimensions.
#[allow(clippy::many_single_char_names)]
pub fn scale2x_rgba(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    let out_w = w * 2;
    let out_h = h * 2;
    let mut out = vec![0u8; out_w * out_h * 4];

    let px = |x: usize, y: usize| -> &[u8] { &rgba[(y * w + x) * 4..(y * w + x) * 4 + 4] };

    for y in 0..h {
        for x in 0..w {
            let p = px(x, y);
            let a = if y > 0 { px(x, y - 1) } else { p };
            let b = if x > 0 { px(x - 1, y) } else { p };
            let c = if x + 1 < w { px(x + 1, y) } else { p };
            let d = if y + 1 < h { px(x, y + 1) } else { p };

            let sharpen = a != d && b != c;
            let e1 = if sharpen && a == b { a } else { p };
            let e2 = if sharpen && a == c { a } else { p };
            let e3 = if sharpen && d == b { d } else { p };
            let e4 = if sharpen && d == c { d } else { p };

            let ox = x * 2;
            let oy = y * 2;
            out[(oy * out_w + ox) * 4..(oy * out_w + ox) * 4 + 4].copy_from_slice(e1);
            out[(oy * out_w + ox + 1) * 4..(oy * out_w + ox + 1) * 4 + 4].copy_from_slice(e2);
            out[((oy + 1) * out_w + ox) * 4..((oy + 1) * out_w + ox) * 4 + 4].copy_from_slice(e3);
            out[((oy + 1) * out_w + ox + 1) * 4..((oy + 1) * out_w + ox + 1) * 4 + 4]
                .copy_from_slice(e4);
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];

    fn pixel_at(buf: &[u8], width: usize, x: usize, y: usize) -> &[u8] {
        let i = (y * width + x) * 4;
        &buf[i..i + 4]
    }

    #[test]
    fn when_single_pixel_then_four_identical_copies() {
        // Arrange
        let pixel = [100u8, 150, 200, 255];

        // Act
        let out = scale2x_rgba(&pixel, 1, 1);

        // Assert
        assert_eq!(out.len(), 16);
        for i in 0..4 {
            assert_eq!(&out[i * 4..(i + 1) * 4], &pixel);
        }
    }

    #[test]
    fn when_uniform_color_then_no_artifacts() {
        // Arrange — 2x2 all red
        let rgba: Vec<u8> = RED.iter().cycle().take(4 * 4).copied().collect();

        // Act
        let out = scale2x_rgba(&rgba, 2, 2);

        // Assert — 4x4 all red
        assert_eq!(out.len(), 4 * 4 * 4);
        for i in 0..16 {
            assert_eq!(&out[i * 4..(i + 1) * 4], &RED, "pixel {i} differs");
        }
    }

    #[test]
    fn when_vertical_edge_then_no_sharpening() {
        // Arrange — 2x2: left column red, right column blue
        let mut rgba = vec![0u8; 2 * 2 * 4];
        rgba[0..4].copy_from_slice(&RED);
        rgba[4..8].copy_from_slice(&BLUE);
        rgba[8..12].copy_from_slice(&RED);
        rgba[12..16].copy_from_slice(&BLUE);

        // Act
        let out = scale2x_rgba(&rgba, 2, 2);

        // Assert — 4x4: left 2 columns red, right 2 columns blue
        for y in 0..4 {
            for x in 0..4 {
                let expected = if x < 2 { &RED } else { &BLUE };
                assert_eq!(pixel_at(&out, 4, x, y), expected, "pixel ({x},{y}) wrong");
            }
        }
    }

    #[test]
    fn when_diagonal_corner_then_epx_sharpening_applies() {
        // Arrange — 3x3 image:
        //   R R B     (row 0)
        //   R R B     (row 1)
        //   B B B     (row 2)
        // Center pixel (1,1): A=R, B=R, C=B, D=B
        // EPX: sharpen = (A!=D && B!=C) = true
        //   E1 = A==B ? A : P = R (correct, A==B=R)
        //   E2 = A==C ? A : P = P=R (A=R != C=B, so P=R)
        //   E3 = D==B ? D : P = P=R (D=B != B=R, so P=R)
        //   E4 = D==C ? D : P = D=B (D=B == C=B)
        let mut rgba = vec![0u8; 3 * 3 * 4];
        for (i, color) in [
            RED, RED, BLUE, // row 0
            RED, RED, BLUE, // row 1
            BLUE, BLUE, BLUE, // row 2
        ]
        .iter()
        .enumerate()
        {
            rgba[i * 4..(i + 1) * 4].copy_from_slice(color);
        }

        // Act
        let out = scale2x_rgba(&rgba, 3, 3);

        // Assert — check the 2x2 block for center pixel (1,1) → output (2,2)-(3,3)
        assert_eq!(pixel_at(&out, 6, 2, 2), &RED, "E1 should be A=red");
        assert_eq!(pixel_at(&out, 6, 3, 2), &RED, "E2 should be P=red");
        assert_eq!(pixel_at(&out, 6, 2, 3), &RED, "E3 should be P=red");
        assert_eq!(pixel_at(&out, 6, 3, 3), &BLUE, "E4 should be D=blue");
    }
}
