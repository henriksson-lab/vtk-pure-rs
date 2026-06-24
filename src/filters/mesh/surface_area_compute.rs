use crate::data::PolyData;

/// Compute the total surface area of a PolyData mesh.
///
/// Handles general polygons by triangle fan decomposition from the first
/// vertex of each polygon. Each triangle's area is computed via the cross
/// product of two edge vectors.
pub fn compute_surface_area(input: &PolyData) -> f64 {
    let mut total_area: f64 = 0.0;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let p0 = input.points.get(cell[0] as usize);

        // Triangle fan decomposition
        for i in 1..(cell.len() - 1) {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);

            let e1: [f64; 3] = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2: [f64; 3] = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

            let cx: f64 = e1[1] * e2[2] - e1[2] * e2[1];
            let cy: f64 = e1[2] * e2[0] - e1[0] * e2[2];
            let cz: f64 = e1[0] * e2[1] - e1[1] * e2[0];

            let tri_area: f64 = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
            total_area += tri_area;
        }
    }

    total_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_right_triangle() {
        // Triangle with vertices at origin, (1,0,0), (0,1,0) -> area = 0.5
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let area: f64 = compute_surface_area(&pd);
        assert!((area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn two_triangles_sum() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 3.0],
                [3.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let area: f64 = compute_surface_area(&pd);
        // First triangle: area = 0.5 * 2 * 2 = 2.0
        // Second triangle: area = 0.5 * |cross((0,0,3),(3,0,0))| = 0.5 * |(0,9,0)| = 4.5
        assert!((area - 6.5).abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let area: f64 = compute_surface_area(&pd);
        assert!((area - 0.0).abs() < 1e-10);
    }
}
