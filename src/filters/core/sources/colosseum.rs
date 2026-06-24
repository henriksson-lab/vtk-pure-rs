//! Colosseum (elliptical tiered amphitheater) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn colosseum(
    major_r: f64,
    minor_r: f64,
    height: f64,
    tiers: usize,
    wall_t: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(12);
    let nt = tiers.max(1);
    let tier_h = height / nt as f64;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer wall
    for ti in 0..=nt {
        let z = ti as f64 * tier_h;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([major_r * a.cos(), minor_r * a.sin(), z]);
        }
    }
    for ti in 0..nt {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (ti * res + i) as i64,
                (ti * res + j) as i64,
                ((ti + 1) * res + j) as i64,
                ((ti + 1) * res + i) as i64,
            ]);
        }
    }
    // Inner wall
    let ir_major = major_r - wall_t;
    let ir_minor = minor_r - wall_t;
    let inner_base = pts.len();
    for ti in 0..=nt {
        let z = ti as f64 * tier_h;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([ir_major * a.cos(), ir_minor * a.sin(), z]);
        }
    }
    for ti in 0..nt {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (inner_base + ti * res + j) as i64,
                (inner_base + ti * res + i) as i64,
                (inner_base + (ti + 1) * res + i) as i64,
                (inner_base + (ti + 1) * res + j) as i64,
            ]);
        }
    }
    // Top ring connecting inner and outer
    let top_outer = nt * res;
    let top_inner = inner_base + nt * res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (top_outer + i) as i64,
            (top_outer + j) as i64,
            (top_inner + j) as i64,
            (top_inner + i) as i64,
        ]);
    }
    // Arena floor
    let fc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[fc as i64, (inner_base + j) as i64, (inner_base + i) as i64]);
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
        let c = colosseum(50.0, 40.0, 20.0, 3, 5.0, 24);
        assert!(c.polys.num_cells() > 100);
    }
}
