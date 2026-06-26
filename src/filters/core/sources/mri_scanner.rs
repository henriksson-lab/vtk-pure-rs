//! MRI scanner geometry (bore + magnet + patient table).
use crate::data::{CellArray, Points, PolyData};
pub fn mri_scanner(
    bore_r: f64,
    bore_l: f64,
    housing_r: f64,
    housing_l: f64,
    table_w: f64,
    table_l: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer housing (cylinder)
    for ring in 0..=1 {
        let x = if ring == 0 {
            -housing_l / 2.0
        } else {
            housing_l / 2.0
        };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, housing_r * a.cos(), housing_r * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // End caps are annuli so the patient bore remains open.
    for &ring_idx in &[0usize, res] {
        let x = if ring_idx == 0 {
            -housing_l / 2.0
        } else {
            housing_l / 2.0
        };
        let ec = pts.len();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, housing_r * a.cos(), housing_r * a.sin()]);
            pts.push([x, bore_r * a.cos(), bore_r * a.sin()]);
        }
        for i in 0..res {
            let j = (i + 1) % res;
            let outer_i = ec + 2 * i;
            let inner_i = outer_i + 1;
            let outer_j = ec + 2 * j;
            let inner_j = outer_j + 1;
            if ring_idx == 0 {
                polys.push_cell(&[
                    outer_i as i64,
                    outer_j as i64,
                    inner_j as i64,
                    inner_i as i64,
                ]);
            } else {
                polys.push_cell(&[
                    outer_i as i64,
                    inner_i as i64,
                    inner_j as i64,
                    outer_j as i64,
                ]);
            }
        }
    }
    // Bore (inner cylinder)
    let ib = pts.len();
    for ring in 0..=1 {
        let x = if ring == 0 {
            -bore_l / 2.0
        } else {
            bore_l / 2.0
        };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, bore_r * a.cos(), bore_r * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (ib + j) as i64,
            (ib + i) as i64,
            (ib + res + i) as i64,
            (ib + res + j) as i64,
        ]);
    }
    // Patient table
    let hw = table_w / 2.0;
    let th = bore_r * 0.05;
    let tb = pts.len();
    pts.push([-table_l / 2.0, -hw, -bore_r + th]);
    pts.push([table_l / 2.0, -hw, -bore_r + th]);
    pts.push([table_l / 2.0, hw, -bore_r + th]);
    pts.push([-table_l / 2.0, hw, -bore_r + th]);
    pts.push([-table_l / 2.0, -hw, -bore_r + th * 2.0]);
    pts.push([table_l / 2.0, -hw, -bore_r + th * 2.0]);
    pts.push([table_l / 2.0, hw, -bore_r + th * 2.0]);
    pts.push([-table_l / 2.0, hw, -bore_r + th * 2.0]);
    let f = |i: usize| (tb + i) as i64;
    polys.push_cell(&[f(4), f(5), f(6), f(7)]); // top
    polys.push_cell(&[f(0), f(1), f(5), f(4)]); // front
    polys.push_cell(&[f(2), f(3), f(7), f(6)]); // back
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
        let m = mri_scanner(0.35, 1.5, 0.9, 2.0, 0.5, 2.5, 12);
        assert!(m.polys.num_cells() > 30);
    }
}
