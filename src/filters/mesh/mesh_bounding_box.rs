//! Compute axis-aligned bounding box dimensions and aspect ratios.
use crate::data::PolyData;

pub struct BoundingBoxInfo {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub size: [f64; 3],
    pub center: [f64; 3],
    pub diagonal: f64,
    pub aspect_ratio: f64,
}

pub fn bounding_box_info(mesh: &PolyData) -> BoundingBoxInfo {
    let n = mesh.points.len();
    if n == 0 {
        return BoundingBoxInfo {
            min: [0.0; 3],
            max: [0.0; 3],
            size: [0.0; 3],
            center: [0.0; 3],
            diagonal: 0.0,
            aspect_ratio: 1.0,
        };
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
    let center = [
        (bmin[0] + bmax[0]) / 2.0,
        (bmin[1] + bmax[1]) / 2.0,
        (bmin[2] + bmax[2]) / 2.0,
    ];
    let diagonal = (size[0] * size[0] + size[1] * size[1] + size[2] * size[2]).sqrt();
    let max_dim = size[0].max(size[1]).max(size[2]);
    let min_dim = size[0].min(size[1]).min(size[2]).max(1e-15);
    BoundingBoxInfo {
        min: bmin,
        max: bmax,
        size,
        center,
        diagonal,
        aspect_ratio: max_dim / min_dim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bbox() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let bb = bounding_box_info(&mesh);
        assert_eq!(bb.min[0], 0.0);
        assert_eq!(bb.max[0], 2.0);
        assert_eq!(bb.size[1], 3.0);
    }
}
