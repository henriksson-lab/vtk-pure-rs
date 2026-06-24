//! Windmill geometry (traditional style with blades and tower).
use crate::data::{CellArray, Points, PolyData};
pub fn windmill(
    tower_height: f64,
    tower_radius: f64,
    blade_length: f64,
    blade_width: f64,
    num_blades: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nb = num_blades.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Tower (tapered)
    let top_r = tower_radius * 0.6;
    for ring in 0..=1 {
        let t = ring as f64;
        let r = tower_radius * (1.0 - t) + top_r * t;
        let z = t * tower_height;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Cap
    let cap = pts.len();
    pts.push([0.0, 0.0, tower_height + top_r * 0.5]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[cap as i64, (res + i) as i64, (res + j) as i64]);
    }
    // Blades
    let hub_z = tower_height;
    let bw = blade_width / 2.0;
    for bi in 0..nb {
        let angle = 2.0 * std::f64::consts::PI * bi as f64 / nb as f64;
        let ca = angle.cos();
        let sa = angle.sin();
        let b = pts.len();
        // Blade quad (in XZ plane, rotated)
        pts.push([0.0, -bw * sa, hub_z - bw * ca]);
        pts.push([
            blade_length * ca,
            -bw * sa,
            hub_z + blade_length * sa - bw * ca,
        ]);
        pts.push([
            blade_length * ca,
            bw * sa,
            hub_z + blade_length * sa + bw * ca,
        ]);
        pts.push([0.0, bw * sa, hub_z + bw * ca]);
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
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
        let w = windmill(10.0, 1.5, 5.0, 0.8, 4, 8);
        assert!(w.polys.num_cells() > 10);
    }
}
