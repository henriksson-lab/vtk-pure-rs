use crate::data::PolyData;

use crate::filters::core::sources::cylinder::{cylinder, CylinderParams};

/// Parameters for generating an arrow (cylinder shaft + cone tip).
pub struct ArrowParams {
    /// Length of the cone tip as a fraction of the unit arrow length.
    pub tip_length: f64,
    pub tip_radius: f64,
    pub tip_resolution: usize,
    pub shaft_radius: f64,
    pub shaft_resolution: usize,
    pub invert: bool,
    pub arrow_origin: ArrowOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrowOrigin {
    Default,
    Center,
}

impl Default for ArrowParams {
    fn default() -> Self {
        Self {
            tip_length: 0.35,
            tip_radius: 0.1,
            tip_resolution: 6,
            shaft_radius: 0.03,
            shaft_resolution: 6,
            invert: false,
            arrow_origin: ArrowOrigin::Default,
        }
    }
}

/// Generate an arrow pointing along +X as PolyData.
///
/// By default the shaft base is at `(0, 0, 0)` and the tip is at `(1, 0, 0)`.
pub fn arrow(params: &ArrowParams) -> PolyData {
    let tip_length = params.tip_length.clamp(0.0, 1.0);
    let shaft_length = 1.0 - tip_length;
    let tip_radius = params.tip_radius.clamp(0.0, 10.0);
    let shaft_radius = params.shaft_radius.clamp(0.0, 5.0);

    // Shaft: VTK creates a Y-axis cylinder centered at y = shaft_length / 2,
    // then rotates it -90 degrees around Z so it spans x = 0..shaft_length.
    let shaft = cylinder(&CylinderParams {
        center: [0.0, shaft_length * 0.5, 0.0],
        height: shaft_length,
        radius: shaft_radius,
        resolution: params.shaft_resolution.clamp(3, 128),
        capping: true,
    });

    let mut shaft_points = Vec::new();
    for i in 0..shaft.points.len() {
        let [x, y, z] = shaft.points.get(i);
        shaft_points.push([y, -x, z]);
    }

    let (tip_points, tip_polys) =
        vtk_arrow_tip(tip_length, tip_radius, params.tip_resolution.clamp(1, 128));

    // Merge into single PolyData
    let shaft_n = shaft_points.len();
    let mut merged = PolyData::new();

    for p in &shaft_points {
        merged.points.push(transform_arrow_point(*p, params));
    }
    for p in &tip_points {
        merged.points.push(transform_arrow_point(*p, params));
    }

    // Copy shaft polys
    for cell in shaft.polys.iter() {
        merged.polys.push_cell(cell);
    }

    // Copy tip polys with offset indices
    for cell in tip_polys.iter() {
        let offset_cell: Vec<i64> = cell.iter().map(|&id| id + shaft_n as i64).collect();
        merged.polys.push_cell(&offset_cell);
    }

    merged
}

fn vtk_arrow_tip(
    tip_length: f64,
    tip_radius: f64,
    tip_resolution: usize,
) -> (Vec<[f64; 3]>, Vec<Vec<i64>>) {
    let mut points = Vec::new();
    let mut polys = Vec::new();
    let apex_x = 1.0;
    let base_x = 1.0 - tip_length;

    points.push([apex_x, 0.0, 0.0]);
    match tip_resolution {
        1 => {
            points.push([base_x, -tip_radius, 0.0]);
            points.push([base_x, tip_radius, 0.0]);
            polys.push(vec![0, 1, 2]);
        }
        2 => {
            points.push([base_x, 0.0, -tip_radius]);
            points.push([base_x, 0.0, tip_radius]);
            polys.push(vec![0, 1, 2]);
            points.push([base_x, -tip_radius, 0.0]);
            points.push([base_x, tip_radius, 0.0]);
            polys.push(vec![0, 3, 4]);
        }
        _ => {
            for i in 0..tip_resolution {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / tip_resolution as f64;
                points.push([base_x, tip_radius * angle.cos(), tip_radius * angle.sin()]);
            }

            let cap: Vec<i64> = (1..=tip_resolution as i64).rev().collect();
            polys.push(cap);

            for i in 0..tip_resolution {
                let next = if i + 1 == tip_resolution { 1 } else { i + 2 };
                polys.push(vec![0, (i + 1) as i64, next as i64]);
            }
        }
    }

    (points, polys)
}

fn transform_arrow_point(mut p: [f64; 3], params: &ArrowParams) -> [f64; 3] {
    if params.invert {
        p[0] = 1.0 - p[0];
    }
    if params.arrow_origin == ArrowOrigin::Center {
        p[0] -= 0.5;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_arrow() {
        let pd = arrow(&ArrowParams::default());
        assert!(pd.points.len() > 0);
        assert!(pd.polys.num_cells() > 0);

        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            xmin = xmin.min(p[0]);
            xmax = xmax.max(p[0]);
        }
        assert!(xmin.abs() < 1e-10);
        assert!((xmax - 1.0).abs() < 1e-10);
    }
}
