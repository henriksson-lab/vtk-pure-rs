use crate::data::{CellArray, Points, PolyData};

/// Extract triangles whose face normal points in a given direction.
///
/// Keeps cells whose face normal has a dot product with `direction`
/// above `threshold` (range -1 to 1). Useful for extracting top/bottom/side faces.
pub fn extract_surface_by_normal(
    input: &PolyData,
    direction: [f64; 3],
    threshold: f64,
) -> PolyData {
    let dlen =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dlen < 1e-15 {
        return PolyData::new();
    }
    let dir = [
        direction[0] / dlen,
        direction[1] / dlen,
        direction[2] / dlen,
    ];

    let mut kept_pts = std::collections::HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Compute face normal from first triangle
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let nlen = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nlen < 1e-15 {
            continue;
        }

        let dot = (n[0] * dir[0] + n[1] * dir[1] + n[2] * dir[2]) / nlen;
        if dot >= threshold {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *kept_pts.entry(id).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_upward_faces() {
        let mut pd = PolyData::new();
        // Upward-facing triangle (normal +Z)
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        // Downward-facing triangle (normal -Z)
        pd.points.push([0.0, 0.0, 1.0]);
        pd.points.push([0.0, 1.0, 1.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = extract_surface_by_normal(&pd, [0.0, 0.0, 1.0], 0.5);
        assert_eq!(result.polys.num_cells(), 1); // only upward
    }

    #[test]
    fn extract_all_with_low_threshold() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = extract_surface_by_normal(&pd, [0.0, 0.0, 1.0], -1.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = extract_surface_by_normal(&pd, [0.0, 1.0, 0.0], 0.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
