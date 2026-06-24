//! Gazebo (open-sided pavilion) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn gazebo(
    radius: f64,
    height: f64,
    roof_height: f64,
    num_pillars: usize,
    resolution: usize,
) -> PolyData {
    let np = num_pillars.max(4);
    let res = resolution.max(np);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Floor (polygon)
    let fc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for i in 0..res {
        let j = if i + 1 < res { fc + 2 + i } else { fc + 1 };
        polys.push_cell(&[fc as i64, (fc + 1 + i) as i64, j as i64]);
    }
    // Pillars
    let _pillar_r = radius * 0.05;
    for pi in 0..np {
        let a = 2.0 * std::f64::consts::PI * pi as f64 / np as f64;
        let x = radius * 0.95 * a.cos();
        let y = radius * 0.95 * a.sin();
        let pb = pts.len();
        pts.push([x, y, 0.0]);
        pts.push([x, y, height]);
        lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    }
    // Roof (cone)
    let rc = pts.len();
    pts.push([0.0, 0.0, height + roof_height]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * 1.1 * a.cos(), radius * 1.1 * a.sin(), height]);
    }
    for i in 0..res {
        let j = if i + 1 < res { rc + 2 + i } else { rc + 1 };
        polys.push_cell(&[rc as i64, (rc + 1 + i) as i64, j as i64]);
    }
    // Eaves ring
    let eb = rc + 1;
    for i in 0..res {
        let j = if i + 1 < res { eb + 1 + i } else { eb };
        lines.push_cell(&[(eb + i) as i64, j as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let g = gazebo(3.0, 2.5, 1.5, 6, 12);
        assert!(g.polys.num_cells() > 10);
        assert!(g.lines.num_cells() > 5);
    }
}
