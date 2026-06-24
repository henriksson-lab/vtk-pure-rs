//! Greek temple (colonnade + pediment) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn greek_temple(
    width: f64,
    depth: f64,
    column_h: f64,
    num_front_cols: usize,
    column_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nfc = num_front_cols.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let add_box = |pts: &mut Points<f64>,
                   polys: &mut CellArray,
                   x0: f64,
                   y0: f64,
                   z0: f64,
                   x1: f64,
                   y1: f64,
                   z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    let hw = width / 2.0;
    let hd = depth / 2.0;
    // Stylobate (base platform)
    let plat_h = column_h * 0.05;
    add_box(
        &mut pts,
        &mut polys,
        -hw * 1.1,
        -hd * 1.1,
        0.0,
        hw * 1.1,
        hd * 1.1,
        plat_h,
    );
    // Columns (front row)
    let spacing = width / (nfc - 1) as f64;
    for ci in 0..nfc {
        let x = -hw + ci as f64 * spacing;
        let cb = pts.len();
        for ring in 0..=1 {
            let z = plat_h + if ring == 0 { 0.0 } else { column_h };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([x + column_r * a.cos(), -hd + column_r * a.sin(), z]);
            }
        }
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (cb + i) as i64,
                (cb + j) as i64,
                (cb + res + j) as i64,
                (cb + res + i) as i64,
            ]);
        }
    }
    // Entablature
    let ent_h = column_h * 0.1;
    let ent_z = plat_h + column_h;
    add_box(&mut pts, &mut polys, -hw, -hd, ent_z, hw, hd, ent_z + ent_h);
    // Pediment (triangular front)
    let ped_h = column_h * 0.2;
    let ped_z = ent_z + ent_h;
    let pb = pts.len();
    pts.push([-hw, -hd, ped_z]);
    pts.push([hw, -hd, ped_z]);
    pts.push([0.0, -hd, ped_z + ped_h]);
    polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64]);
    // Back pediment
    let pb2 = pts.len();
    pts.push([-hw, hd, ped_z]);
    pts.push([hw, hd, ped_z]);
    pts.push([0.0, hd, ped_z + ped_h]);
    polys.push_cell(&[(pb2 + 1) as i64, pb2 as i64, (pb2 + 2) as i64]);
    // Roof slopes
    polys.push_cell(&[pb as i64, (pb + 2) as i64, (pb2 + 2) as i64, pb2 as i64]);
    polys.push_cell(&[
        (pb + 1) as i64,
        (pb2 + 1) as i64,
        (pb2 + 2) as i64,
        (pb + 2) as i64,
    ]);
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
        let t = greek_temple(10.0, 15.0, 5.0, 6, 0.3, 8);
        assert!(t.polys.num_cells() > 40);
    }
}
