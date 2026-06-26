use crate::data::{CellArray, Points, PolyData};

/// Parameters for generating 3D text geometry.
pub struct Text3dParams {
    /// The text string. Default: "VTK"
    pub text: String,
    /// Height of each character. Default: 1.0
    pub height: f64,
    /// Depth of extrusion. Default: 0.2 (0 = flat)
    pub depth: f64,
    /// Origin (bottom-left of first character). Default: [0,0,0]
    pub origin: [f64; 3],
}

impl Default for Text3dParams {
    fn default() -> Self {
        Self {
            text: "VTK".to_string(),
            height: 1.0,
            depth: 0.2,
            origin: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate 3D text geometry using a simple built-in font.
///
/// Characters are rendered as polyline strokes extruded to the given depth.
/// Only uppercase letters, digits, and a few symbols are supported.
/// Returns PolyData with quad cells (if depth > 0) or line cells (if depth == 0).
pub fn text_3d(params: &Text3dParams) -> PolyData {
    let scale = params.height;
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();

    let mut cursor_x = params.origin[0];
    let char_width = scale * 0.7;
    let spacing = scale * 0.15;

    for ch in params.text.chars() {
        let strokes = char_strokes(ch);
        for stroke in &strokes {
            if stroke.len() < 2 {
                continue;
            }

            if params.depth > 0.0 {
                // Extruded text: create quad strip
                for i in 0..stroke.len() - 1 {
                    let (x0, y0) = stroke[i];
                    let (x1, y1) = stroke[i + 1];

                    let p0 = [
                        cursor_x + x0 * scale,
                        params.origin[1] + y0 * scale,
                        params.origin[2],
                    ];
                    let p1 = [
                        cursor_x + x1 * scale,
                        params.origin[1] + y1 * scale,
                        params.origin[2],
                    ];
                    let p2 = [
                        cursor_x + x1 * scale,
                        params.origin[1] + y1 * scale,
                        params.origin[2] + params.depth,
                    ];
                    let p3 = [
                        cursor_x + x0 * scale,
                        params.origin[1] + y0 * scale,
                        params.origin[2] + params.depth,
                    ];

                    let base = points.len() as i64;
                    points.push(p0);
                    points.push(p1);
                    points.push(p2);
                    points.push(p3);
                    polys.push_cell(&[base, base + 1, base + 2, base + 3]);
                }
            } else {
                // Flat text: create line
                let _base = points.len() as i64;
                let mut ids = Vec::new();
                for &(x, y) in stroke {
                    let idx = points.len() as i64;
                    points.push([
                        cursor_x + x * scale,
                        params.origin[1] + y * scale,
                        params.origin[2],
                    ]);
                    ids.push(idx);
                }
                lines.push_cell(&ids);
            }
        }
        cursor_x += char_width + spacing;
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.lines = lines;
    pd
}

/// Simple vector font: returns strokes for a character.
/// Each stroke is a sequence of (x, y) in [0,0.6] × [0,1] space.
fn char_strokes(ch: char) -> Vec<Vec<(f64, f64)>> {
    match ch.to_ascii_uppercase() {
        'A' => vec![
            vec![(0.0, 0.0), (0.3, 1.0), (0.6, 0.0)],
            vec![(0.12, 0.4), (0.48, 0.4)],
        ],
        'B' => vec![
            vec![
                (0.0, 0.0),
                (0.0, 1.0),
                (0.45, 1.0),
                (0.55, 0.85),
                (0.55, 0.65),
                (0.45, 0.5),
                (0.0, 0.5),
            ],
            vec![
                (0.0, 0.5),
                (0.45, 0.5),
                (0.6, 0.35),
                (0.6, 0.15),
                (0.45, 0.0),
                (0.0, 0.0),
            ],
        ],
        'C' => vec![vec![
            (0.6, 0.85),
            (0.45, 1.0),
            (0.15, 1.0),
            (0.0, 0.85),
            (0.0, 0.15),
            (0.15, 0.0),
            (0.45, 0.0),
            (0.6, 0.15),
        ]],
        'D' => vec![vec![
            (0.0, 0.0),
            (0.0, 1.0),
            (0.4, 1.0),
            (0.6, 0.8),
            (0.6, 0.2),
            (0.4, 0.0),
            (0.0, 0.0),
        ]],
        'E' => vec![
            vec![(0.6, 1.0), (0.0, 1.0), (0.0, 0.0), (0.6, 0.0)],
            vec![(0.0, 0.5), (0.45, 0.5)],
        ],
        'F' => vec![
            vec![(0.6, 1.0), (0.0, 1.0), (0.0, 0.0)],
            vec![(0.0, 0.5), (0.45, 0.5)],
        ],
        'G' => vec![vec![
            (0.6, 0.85),
            (0.45, 1.0),
            (0.15, 1.0),
            (0.0, 0.85),
            (0.0, 0.15),
            (0.15, 0.0),
            (0.45, 0.0),
            (0.6, 0.15),
            (0.6, 0.4),
            (0.35, 0.4),
        ]],
        'H' => vec![
            vec![(0.0, 0.0), (0.0, 1.0)],
            vec![(0.6, 0.0), (0.6, 1.0)],
            vec![(0.0, 0.5), (0.6, 0.5)],
        ],
        'I' => vec![
            vec![(0.15, 1.0), (0.45, 1.0)],
            vec![(0.3, 1.0), (0.3, 0.0)],
            vec![(0.15, 0.0), (0.45, 0.0)],
        ],
        'J' => vec![vec![
            (0.15, 1.0),
            (0.6, 1.0),
            (0.6, 0.2),
            (0.45, 0.0),
            (0.15, 0.0),
            (0.0, 0.2),
        ]],
        'K' => vec![
            vec![(0.0, 0.0), (0.0, 1.0)],
            vec![(0.6, 1.0), (0.0, 0.5), (0.6, 0.0)],
        ],
        'L' => vec![vec![(0.0, 1.0), (0.0, 0.0), (0.6, 0.0)]],
        'M' => vec![vec![
            (0.0, 0.0),
            (0.0, 1.0),
            (0.3, 0.5),
            (0.6, 1.0),
            (0.6, 0.0),
        ]],
        'N' => vec![vec![(0.0, 0.0), (0.0, 1.0), (0.6, 0.0), (0.6, 1.0)]],
        'O' => vec![vec![
            (0.15, 0.0),
            (0.0, 0.15),
            (0.0, 0.85),
            (0.15, 1.0),
            (0.45, 1.0),
            (0.6, 0.85),
            (0.6, 0.15),
            (0.45, 0.0),
            (0.15, 0.0),
        ]],
        'P' => vec![vec![
            (0.0, 0.0),
            (0.0, 1.0),
            (0.45, 1.0),
            (0.6, 0.85),
            (0.6, 0.65),
            (0.45, 0.5),
            (0.0, 0.5),
        ]],
        'Q' => vec![
            vec![
                (0.15, 0.0),
                (0.0, 0.15),
                (0.0, 0.85),
                (0.15, 1.0),
                (0.45, 1.0),
                (0.6, 0.85),
                (0.6, 0.15),
                (0.45, 0.0),
                (0.15, 0.0),
            ],
            vec![(0.38, 0.25), (0.6, 0.0)],
        ],
        'R' => vec![
            vec![
                (0.0, 0.0),
                (0.0, 1.0),
                (0.45, 1.0),
                (0.6, 0.85),
                (0.6, 0.65),
                (0.45, 0.5),
                (0.0, 0.5),
            ],
            vec![(0.35, 0.5), (0.6, 0.0)],
        ],
        'S' => vec![vec![
            (0.6, 0.85),
            (0.45, 1.0),
            (0.15, 1.0),
            (0.0, 0.85),
            (0.0, 0.6),
            (0.6, 0.4),
            (0.6, 0.15),
            (0.45, 0.0),
            (0.15, 0.0),
            (0.0, 0.15),
        ]],
        'T' => vec![vec![(0.0, 1.0), (0.6, 1.0)], vec![(0.3, 1.0), (0.3, 0.0)]],
        'U' => vec![vec![
            (0.0, 1.0),
            (0.0, 0.15),
            (0.15, 0.0),
            (0.45, 0.0),
            (0.6, 0.15),
            (0.6, 1.0),
        ]],
        'V' => vec![vec![(0.0, 1.0), (0.3, 0.0), (0.6, 1.0)]],
        'W' => vec![vec![
            (0.0, 1.0),
            (0.15, 0.0),
            (0.3, 0.5),
            (0.45, 0.0),
            (0.6, 1.0),
        ]],
        'X' => vec![vec![(0.0, 0.0), (0.6, 1.0)], vec![(0.0, 1.0), (0.6, 0.0)]],
        'Y' => vec![
            vec![(0.0, 1.0), (0.3, 0.5)],
            vec![(0.6, 1.0), (0.3, 0.5)],
            vec![(0.3, 0.5), (0.3, 0.0)],
        ],
        'Z' => vec![vec![(0.0, 1.0), (0.6, 1.0), (0.0, 0.0), (0.6, 0.0)]],
        '0' => vec![vec![
            (0.15, 0.0),
            (0.0, 0.15),
            (0.0, 0.85),
            (0.15, 1.0),
            (0.45, 1.0),
            (0.6, 0.85),
            (0.6, 0.15),
            (0.45, 0.0),
            (0.15, 0.0),
        ]],
        '1' => vec![vec![(0.15, 0.8), (0.3, 1.0), (0.3, 0.0)]],
        '2' => vec![vec![
            (0.0, 0.8),
            (0.15, 1.0),
            (0.45, 1.0),
            (0.6, 0.8),
            (0.6, 0.6),
            (0.0, 0.0),
            (0.6, 0.0),
        ]],
        '3' => vec![vec![
            (0.0, 1.0),
            (0.6, 1.0),
            (0.35, 0.5),
            (0.6, 0.5),
            (0.6, 0.15),
            (0.45, 0.0),
            (0.15, 0.0),
            (0.0, 0.15),
        ]],
        '4' => vec![
            vec![(0.55, 0.0), (0.55, 1.0)],
            vec![(0.0, 0.45), (0.6, 0.45)],
            vec![(0.0, 1.0), (0.0, 0.45)],
        ],
        '5' => vec![vec![
            (0.6, 1.0),
            (0.0, 1.0),
            (0.0, 0.55),
            (0.45, 0.55),
            (0.6, 0.4),
            (0.6, 0.15),
            (0.45, 0.0),
            (0.15, 0.0),
            (0.0, 0.15),
        ]],
        '6' => vec![vec![
            (0.55, 0.85),
            (0.4, 1.0),
            (0.15, 1.0),
            (0.0, 0.8),
            (0.0, 0.15),
            (0.15, 0.0),
            (0.45, 0.0),
            (0.6, 0.15),
            (0.6, 0.4),
            (0.45, 0.55),
            (0.0, 0.55),
        ]],
        '7' => vec![vec![(0.0, 1.0), (0.6, 1.0), (0.2, 0.0)]],
        '8' => vec![
            vec![
                (0.15, 0.5),
                (0.0, 0.65),
                (0.0, 0.85),
                (0.15, 1.0),
                (0.45, 1.0),
                (0.6, 0.85),
                (0.6, 0.65),
                (0.45, 0.5),
                (0.15, 0.5),
            ],
            vec![
                (0.15, 0.5),
                (0.0, 0.35),
                (0.0, 0.15),
                (0.15, 0.0),
                (0.45, 0.0),
                (0.6, 0.15),
                (0.6, 0.35),
                (0.45, 0.5),
                (0.15, 0.5),
            ],
        ],
        '9' => vec![vec![
            (0.6, 0.45),
            (0.15, 0.45),
            (0.0, 0.6),
            (0.0, 0.85),
            (0.15, 1.0),
            (0.45, 1.0),
            (0.6, 0.85),
            (0.6, 0.15),
            (0.45, 0.0),
            (0.2, 0.0),
            (0.05, 0.15),
        ]],
        '-' => vec![vec![(0.1, 0.5), (0.5, 0.5)]],
        '.' => vec![vec![
            (0.25, 0.05),
            (0.35, 0.05),
            (0.35, 0.0),
            (0.25, 0.0),
            (0.25, 0.05),
        ]],
        ' ' => vec![],
        _ => vec![], // unsupported char = blank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_text() {
        let pd = text_3d(&Text3dParams::default());
        assert!(pd.points.len() > 0);
        assert!(pd.polys.num_cells() > 0); // depth > 0 -> quads
    }

    #[test]
    fn flat_text() {
        let pd = text_3d(&Text3dParams {
            text: "HI".into(),
            depth: 0.0,
            ..Default::default()
        });
        assert!(pd.points.len() > 0);
        assert_eq!(pd.polys.num_cells(), 0);
        assert!(pd.lines.num_cells() > 0);
    }

    #[test]
    fn empty_text() {
        let pd = text_3d(&Text3dParams {
            text: "".into(),
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 0);
    }

    #[test]
    fn single_char() {
        let pd = text_3d(&Text3dParams {
            text: "A".into(),
            depth: 0.5,
            ..Default::default()
        });
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn alphanumeric_glyphs_are_supported() {
        for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars() {
            assert!(!char_strokes(ch).is_empty(), "missing glyph for {ch}");
        }
    }
}
