//! 2D arrow glyph source for vector field visualization.

use crate::data::{CellArray, Points, PolyData};

/// Parameters for 2D arrow generation.
pub struct Arrow2dParams {
    /// Total length. Default: 1.0
    pub length: f64,
    /// Head width (fraction of length). Default: 0.3
    pub head_width: f64,
    /// Head length (fraction of total length). Default: 0.3
    pub head_length: f64,
    /// Shaft width (fraction of length). Default: 0.1
    pub shaft_width: f64,
}

impl Default for Arrow2dParams {
    fn default() -> Self {
        Self {
            length: 1.0,
            head_width: 0.3,
            head_length: 0.3,
            shaft_width: 0.1,
        }
    }
}

/// Generate a 2D arrow polygon in the XY plane pointing in +X.
pub fn arrow_2d(params: &Arrow2dParams) -> PolyData {
    let l = params.length;
    let hw = params.head_width * l;
    let hl = params.head_length * l;
    let sw = params.shaft_width * l;
    let shaft_end = l - hl;

    let pts = vec![
        [0.0, -sw / 2.0, 0.0],       // 0: shaft bottom-left
        [shaft_end, -sw / 2.0, 0.0], // 1: shaft bottom-right
        [shaft_end, -hw / 2.0, 0.0], // 2: head bottom
        [l, 0.0, 0.0],               // 3: tip
        [shaft_end, hw / 2.0, 0.0],  // 4: head top
        [shaft_end, sw / 2.0, 0.0],  // 5: shaft top-right
        [0.0, sw / 2.0, 0.0],        // 6: shaft top-left
    ];

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }

    let mut polys = CellArray::new();
    polys.push_cell(&[0, 1, 5, 6]); // shaft quad
    polys.push_cell(&[2, 3, 4]); // head triangle

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_arrow() {
        let a = arrow_2d(&Arrow2dParams::default());
        assert_eq!(a.points.len(), 7);
        assert_eq!(a.polys.num_cells(), 2);
    }

    #[test]
    fn custom_arrow() {
        let a = arrow_2d(&Arrow2dParams {
            length: 2.0,
            ..Default::default()
        });
        let tip = a.points.get(3);
        assert!((tip[0] - 2.0).abs() < 1e-10);
    }
}
