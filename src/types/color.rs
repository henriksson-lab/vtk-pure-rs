//! Color conversion utilities.

/// Convert RGB [0,1] to VTK HSV [0,1], [0,1], [0,1].
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let mut cmax = r;
    let mut cmin = r;
    if g > cmax {
        cmax = g;
    } else if g < cmin {
        cmin = g;
    }
    if b > cmax {
        cmax = b;
    } else if b < cmin {
        cmin = b;
    }

    let v = cmax;
    let s = if v > 0.0 { (cmax - cmin) / cmax } else { 0.0 };

    let h = if s > 0.0 {
        let mut h = if r == cmax {
            (1.0 / 6.0) * (g - b) / (cmax - cmin)
        } else if g == cmax {
            (1.0 / 3.0) + (1.0 / 6.0) * (b - r) / (cmax - cmin)
        } else {
            (2.0 / 3.0) + (1.0 / 6.0) * (r - g) / (cmax - cmin)
        };
        if h < 0.0 {
            h += 1.0;
        }
        h
    } else {
        0.0
    };

    (h, s, v)
}

/// Convert VTK HSV [0,1], [0,1], [0,1] to RGB [0,1].
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let onethird = 1.0 / 3.0;
    let onesixth = 1.0 / 6.0;
    let twothird = 2.0 / 3.0;
    let fivesixth = 5.0 / 6.0;

    let (mut r, mut g, mut b) = if h > onesixth && h <= onethird {
        ((onethird - h) / onesixth, 1.0, 0.0)
    } else if h > onethird && h <= 0.5 {
        (0.0, 1.0, (h - onethird) / onesixth)
    } else if h > 0.5 && h <= twothird {
        (0.0, (twothird - h) / onesixth, 1.0)
    } else if h > twothird && h <= fivesixth {
        ((h - twothird) / onesixth, 0.0, 1.0)
    } else if h > fivesixth && h <= 1.0 {
        (1.0, 0.0, (1.0 - h) / onesixth)
    } else {
        (1.0, h / onesixth, 0.0)
    };

    r = s * r + (1.0 - s);
    g = s * g + (1.0 - s);
    b = s * b + (1.0 - s);

    (r * v, g * v, b * v)
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
        assert!((h).abs() < 1e-5); // red = 0 in VTK's normalized hue
        assert!((s - 1.0).abs() < 1e-5);
        assert!((v - 1.0).abs() < 1e-5);

        let (r, g, b) = hsv_to_rgb(h, s, v);
        assert!((r - 1.0).abs() < 1e-4);
        assert!(g.abs() < 1e-4);
        assert!(b.abs() < 1e-4);
    }

    #[test]
    fn green_hsv() {
        let (h, _, _) = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h - (1.0 / 3.0)).abs() < 1e-5);
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
