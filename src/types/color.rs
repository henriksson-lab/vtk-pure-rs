//! Color conversion utilities.

/// Convert RGB [0,1] to HSV [0,360], [0,1], [0,1].
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max > 0.0 { delta / max } else { 0.0 };

    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Convert HSV [0,360], [0,1], [0,1] to RGB [0,1].
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - ((h2 % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h2 < 1.0 {
        (c, x, 0.0)
    } else if h2 < 2.0 {
        (x, c, 0.0)
    } else if h2 < 3.0 {
        (0.0, c, x)
    } else if h2 < 4.0 {
        (0.0, x, c)
    } else if h2 < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

/// Parse a hex color string (#RRGGBB or RRGGBB) to RGB [0,1].
pub fn hex_to_rgb(hex: &str) -> Option<[f32; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some([r, g, b])
}

/// Convert RGB [0,1] to hex string (#RRGGBB).
pub fn rgb_to_hex(r: f32, g: f32, b: f32) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    )
}

/// Linearly interpolate between two RGB colors.
pub fn lerp_rgb(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

/// Compute luminance of an RGB color (perceptual brightness).
pub fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Compute contrast ratio between two colors (WCAG formula).
pub fn contrast_ratio(lum_a: f32, lum_b: f32) -> f32 {
    let lighter = lum_a.max(lum_b) + 0.05;
    let darker = lum_a.min(lum_b) + 0.05;
    lighter / darker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_hsv_roundtrip() {
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h).abs() < 1.0); // red = 0°
        assert!((s - 1.0).abs() < 1e-5);
        assert!((v - 1.0).abs() < 1e-5);

        let (r, g, b) = hsv_to_rgb(h, s, v);
        assert!((r - 1.0).abs() < 1e-4);
        assert!(g.abs() < 1e-4);
    }

    #[test]
    fn green_hsv() {
        let (h, _, _) = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h - 120.0).abs() < 1.0);
    }

    #[test]
    fn hex_conversions() {
        let rgb = hex_to_rgb("#FF0000").unwrap();
        assert!((rgb[0] - 1.0).abs() < 0.01);
        assert!(rgb[1].abs() < 0.01);

        let hex = rgb_to_hex(1.0, 0.0, 0.0);
        assert_eq!(hex, "#FF0000");
    }

    #[test]
    fn hex_no_hash() {
        let rgb = hex_to_rgb("00FF00").unwrap();
        assert!((rgb[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn lerp() {
        let c = lerp_rgb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.5);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn luminance_test() {
        assert!(luminance(1.0, 1.0, 1.0) > luminance(0.0, 0.0, 0.0));
        assert!(luminance(1.0, 1.0, 1.0) > 0.9);
    }
}
