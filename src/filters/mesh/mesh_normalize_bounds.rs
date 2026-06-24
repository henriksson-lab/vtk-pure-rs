//! Normalize mesh to fit within a unit bounding box centered at origin.
use crate::data::{Points, PolyData};

pub fn normalize_bounds(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut bmin = [f64::INFINITY; 3];
    let mut bmax = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for d in 0..3 {
            bmin[d] = bmin[d].min(p[d]);
            bmax[d] = bmax[d].max(p[d]);
        }
    }
    let size = [bmax[0] - bmin[0], bmax[1] - bmin[1], bmax[2] - bmin[2]];
    let max_size = size[0].max(size[1]).max(size[2]).max(1e-15);
    let center = [
        (bmin[0] + bmax[0]) / 2.0,
        (bmin[1] + bmax[1]) / 2.0,
        (bmin[2] + bmax[2]) / 2.0,
    ];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            (p[0] - center[0]) / max_size,
            (p[1] - center[1]) / max_size,
            (p[2] - center[2]) / max_size,
        ]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize() {
        let mesh = PolyData::from_triangles(
            vec![[10.0, 20.0, 30.0], [20.0, 20.0, 30.0], [15.0, 30.0, 30.0]],
            vec![[0, 1, 2]],
        );
        let r = normalize_bounds(&mesh);
        for i in 0..3 {
            let p = r.points.get(i);
            assert!(p[0] >= -0.5 && p[0] <= 0.5);
            assert!(p[1] >= -0.5 && p[1] <= 0.5);
        }
    }
}
