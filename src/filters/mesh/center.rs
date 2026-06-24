use crate::data::{DataSet, Points, PolyData};

/// Center a mesh so its bounding box center is at the origin.
pub fn center_mesh(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    let cx = (bb.x_min + bb.x_max) * 0.5;
    let cy = (bb.y_min + bb.y_max) * 0.5;
    let cz = (bb.z_min + bb.z_max) * 0.5;

    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([p[0] - cx, p[1] - cy, p[2] - cz]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Normalize a mesh to fit inside a unit cube centered at origin.
pub fn normalize_mesh(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    let cx = (bb.x_min + bb.x_max) * 0.5;
    let cy = (bb.y_min + bb.y_max) * 0.5;
    let cz = (bb.z_min + bb.z_max) * 0.5;
    let dx = (bb.x_max - bb.x_min).max(1e-15);
    let dy = (bb.y_max - bb.y_min).max(1e-15);
    let dz = (bb.z_max - bb.z_min).max(1e-15);
    let scale = 1.0 / dx.max(dy).max(dz);

    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([
            (p[0] - cx) * scale,
            (p[1] - cy) * scale,
            (p[2] - cz) * scale,
        ]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Align the principal axis of the mesh to the Z axis using PCA.
pub fn align_to_z(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return input.clone();
    }

    // Centroid
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
    cx /= nf;
    cy /= nf;
    cz /= nf;

    // Covariance
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }

    // Find dominant eigenvector via power iteration
    let s = 1.0f64 / 3.0f64.sqrt();
    let mut v = [s, s, s];
    for _ in 0..50 {
        let mut nv = [0.0; 3];
        for r in 0..3 {
            for c in 0..3 {
                nv[r] += cov[r][c] * v[c];
            }
        }
        let len = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt();
        if len > 1e-15 {
            v = [nv[0] / len, nv[1] / len, nv[2] / len];
        }
    }

    // Rotation from v to [0,0,1]
    let target = [0.0, 0.0, 1.0];
    let dot = v[0] * target[0] + v[1] * target[1] + v[2] * target[2];

    if dot.abs() > 0.999 {
        // Already aligned (or anti-aligned)
        return center_mesh(input);
    }

    let cross = [
        v[1] * target[2] - v[2] * target[1],
        v[2] * target[0] - v[0] * target[2],
        v[0] * target[1] - v[1] * target[0],
    ];
    let sin_a = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let cos_a = dot;

    let k = [cross[0] / sin_a, cross[1] / sin_a, cross[2] / sin_a];

    // Rodrigues rotation matrix
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        let kdot = k[0] * d[0] + k[1] * d[1] + k[2] * d[2];
        let kcross = [
            k[1] * d[2] - k[2] * d[1],
            k[2] * d[0] - k[0] * d[2],
            k[0] * d[1] - k[1] * d[0],
        ];
        points.push([
            d[0] * cos_a + kcross[0] * sin_a + k[0] * kdot * (1.0 - cos_a),
            d[1] * cos_a + kcross[1] * sin_a + k[1] * kdot * (1.0 - cos_a),
            d[2] * cos_a + kcross[2] * sin_a + k[2] * kdot * (1.0 - cos_a),
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_basic() {
        let mut pd = PolyData::new();
        pd.points.push([2.0, 4.0, 6.0]);
        pd.points.push([4.0, 6.0, 8.0]);

        let result = center_mesh(&pd);
        let bb = result.bounds();
        let center = bb.center();
        assert!(center[0].abs() < 1e-10);
        assert!(center[1].abs() < 1e-10);
        assert!(center[2].abs() < 1e-10);
    }

    #[test]
    fn normalize_fits_unit() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([0.0, 5.0, 0.0]);

        let result = normalize_mesh(&pd);
        let bb = result.bounds();
        let dx = bb.x_max - bb.x_min;
        let dy = bb.y_max - bb.y_min;
        assert!(dx <= 1.0 + 1e-10);
        assert!(dy <= 1.0 + 1e-10);
    }

    #[test]
    fn align_x_points_to_z() {
        let mut pd = PolyData::new();
        // Points primarily along X axis
        for i in 0..20 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }

        let result = align_to_z(&pd);
        // After alignment, spread should be primarily along Z
        let bb = result.bounds();
        let dz = bb.z_max - bb.z_min;
        let dx = bb.x_max - bb.x_min;
        assert!(dz > dx, "dz={} dx={}", dz, dx);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let _ = center_mesh(&pd);
        let _ = normalize_mesh(&pd);
    }
}
