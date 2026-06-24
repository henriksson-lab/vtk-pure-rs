use crate::data::{AnyDataArray, DataArray, PolyData};

/// Per-face area statistics.
pub struct FaceAreaStats {
    pub min_area: f64,
    pub max_area: f64,
    pub total_area: f64,
    pub mean_area: f64,
}

/// Compute per-face area and add as cell data ("FaceArea").
///
/// For triangles, the exact area formula is used. For polygons with more
/// vertices, the polygon is broken into a triangle fan from the first vertex
/// and areas are summed.
pub fn compute_face_areas(input: &PolyData) -> PolyData {
    let mut areas: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 3 {
            areas.push(0.0);
            continue;
        }

        let p0 = input.points.get(cell[0] as usize);
        let mut area: f64 = 0.0;
        for i in 1..(n - 1) {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);
            // cross product of (p1-p0) x (p2-p0)
            let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cx: f64 = u[1] * v[2] - u[2] * v[1];
            let cy: f64 = u[2] * v[0] - u[0] * v[2];
            let cz: f64 = u[0] * v[1] - u[1] * v[0];
            area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }
        areas.push(area);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("FaceArea", areas, 1)));
    pd
}

/// Compute face area statistics for the whole mesh.
///
/// Returns `None` if the mesh has no polygon cells.
pub fn face_area_stats(input: &PolyData) -> Option<FaceAreaStats> {
    let result = compute_face_areas(input);
    let arr = result.cell_data().get_array("FaceArea")?;
    let count: usize = arr.num_tuples();
    if count == 0 {
        return None;
    }

    let mut min_a: f64 = f64::MAX;
    let mut max_a: f64 = 0.0;
    let mut total: f64 = 0.0;
    let mut buf = [0.0f64];

    for i in 0..count {
        arr.tuple_as_f64(i, &mut buf);
        let a: f64 = buf[0];
        if a < min_a {
            min_a = a;
        }
        if a > max_a {
            max_a = a;
        }
        total += a;
    }

    Some(FaceAreaStats {
        min_area: min_a,
        max_area: max_a,
        total_area: total,
        mean_area: total / count as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_right_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let stats = face_area_stats(&pd).unwrap();
        assert!((stats.total_area - 0.5).abs() < 1e-10);
        assert!((stats.min_area - 0.5).abs() < 1e-10);
        assert!((stats.max_area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn two_triangles_different_areas() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let stats = face_area_stats(&pd).unwrap();
        // Triangle 1: area = 2.0, Triangle 2: area = 0.5
        assert!((stats.min_area - 0.5).abs() < 1e-10);
        assert!((stats.max_area - 2.0).abs() < 1e-10);
        assert!((stats.total_area - 2.5).abs() < 1e-10);
        assert!((stats.mean_area - 1.25).abs() < 1e-10);
    }

    #[test]
    fn face_area_cell_data() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_face_areas(&pd);
        let arr = result.cell_data().get_array("FaceArea").unwrap();
        assert_eq!(arr.num_tuples(), 1);
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        // area of 3-4-5 right triangle = 6.0
        assert!((buf[0] - 6.0).abs() < 1e-10);
    }
}
