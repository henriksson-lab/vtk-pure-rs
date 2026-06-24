use crate::data::PolyData;

/// Compute the signed volume enclosed by a closed triangle mesh.
///
/// Uses the divergence theorem: V = (1/6) Σ det([v0, v1, v2]) for each triangle.
/// Positive for outward-facing normals. Absolute value gives true volume.
pub fn signed_volume(input: &PolyData) -> f64 {
    let mut vol = 0.0;
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            vol += v0[0] * (v1[1] * v2[2] - v2[1] * v1[2])
                - v1[0] * (v0[1] * v2[2] - v2[1] * v0[2])
                + v2[0] * (v0[1] * v1[2] - v1[1] * v0[2]);
        }
    }
    vol / 6.0
}

/// Compute the total surface area of a triangle mesh.
pub fn surface_area(input: &PolyData) -> f64 {
    let mut area = 0.0;
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let c = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            area += 0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        }
    }
    area
}

/// Compute volume-to-surface-area ratio (sphericity indicator).
/// For a sphere, V/A = r/3. Higher ratio = more spherical.
pub fn compactness(input: &PolyData) -> f64 {
    let v = signed_volume(input).abs();
    let a = surface_area(input);
    if a > 1e-15 {
        36.0 * std::f64::consts::PI * v * v / (a * a * a)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box_mesh() -> PolyData {
        let mut pd = PolyData::new();
        let c = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        for p in &c {
            pd.points.push(*p);
        }
        let faces = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
        ];
        for f in &faces {
            pd.polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
            pd.polys.push_cell(&[f[0] as i64, f[2] as i64, f[3] as i64]);
        }
        pd
    }

    #[test]
    fn unit_cube_volume() {
        let pd = make_box_mesh();
        let v = signed_volume(&pd).abs();
        assert!((v - 1.0).abs() < 1e-10);
    }

    #[test]
    fn unit_cube_area() {
        let pd = make_box_mesh();
        let a = surface_area(&pd);
        assert!((a - 6.0).abs() < 1e-10);
    }

    #[test]
    fn compactness_bounded() {
        let pd = make_box_mesh();
        let c = compactness(&pd);
        assert!(c > 0.0 && c <= 1.0);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert_eq!(signed_volume(&pd), 0.0);
        assert_eq!(surface_area(&pd), 0.0);
    }
}
