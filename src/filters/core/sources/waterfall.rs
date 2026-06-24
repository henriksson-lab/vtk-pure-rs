//! Waterfall (cliff face with cascading water surface).
use crate::data::{CellArray, Points, PolyData};

pub fn waterfall(
    width: f64,
    cliff_height: f64,
    _pool_depth: f64,
    n_width: usize,
    n_height: usize,
) -> PolyData {
    let nw = n_width.max(4);
    let nh = n_height.max(6);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Cliff face
    for h in 0..=nh {
        let t = h as f64 / nh as f64;
        let z = cliff_height * (1.0 - t);
        let y = -0.1 * t; // slight overhang
        for w in 0..=nw {
            let x = -hw + width * w as f64 / nw as f64;
            pts.push([x, y, z]);
        }
    }
    let stride = nw + 1;
    for h in 0..nh {
        for w in 0..nw {
            let i0 = h * stride + w;
            let i1 = (h + 1) * stride + w;
            polys.push_cell(&[i0 as i64, i1 as i64, (i1 + 1) as i64, (i0 + 1) as i64]);
        }
    }
    // Water surface (curved sheet falling)
    let wb = pts.len();
    let water_segs = 8;
    for s in 0..=water_segs {
        let t = s as f64 / water_segs as f64;
        let y = -0.5 * t * t * cliff_height * 0.3; // parabolic fall
        let z = cliff_height * (1.0 - t);
        for w in 0..=nw {
            let x = -hw * 0.8 + width * 0.8 * w as f64 / nw as f64;
            pts.push([x, y - 0.2, z]);
        }
    }
    for s in 0..water_segs {
        for w in 0..nw {
            let i0 = wb + s * stride + w;
            let i1 = wb + (s + 1) * stride + w;
            polys.push_cell(&[i0 as i64, (i0 + 1) as i64, (i1 + 1) as i64, i1 as i64]);
        }
    }
    // Pool at base
    let pb = pts.len();
    let pool_w = width * 1.2;
    let pool_d = cliff_height * 0.4;
    pts.push([-pool_w / 2.0, -pool_d, 0.0]);
    pts.push([pool_w / 2.0, -pool_d, 0.0]);
    pts.push([pool_w / 2.0, 0.0, 0.0]);
    pts.push([-pool_w / 2.0, 0.0, 0.0]);
    polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_waterfall() {
        let m = waterfall(5.0, 10.0, 2.0, 6, 8);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 50);
    }
}
