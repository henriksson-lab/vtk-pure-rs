use crate::data::PolyData;

/// Merge multiple PolyData meshes into a single PolyData.
///
/// Point indices in cells are renumbered to account for the offset of
/// each input's points in the combined output.
pub fn merge(inputs: &[&PolyData]) -> PolyData {
    let mut result = PolyData::new();

    for &pd in inputs {
        let base = result.points.len() as i64;

        // Copy points
        for p in &pd.points {
            result.points.push(p);
        }

        // Copy polys with offset
        for cell in pd.polys.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.polys.push_cell(&offset_cell);
        }

        // Copy lines with offset
        for cell in pd.lines.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.lines.push_cell(&offset_cell);
        }

        // Copy verts with offset
        for cell in pd.verts.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.verts.push_cell(&offset_cell);
        }

        // Copy strips with offset
        for cell in pd.strips.iter() {
            let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            result.strips.push_cell(&offset_cell);
        }
    }

    result
}

/// Merge two PolyData meshes.
pub fn merge_two(a: &PolyData, b: &PolyData) -> PolyData {
    merge(&[a, b])
}

/// Apply a translation to all points in a PolyData (returns a copy).
pub fn translate(pd: &PolyData, dx: f64, dy: f64, dz: f64) -> PolyData {
    let mut result = pd.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result.points.set(i, [p[0] + dx, p[1] + dy, p[2] + dz]);
    }
    result
}

/// Apply uniform scaling to all points (returns a copy).
pub fn scale_uniform(pd: &PolyData, factor: f64) -> PolyData {
    let mut result = pd.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [p[0] * factor, p[1] * factor, p[2] * factor]);
    }
    result
}

/// Apply position + scale to create a transformed copy.
pub fn transform_position_scale(pd: &PolyData, position: [f64; 3], scale: f64) -> PolyData {
    let mut result = pd.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result.points.set(
            i,
            [
                p[0] * scale + position[0],
                p[1] * scale + position[1],
                p[2] * scale + position[2],
            ],
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> PolyData {
        PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn merge_two_meshes() {
        let a = tri();
        let b = PolyData::from_triangles(
            vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let merged = merge_two(&a, &b);
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);
        // Second triangle should have offset indices
        assert_eq!(merged.polys.cell(1), &[3, 4, 5]);
    }

    #[test]
    fn merge_many() {
        let a = tri();
        let b = tri();
        let c = tri();
        let merged = merge(&[&a, &b, &c]);
        assert_eq!(merged.points.len(), 9);
        assert_eq!(merged.polys.num_cells(), 3);
    }

    #[test]
    fn merge_empty() {
        let result = merge(&[]);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn translate_mesh() {
        let pd = tri();
        let moved = translate(&pd, 10.0, 0.0, 0.0);
        let p = moved.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn scale_mesh() {
        let pd = tri();
        let scaled = scale_uniform(&pd, 3.0);
        let p = scaled.points.get(1);
        assert!((p[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn transform_pos_scale() {
        let pd = tri();
        let t = transform_position_scale(&pd, [10.0, 0.0, 0.0], 2.0);
        let p = t.points.get(1);
        assert!((p[0] - 12.0).abs() < 1e-10); // 1.0*2 + 10.0
    }
}
