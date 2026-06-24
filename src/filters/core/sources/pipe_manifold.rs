//! Pipe manifold (main pipe with multiple branches).
use crate::data::{CellArray, Points, PolyData};
pub fn pipe_manifold(
    main_radius: f64,
    main_length: f64,
    branch_radius: f64,
    branch_length: f64,
    branch_positions: &[f64],
    resolution: usize,
) -> PolyData {
    let res = resolution.max(4);
    let hl = main_length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main pipe
    for ring in 0..=1 {
        let z = if ring == 0 { -hl } else { hl };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([main_radius * a.cos(), main_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Branches
    for &bz in branch_positions {
        let bo = pts.len();
        for ring in 0..=1 {
            let x = if ring == 0 {
                main_radius
            } else {
                main_radius + branch_length
            };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([x, branch_radius * a.cos() + bz, branch_radius * a.sin()]);
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
        let m = pipe_manifold(0.5, 6.0, 0.2, 1.0, &[-2.0, 0.0, 2.0], 8);
        assert!(m.polys.num_cells() > 20);
    }
}
