use crate::data::{CellArray, Points, PolyData};

/// Thicken a surface mesh into a solid by extruding along normals.
///
/// Creates inner and outer surfaces offset by `thickness/2` along
/// vertex normals, then connects them with side faces at boundaries.
/// Returns a closed solid mesh.
pub fn thicken(input: &PolyData, thickness: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let half = thickness * 0.5;

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }
    for nm in &mut vnormals {
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }
    }

    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Outer surface (original indices 0..n)
    for i in 0..n {
        let p = input.points.get(i);
        out_pts.push([
            p[0] + vnormals[i][0] * half,
            p[1] + vnormals[i][1] * half,
            p[2] + vnormals[i][2] * half,
        ]);
    }

    // Inner surface (indices n..2n)
    for i in 0..n {
        let p = input.points.get(i);
        out_pts.push([
            p[0] - vnormals[i][0] * half,
            p[1] - vnormals[i][1] * half,
            p[2] - vnormals[i][2] * half,
        ]);
    }

    // Outer faces (same winding)
    for cell in input.polys.iter() {
        out_polys.push_cell(cell);
    }

    // Inner faces (reversed winding)
    for cell in input.polys.iter() {
        let mut rev: Vec<i64> = cell.iter().map(|&id| id + n as i64).collect();
        rev.reverse();
        out_polys.push_cell(&rev);
    }

    // Side faces at boundary edges
    let mut edge_count = std::collections::HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0usize) += 1;
        }
    }
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            // Boundary edge: create quad connecting outer and inner
            out_polys.push_cell(&[a, b, b + n as i64, a + n as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thicken_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = thicken(&pd, 0.2);
        assert_eq!(result.points.len(), 6); // 3 outer + 3 inner
        assert!(result.polys.num_cells() >= 2); // outer + inner + sides
    }

    #[test]
    fn closed_quad_thicken() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = thicken(&pd, 0.5);
        assert_eq!(result.points.len(), 8);
    }

    #[test]
    fn zero_thickness() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = thicken(&pd, 0.0);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = thicken(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
