use crate::data::{CellArray, DataSet, Points, PolyData};

/// Compute the axis-aligned bounding box of a PolyData as a closed quad mesh.
///
/// Returns a PolyData with 8 vertices and 6 quad faces representing
/// the tight bounding box of the input geometry.
pub fn bounding_box_mesh(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    let corners = [
        [bb.x_min, bb.y_min, bb.z_min],
        [bb.x_max, bb.y_min, bb.z_min],
        [bb.x_max, bb.y_max, bb.z_min],
        [bb.x_min, bb.y_max, bb.z_min],
        [bb.x_min, bb.y_min, bb.z_max],
        [bb.x_max, bb.y_min, bb.z_max],
        [bb.x_max, bb.y_max, bb.z_max],
        [bb.x_min, bb.y_max, bb.z_max],
    ];

    let mut points = Points::<f64>::new();
    for c in &corners {
        points.push(*c);
    }

    let mut polys = CellArray::new();
    let faces: [[i64; 4]; 6] = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for f in &faces {
        polys.push_cell(f);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

/// Compute the oriented bounding box dimensions (extent along each principal axis).
///
/// Returns (center, extents, axes) where extents are half-sizes and
/// axes are the 3 principal directions. Uses PCA on the point set.
pub fn oriented_bounding_box(input: &PolyData) -> ([f64; 3], [f64; 3], [[f64; 3]; 3]) {
    let n = input.points.len();
    if n == 0 {
        return (
            [0.0; 3],
            [0.0; 3],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        );
    }

    // Compute centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    let center = [cx / nf, cy / nf, cz / nf];

    // Covariance matrix
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }
    for r in 0..3 {
        for c in 0..3 {
            cov[r][c] /= nf;
        }
    }

    // Simple power iteration for dominant eigenvector
    let axes = approximate_eigenvectors(&cov);

    // Project points onto axes to find extents
    let mut extents = [0.0f64; 3];
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        for a in 0..3 {
            let proj = d[0] * axes[a][0] + d[1] * axes[a][1] + d[2] * axes[a][2];
            extents[a] = extents[a].max(proj.abs());
        }
    }

    (center, extents, axes)
}

fn approximate_eigenvectors(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    // Power iteration for first eigenvector
    let v1 = power_iteration(m);
    // Deflate and find second
    let mut m2 = *m;
    let e1 = rayleigh(m, &v1);
    for r in 0..3 {
        for c in 0..3 {
            m2[r][c] -= e1 * v1[r] * v1[c];
        }
    }
    let v2 = power_iteration(&m2);
    // Third = cross product
    let v3 = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    [v1, v2, v3]
}

fn power_iteration(m: &[[f64; 3]; 3]) -> [f64; 3] {
    let mut v = [1.0, 0.0, 0.0];
    for _ in 0..50 {
        let mut nv = [0.0; 3];
        for r in 0..3 {
            for c in 0..3 {
                nv[r] += m[r][c] * v[c];
            }
        }
        let len = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt();
        if len > 1e-15 {
            v = [nv[0] / len, nv[1] / len, nv[2] / len];
        }
    }
    v
}

fn rayleigh(m: &[[f64; 3]; 3], v: &[f64; 3]) -> f64 {
    let mut mv = [0.0; 3];
    for r in 0..3 {
        for c in 0..3 {
            mv[r] += m[r][c] * v[c];
        }
    }
    v[0] * mv[0] + v[1] * mv[1] + v[2] * mv[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 3.0]);
        pd.polys.push_cell(&[0, 1]);

        let result = bounding_box_mesh(&pd);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.polys.num_cells(), 6);
    }

    #[test]
    fn obb_extents() {
        let mut pd = PolyData::new();
        // Points along X axis
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let (center, extents, _) = oriented_bounding_box(&pd);
        assert!((center[0] - 4.5).abs() < 1e-10);
        // Largest extent should be along X
        let max_ext = extents[0].max(extents[1]).max(extents[2]);
        assert!((max_ext - 4.5).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (_, extents, _) = oriented_bounding_box(&pd);
        assert_eq!(extents, [0.0, 0.0, 0.0]);
    }
}
