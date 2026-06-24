//! Simplified turbine/fan blade geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn turbine_blades(
    hub_radius: f64,
    blade_length: f64,
    blade_width: f64,
    num_blades: usize,
    twist: f64,
) -> PolyData {
    let nb = num_blades.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let bw = blade_width / 2.0;
    let steps = 8;
    for bi in 0..nb {
        let base_angle = 2.0 * std::f64::consts::PI * bi as f64 / nb as f64;
        for si in 0..=steps {
            let t = si as f64 / steps as f64;
            let r = hub_radius + blade_length * t;
            let tw = twist * t;
            let a = base_angle + tw;
            let ca = a.cos();
            let sa = a.sin();
            pts.push([r * ca - bw * sa, r * sa + bw * ca, 0.0]);
            pts.push([r * ca + bw * sa, r * sa - bw * ca, 0.0]);
        }
        let base = bi * (steps + 1) * 2;
        for si in 0..steps {
            let b = base + si * 2;
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 3) as i64, (b + 2) as i64]);
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
        let t = turbine_blades(0.5, 2.0, 0.3, 5, 0.3);
        assert!(t.polys.num_cells() >= 40);
    }
}
