//! Compute position statistics: centroid, bounding box, principal axes lengths.
use crate::data::PolyData;

pub struct PositionStats {
    pub centroid: [f64; 3],
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
    pub rms_distance: f64,
    pub max_distance: f64,
    pub n_vertices: usize,
}

pub fn position_stats(mesh: &PolyData) -> PositionStats {
    let n = mesh.points.len();
    if n == 0 {
        return PositionStats {
            centroid: [0.0; 3],
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            rms_distance: 0.0,
            max_distance: 0.0,
            n_vertices: 0,
        };
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    let mut bmin = [f64::INFINITY; 3];
    let mut bmax = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
        for d in 0..3 {
            bmin[d] = bmin[d].min(p[d]);
            bmax[d] = bmax[d].max(p[d]);
        }
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;
    let mut rms2 = 0.0f64;
    let mut max_d = 0.0f64;
    for i in 0..n {
        let p = mesh.points.get(i);
        let d2 = (p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2);
        rms2 += d2;
        let d = d2.sqrt();
        if d > max_d {
            max_d = d;
        }
    }
    PositionStats {
        centroid: [cx, cy, cz],
        bbox_min: bmin,
        bbox_max: bmax,
        rms_distance: (rms2 / n as f64).sqrt(),
        max_distance: max_d,
        n_vertices: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pos_stats() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = position_stats(&mesh);
        assert!((s.centroid[0] - 1.0).abs() < 1e-9);
        assert_eq!(s.n_vertices, 3);
    }
}
