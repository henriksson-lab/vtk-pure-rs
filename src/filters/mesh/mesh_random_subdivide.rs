//! Randomly subdivide selected triangles by inserting midpoints.
use crate::data::{CellArray, Points, PolyData};

pub fn random_subdivide(mesh: &PolyData, fraction: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.is_empty() {
        return mesh.clone();
    }
    let mut rng = seed;
    let mut next_rand = || -> f64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng >> 33) as f64) / (u32::MAX as f64)
    };
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        pts.push(mesh.points.get(i).try_into().unwrap());
    }
    let mut polys = CellArray::new();
    for &[a, b, c] in &tris {
        if next_rand() < fraction {
            // Subdivide: insert centroid, create 3 sub-triangles
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let mid = pts.len();
            pts.push([
                (pa[0] + pb[0] + pc[0]) / 3.0,
                (pa[1] + pb[1] + pc[1]) / 3.0,
                (pa[2] + pb[2] + pc[2]) / 3.0,
            ]);
            polys.push_cell(&[a as i64, b as i64, mid as i64]);
            polys.push_cell(&[b as i64, c as i64, mid as i64]);
            polys.push_cell(&[c as i64, a as i64, mid as i64]);
        } else {
            polys.push_cell(&[a as i64, b as i64, c as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random_sub() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = random_subdivide(&mesh, 1.0, 42); // subdivide all
        assert_eq!(r.polys.num_cells(), 6); // 2 tris * 3 = 6
        assert_eq!(r.points.len(), 6); // 4 original + 2 centroids
    }
}
