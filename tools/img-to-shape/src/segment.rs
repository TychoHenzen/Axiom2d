use engine_core::color::Color;

pub struct Region {
    pub mask: Vec<bool>,
    pub color: Color,
}

fn pixel_color(rgba: &[u8], index: usize) -> Color {
    let base = index * 4;
    Color::from_u8(rgba[base], rgba[base + 1], rgba[base + 2], rgba[base + 3])
}

fn color_distance(a: Color, b: Color) -> f32 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    (dr * dr + dg * dg + db * db).sqrt()
}

pub fn segment(
    rgba: &[u8],
    width: u32,
    height: u32,
    color_threshold: f32,
    alpha_threshold: u8,
) -> Vec<Region> {
    let pixel_count = (width * height) as usize;
    let mut visited = vec![false; pixel_count];
    let mut regions = Vec::new();

    for start in 0..pixel_count {
        if visited[start] {
            continue;
        }
        let start_color = pixel_color(rgba, start);
        if rgba[start * 4 + 3] < alpha_threshold {
            visited[start] = true;
            continue;
        }

        let mut mask = vec![false; pixel_count];
        let mut stack = vec![start];
        let mut sum_r = 0.0_f32;
        let mut sum_g = 0.0_f32;
        let mut sum_b = 0.0_f32;
        let mut count = 0usize;

        while let Some(idx) = stack.pop() {
            if visited[idx] {
                continue;
            }
            let c = pixel_color(rgba, idx);
            if rgba[idx * 4 + 3] < alpha_threshold
                || color_distance(c, start_color) > color_threshold
            {
                continue;
            }
            visited[idx] = true;
            mask[idx] = true;
            sum_r += c.r;
            sum_g += c.g;
            sum_b += c.b;
            count += 1;

            let x = (idx as u32) % width;
            let y = (idx as u32) / width;
            if x > 0 {
                stack.push(idx - 1);
            }
            if x + 1 < width {
                stack.push(idx + 1);
            }
            if y > 0 {
                stack.push(idx - width as usize);
            }
            if y + 1 < height {
                stack.push(idx + width as usize);
            }
        }

        if count > 0 {
            let inv = 1.0 / count as f32;
            regions.push(Region {
                mask,
                color: Color::new(sum_r * inv, sum_g * inv, sum_b * inv, 1.0),
            });
        }
    }

    regions
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::segment;

    #[test]
    fn when_single_color_opaque_image_segmented_then_one_region_covers_all_pixels() {
        // Arrange
        #[rustfmt::skip]
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,  255, 0, 0, 255,
            255, 0, 0, 255,  255, 0, 0, 255,
        ];

        // Act
        let regions = segment(&rgba, 2, 2, 0.01, 128);

        // Assert
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].mask, vec![true, true, true, true]);
    }

    #[test]
    fn when_two_color_halves_segmented_then_masks_partition_all_pixels() {
        // Arrange
        #[rustfmt::skip]
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,
            255, 0, 0, 255,  255, 0, 0, 255,  0, 0, 255, 255,  0, 0, 255, 255,
        ];

        // Act
        let regions = segment(&rgba, 4, 2, 0.1, 128);

        // Assert
        assert_eq!(regions.len(), 2);
        let coverage: Vec<usize> = (0..8)
            .map(|i| regions.iter().filter(|r| r.mask[i]).count())
            .collect();
        assert!(
            coverage.iter().all(|&c| c == 1),
            "every pixel must belong to exactly one region"
        );
    }

    #[test]
    fn when_similar_colors_within_threshold_then_merged_into_one_region() {
        // Arrange — two slightly different reds (250 vs 255), threshold wide enough to merge
        #[rustfmt::skip]
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,  250, 5, 0, 255,
        ];

        // Act
        let regions = segment(&rgba, 2, 1, 0.1, 128);

        // Assert
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].mask, vec![true, true]);
    }

    #[test]
    fn when_similar_colors_exceed_threshold_then_separate_regions() {
        // Arrange — red vs green, threshold too tight to merge
        #[rustfmt::skip]
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,  0, 255, 0, 255,
        ];

        // Act
        let regions = segment(&rgba, 2, 1, 0.01, 128);

        // Assert
        assert_eq!(regions.len(), 2);
    }

    #[test]
    fn when_pixels_below_alpha_threshold_then_excluded_from_all_regions() {
        // Arrange — one opaque red, one transparent red
        #[rustfmt::skip]
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,  255, 0, 0, 50,
        ];

        // Act
        let regions = segment(&rgba, 2, 1, 0.1, 128);

        // Assert
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].mask[0], true);
        assert_eq!(regions[0].mask[1], false);
    }

    #[test]
    fn when_region_segmented_then_color_is_average_of_member_pixels() {
        // Arrange — two pixels: (200,100,0) and (100,200,0), threshold wide enough to merge
        #[rustfmt::skip]
        let rgba: Vec<u8> = vec![
            200, 100, 0, 255,  100, 200, 0, 255,
        ];

        // Act
        let regions = segment(&rgba, 2, 1, 1.0, 128);

        // Assert
        assert_eq!(regions.len(), 1);
        let c = regions[0].color;
        let expected_r = (200.0 / 255.0 + 100.0 / 255.0) / 2.0;
        let expected_g = (100.0 / 255.0 + 200.0 / 255.0) / 2.0;
        assert!(
            (c.r - expected_r).abs() < 1e-3,
            "r: expected {expected_r}, got {}",
            c.r
        );
        assert!(
            (c.g - expected_g).abs() < 1e-3,
            "g: expected {expected_g}, got {}",
            c.g
        );
        assert!(c.b.abs() < 1e-3);
    }
}
