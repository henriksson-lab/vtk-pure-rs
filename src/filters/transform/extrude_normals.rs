use crate::data::{CellArray, Points, PolyData};

/// Extrude a surface along its vertex normals.
///
/// Each point is displaced along its normal by `distance`. Side quads
/// are generated between the original and displaced edges. Optionally
/// caps the ends.
pub fn extrude_along_normals(input: &PolyData, distance: f64, capping: bool) -> PolyData {
    let n = input.points.len();
    let normals = extract_normals(input);

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Original points
    for i in 0..n {
        out_points.push(input.points.get(i));
    }
    // Displaced points
    for (i, nm) in normals.iter().enumerate() {
        let p = input.points.get(i);
        out_points.push([
            p[0] + nm[0] * distance,
            p[1] + nm[1] * distance,
            p[2] + nm[2] * distance,
        ]);
    }

    let offset = n as i64;

    // Side quads
    for cell in input.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            out_polys.push_cell(&[a, b, b + offset, a + offset]);
        }
    }

    if capping {
        for cell in input.polys.iter() {
            out_polys.push_cell(cell);
        }
        for cell in input.polys.iter() {
            let reversed: Vec<i64> = cell.iter().rev().map(|&id| id + offset).collect();
            out_polys.push_cell(&reversed);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn extract_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let n = input.points.len();
    if let Some(normals_arr) = input.point_data().normals() {
        if normals_arr.num_components() == 3 && normals_arr.num_tuples() == n {
            let mut result = Vec::with_capacity(n);
            let mut buf = [0.0f64; 3];
            for i in 0..n {
                normals_arr.tuple_as_f64(i, &mut buf);
                result.push(buf);
            }
            return result;
        }
    }
    // Fallback: compute face normals averaged at vertices
    let mut normals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let p0 = input.points.get(cell[0] as usize);
        let p1 = input.points.get(cell[1] as usize);
        let p2 = input.points.get(cell[2] as usize);
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &id in cell {
            let nm = &mut normals[id as usize];
            nm[0] += fn_[0];
            nm[1] += fn_[1];
            nm[2] += fn_[2];
        }
    }
    for nm in &mut normals {
        let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if len > 1e-20 {
            nm[0] /= len;
            nm[1] /= len;
            nm[2] /= len;
        }
    }
    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrude_flat_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extrude_along_normals(&pd, 1.0, true);
        assert_eq!(result.points.len(), 6);
        // Displaced points should be at z ≈ 1 (normal is +z for this triangle)
        let p = result.points.get(3);
        assert!((p[2] - 1.0).abs() < 0.5, "z = {}", p[2]);
    }

    #[test]
    fn extrude_no_cap() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extrude_along_normals(&pd, 2.0, false);
        assert_eq!(result.polys.num_cells(), 3); // just side quads
    }
}
