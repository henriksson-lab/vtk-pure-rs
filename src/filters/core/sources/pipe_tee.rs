//! Pipe T-junction and cross-junction geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn pipe_tee(radius: f64, main_length: f64, branch_length: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let hl = main_length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main pipe (along Z)
    for ring in 0..=1 {
        let z = if ring == 0 { -hl } else { hl };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Branch pipe (along X, from center)
    let bo = pts.len();
    for ring in 0..=1 {
        let x = if ring == 0 {
            radius
        } else {
            radius + branch_length
        };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, radius * a.cos(), radius * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (bo + i) as i64,
            (bo + j) as i64,
            (bo + res + j) as i64,
            (bo + res + i) as i64,
        ]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let t = pipe_tee(0.5, 4.0, 2.0, 8);
        assert!(t.points.len() > 20);
        assert!(t.polys.num_cells() > 10);
    }
}
