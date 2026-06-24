use crate::data::PolyData;

/// Compute the signed volume of a closed triangular mesh using the divergence theorem.
///
/// Each triangle contributes a signed tetrahedron volume formed with the origin.
/// The formula is: V = (1/6) * sum over triangles of p0 . (p1 x p2).
/// The sign depends on winding order; outward-facing normals give positive volume.
pub fn compute_signed_volume(input: &PolyData) -> f64 {
    let mut total: f64 = 0.0;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let p0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);

            // Signed volume of tetrahedron formed by origin and triangle p0,p1,p2:
            // (1/6) * p0 . (p1 x p2)
            let cross_x: f64 = p1[1] * p2[2] - p1[2] * p2[1];
            let cross_y: f64 = p1[2] * p2[0] - p1[0] * p2[2];
            let cross_z: f64 = p1[0] * p2[1] - p1[1] * p2[0];
            total += p0[0] * cross_x + p0[1] * cross_y + p0[2] * cross_z;
        }
    }

    total / 6.0
}

/// Compute the absolute volume of a closed triangular mesh.
pub fn compute_volume(input: &PolyData) -> f64 {
    compute_signed_volume(input).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit cube made of 12 triangles (2 per face), outward-facing normals.
    fn unit_cube() -> PolyData {
        let pts = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        // CCW winding when viewed from outside
        let tris: Vec<[i64; 3]> = vec![
            // -Z face
            [0, 3, 2],
            [0, 2, 1],
            // +Z face
            [4, 5, 6],
            [4, 6, 7],
            // -Y face
            [0, 1, 5],
            [0, 5, 4],
            // +Y face
            [2, 3, 7],
            [2, 7, 6],
            // -X face
            [0, 4, 7],
            [0, 7, 3],
            // +X face
            [1, 2, 6],
            [1, 6, 5],
        ];
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn unit_cube_volume() {
        let cube = unit_cube();
        let vol: f64 = compute_volume(&cube);
        assert!((vol - 1.0).abs() < 1e-10, "expected 1.0, got {}", vol);
    }

    #[test]
    fn signed_volume_sign() {
        let cube = unit_cube();
        let sv: f64 = compute_signed_volume(&cube);
        // With outward normals (CCW), signed volume should be positive
        assert!(sv > 0.0, "expected positive signed volume, got {}", sv);
    }

    #[test]
    fn single_tetrahedron() {
        // Regular tetrahedron with known volume = sqrt(2)/12 * a^3, a=1 => ~0.11785
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, (3.0f64).sqrt() / 2.0, 0.0],
            [0.5, (3.0f64).sqrt() / 6.0, (2.0f64 / 3.0).sqrt()],
        ];
        let tris: Vec<[i64; 3]> = vec![
            [0, 2, 1], // bottom face (outward = -Z)
            [0, 1, 3], // front
            [1, 2, 3], // right
            [2, 0, 3], // left
        ];
        let pd = PolyData::from_triangles(pts, tris);
        let vol: f64 = compute_volume(&pd);
        let expected: f64 = (2.0f64).sqrt() / 12.0;
        assert!(
            (vol - expected).abs() < 1e-10,
            "expected {}, got {}",
            expected,
            vol
        );
    }
}
