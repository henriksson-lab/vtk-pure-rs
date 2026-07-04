use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

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
    let mut out_lines = CellArray::new();
    let mut out_strips = CellArray::new();

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

    for cell in input.verts.iter() {
        for &pt_id in cell {
            out_lines.push_cell(&[pt_id, pt_id + offset]);
        }
    }

    for cell in input.lines.iter() {
        for i in 0..cell.len().saturating_sub(1) {
            let a = cell[i];
            let b = cell[i + 1];
            out_strips.push_cell(&[a, b, a + offset, b + offset]);
        }
    }

    if capping {
        for cell in input.polys.iter() {
            out_polys.push_cell(cell);
        }
        for cell in input.polys.iter() {
            let extruded: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            out_polys.push_cell(&extruded);
        }
    }

    for (a, b) in boundary_edges(input) {
        out_strips.push_cell(&[a, b, a + offset, b + offset]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.polys = out_polys;
    pd.strips = out_strips;
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
    vec![[0.0, 0.0, 1.0]; n]
}

fn boundary_edges(input: &PolyData) -> Vec<(i64, i64)> {
    let mut edge_counts: HashMap<(i64, i64), usize> = HashMap::new();
    let mut ordered_edges = Vec::new();

    for cell in input.polys.iter().chain(input.strips.iter()) {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_counts
                .entry(key)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            ordered_edges.push((a, b, key));
        }
    }

    ordered_edges
        .into_iter()
        .filter_map(|(a, b, key)| (edge_counts[&key] == 1).then_some((a, b)))
        .collect()
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
        assert_eq!(result.strips.num_cells(), 3); // just side strips
    }
}
