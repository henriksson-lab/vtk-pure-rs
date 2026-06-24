//! Medieval watchtower with crenellations and spiral stair.
use crate::data::{CellArray, Points, PolyData};
pub fn medieval_tower(
    radius: f64,
    height: f64,
    wall_thickness: f64,
    num_crenellations: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(12);
    let nc = num_crenellations.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Outer cylinder
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Inner cylinder (hollow)
    let ir = radius - wall_thickness;
    let ib = pts.len();
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([ir * a.cos(), ir * a.sin(), z]);
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
    // Top ring floor
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (res + i) as i64,
            (res + j) as i64,
            (ib + res + j) as i64,
            (ib + res + i) as i64,
        ]);
    }
    // Crenellations
    let merlon_arc = 2.0 * std::f64::consts::PI / nc as f64;
    let merlon_h = height * 0.1;
    for mi in 0..nc {
        let a0 = mi as f64 * merlon_arc;
        let a1 = a0 + merlon_arc * 0.5;
        let mb = pts.len();
        pts.push([radius * a0.cos(), radius * a0.sin(), height]);
        pts.push([radius * a1.cos(), radius * a1.sin(), height]);
        pts.push([radius * a1.cos(), radius * a1.sin(), height + merlon_h]);
        pts.push([radius * a0.cos(), radius * a0.sin(), height + merlon_h]);
        polys.push_cell(&[mb as i64, (mb + 1) as i64, (mb + 2) as i64, (mb + 3) as i64]);
    }
    // Spiral stair hint (line)
    let stair_steps = 20;
    let mut stair_ids = Vec::new();
    for si in 0..=stair_steps {
        let t = si as f64 / stair_steps as f64;
        let a = t * 4.0 * std::f64::consts::PI;
        let z = t * height;
        let sr = ir * 0.7;
        let idx = pts.len();
        pts.push([sr * a.cos(), sr * a.sin(), z]);
        stair_ids.push(idx as i64);
    }
    lines.push_cell(&stair_ids);
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
        let t = medieval_tower(3.0, 12.0, 0.8, 8, 16);
        assert!(t.polys.num_cells() > 30);
    }
}
