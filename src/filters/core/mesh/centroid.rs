use crate::data::PolyData;

/// Compute the centroid of a PolyData as the simple average of all points.
///
/// Returns [0.0, 0.0, 0.0] if the mesh has no points.
pub fn compute_centroid(input: &PolyData) -> [f64; 3] {
    let n = input.points.len();
    if n == 0 {
        return [0.0, 0.0, 0.0];
    }

    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;

    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }

    let nf: f64 = n as f64;
    [cx / nf, cy / nf, cz / nf]
}

/// Compute the area-weighted centroid of a PolyData.
///
/// Each polygon is triangulated as a fan and each fan triangle contributes its
/// centroid weighted by its area.
/// Returns [0.0, 0.0, 0.0] if no polygons exist or total area is zero.
pub fn compute_area_weighted_centroid(input: &PolyData) -> [f64; 3] {
    let mut total_area: f64 = 0.0;
    let mut wx: f64 = 0.0;
    let mut wy: f64 = 0.0;
    let mut wz: f64 = 0.0;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let p0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);
            let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cx: f64 = u[1] * v[2] - u[2] * v[1];
            let cy: f64 = u[2] * v[0] - u[0] * v[2];
            let cz: f64 = u[0] * v[1] - u[1] * v[0];
            let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
            let tri_centroid = [
                (p0[0] + p1[0] + p2[0]) / 3.0,
                (p0[1] + p1[1] + p2[1]) / 3.0,
                (p0[2] + p1[2] + p2[2]) / 3.0,
            ];

            total_area += area;
            wx += tri_centroid[0] * area;
            wy += tri_centroid[1] * area;
            wz += tri_centroid[2] * area;
        }
    }

    if total_area < 1e-30 {
        return [0.0, 0.0, 0.0];
    }

    [wx / total_area, wy / total_area, wz / total_area]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centroid_of_unit_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let c = compute_centroid(&pd);
        assert!((c[0] - 1.0).abs() < 1e-10);
        assert!((c[1] - 1.0).abs() < 1e-10);
        assert!(c[2].abs() < 1e-10);
    }

    #[test]
    fn empty_mesh_centroid() {
        let pd = PolyData::default();
        let c = compute_centroid(&pd);
        assert!((c[0]).abs() < 1e-10);
        assert!((c[1]).abs() < 1e-10);
        assert!((c[2]).abs() < 1e-10);
    }

    #[test]
    fn area_weighted_single_triangle() {
        // For a single triangle, area-weighted centroid should equal the face centroid
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [6.0, 0.0, 0.0], [0.0, 6.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let c = compute_area_weighted_centroid(&pd);
        assert!((c[0] - 2.0).abs() < 1e-10, "x = {}", c[0]);
        assert!((c[1] - 2.0).abs() < 1e-10, "y = {}", c[1]);
        assert!(c[2].abs() < 1e-10);
    }

    #[test]
    fn area_weighted_polygon_uses_triangle_fan_centroids() {
        let pd = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 1.0, 0.0],
                [0.0, 3.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3]],
        );
        let c = compute_area_weighted_centroid(&pd);
        assert!((c[0] - 1.7777777777777777).abs() < 1e-10, "x = {}", c[0]);
        assert!((c[1] - 1.0).abs() < 1e-10, "y = {}", c[1]);
        assert!(c[2].abs() < 1e-10);
    }
}
