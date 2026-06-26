//! Arrow source: capped shaft plus capped cone tip.

use crate::data::{CellArray, Points, PolyData};

/// Create a Z-oriented arrow as a capped cylinder shaft appended to a capped cone tip.
pub fn arrow_z(
    shaft_radius: f64,
    shaft_length: f64,
    tip_radius: f64,
    tip_length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    append_capped_cylinder_z(&mut pts, &mut polys, 0.0, shaft_length, shaft_radius, res);
    append_capped_cone_z(
        &mut pts,
        &mut polys,
        shaft_length,
        shaft_length + tip_length,
        tip_radius,
        res,
    );

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a double-headed arrow along Z.
pub fn double_arrow_z(
    shaft_radius: f64,
    shaft_length: f64,
    tip_radius: f64,
    tip_length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(3);
    let half = shaft_length * 0.5;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    append_capped_cylinder_z(&mut pts, &mut polys, -half, half, shaft_radius, res);
    append_capped_cone_z(
        &mut pts,
        &mut polys,
        half,
        half + tip_length,
        tip_radius,
        res,
    );
    append_capped_cone_z(
        &mut pts,
        &mut polys,
        -half,
        -half - tip_length,
        tip_radius,
        res,
    );

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn append_capped_cylinder_z(
    pts: &mut Points<f64>,
    polys: &mut CellArray,
    z0: f64,
    z1: f64,
    radius: f64,
    resolution: usize,
) {
    let base = pts.len();
    for i in 0..resolution {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / resolution as f64;
        pts.push([radius * angle.cos(), radius * angle.sin(), z0]);
    }
    for i in 0..resolution {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / resolution as f64;
        pts.push([radius * angle.cos(), radius * angle.sin(), z1]);
    }
    let center0 = pts.len();
    pts.push([0.0, 0.0, z0]);
    let center1 = pts.len();
    pts.push([0.0, 0.0, z1]);

    for i in 0..resolution {
        let j = (i + 1) % resolution;
        polys.push_cell(&[
            (base + i) as i64,
            (base + j) as i64,
            (base + resolution + j) as i64,
            (base + resolution + i) as i64,
        ]);
        polys.push_cell(&[center0 as i64, (base + j) as i64, (base + i) as i64]);
        polys.push_cell(&[
            center1 as i64,
            (base + resolution + i) as i64,
            (base + resolution + j) as i64,
        ]);
    }
}

fn append_capped_cone_z(
    pts: &mut Points<f64>,
    polys: &mut CellArray,
    base_z: f64,
    tip_z: f64,
    radius: f64,
    resolution: usize,
) {
    let base = pts.len();
    for i in 0..resolution {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / resolution as f64;
        pts.push([radius * angle.cos(), radius * angle.sin(), base_z]);
    }
    let tip = pts.len();
    pts.push([0.0, 0.0, tip_z]);
    let center = pts.len();
    pts.push([0.0, 0.0, base_z]);

    for i in 0..resolution {
        let j = (i + 1) % resolution;
        if tip_z >= base_z {
            polys.push_cell(&[(base + i) as i64, (base + j) as i64, tip as i64]);
            polys.push_cell(&[center as i64, (base + j) as i64, (base + i) as i64]);
        } else {
            polys.push_cell(&[(base + j) as i64, (base + i) as i64, tip as i64]);
            polys.push_cell(&[center as i64, (base + i) as i64, (base + j) as i64]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_arrow() {
        let a = arrow_z(0.1, 1.0, 0.2, 0.3, 8);
        assert_eq!(a.points.len(), 28); // capped cylinder (2*8+2) + capped cone (8+2)
        assert_eq!(a.polys.num_cells(), 40); // 24 shaft + 16 tip
    }
    #[test]
    fn test_double() {
        let a = double_arrow_z(0.1, 1.0, 0.2, 0.3, 6);
        assert_eq!(a.points.len(), 30); // capped cylinder + two capped cones
    }
}
