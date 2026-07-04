use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Extrude a surface mesh along its face normals by a given distance.
///
/// Creates a thickened solid from a surface by duplicating the surface (offset
/// along vertex normals) and connecting the two copies with side-wall quads
/// along boundary edges.
pub fn extrude_along_normals(input: &PolyData, distance: f64) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let vertex_normals = extract_vertex_normals(input);

    // Build output points: original points + offset points
    let mut points = Points::<f64>::new();
    for i in 0..n {
        points.push(input.points.get(i));
    }
    for i in 0..n {
        let p = input.points.get(i);
        let vn = vertex_normals[i];
        points.push([
            p[0] + vn[0] * distance,
            p[1] + vn[1] * distance,
            p[2] + vn[2] * distance,
        ]);
    }

    let mut polys = CellArray::new();

    // Outer surface: original faces
    for cell in input.polys.iter() {
        if valid_point_ids(cell, n).is_some() {
            polys.push_cell(cell);
        }
    }

    // Translated cap: VTK's linear extrusion keeps the input winding.
    for cell in input.polys.iter() {
        if let Some(ids) = valid_point_ids(cell, n) {
            let ids: Vec<i64> = ids.into_iter().map(|id| id as i64 + n as i64).collect();
            polys.push_cell(&ids);
        }
    }

    // Find boundary edges (edges with only one adjacent face) for side walls
    let boundary_edges = find_boundary_edges(input);

    // Create side-wall quads for each boundary edge
    for &(a, b) in &boundary_edges {
        let a_off: i64 = a as i64 + n as i64;
        let b_off: i64 = b as i64 + n as i64;
        polys.push_cell(&[a as i64, b as i64, b_off, a_off]);
    }

    let mut output = PolyData::new();
    output.points = points;
    output.polys = polys;
    output
}

fn extract_vertex_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let n = input.points.len();
    if let Some(normals) = input.point_data().normals() {
        if normals.num_components() == 3 && normals.num_tuples() == n {
            let mut result = Vec::with_capacity(n);
            let mut tuple = [0.0; 3];
            for i in 0..n {
                normals.tuple_as_f64(i, &mut tuple);
                result.push(tuple);
            }
            return result;
        }
    }

    compute_vertex_normals(input)
}

fn compute_vertex_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let n: usize = input.points.len();
    let mut normals: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];

    for cell in input.polys.iter() {
        let Some(ids) = valid_point_ids(cell, n) else {
            continue;
        };
        if cell.len() < 3 {
            continue;
        }
        // Compute face normal via Newell's method
        let mut nx: f64 = 0.0;
        let mut ny: f64 = 0.0;
        let mut nz: f64 = 0.0;
        let len: usize = ids.len();
        for j in 0..len {
            let pi = input.points.get(ids[j]);
            let pj = input.points.get(ids[(j + 1) % len]);
            nx += (pi[1] - pj[1]) * (pi[2] + pj[2]);
            ny += (pi[2] - pj[2]) * (pi[0] + pj[0]);
            nz += (pi[0] - pj[0]) * (pi[1] + pj[1]);
        }
        // Accumulate (unnormalized) face normal to each vertex
        for &idx in &ids {
            normals[idx][0] += nx;
            normals[idx][1] += ny;
            normals[idx][2] += nz;
        }
    }

    // Normalize
    for normal in normals.iter_mut() {
        let mag: f64 =
            (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if mag > 1e-12 {
            normal[0] /= mag;
            normal[1] /= mag;
            normal[2] /= mag;
        } else {
            *normal = [0.0, 0.0, 1.0];
        }
    }

    normals
}

fn find_boundary_edges(input: &PolyData) -> Vec<(usize, usize)> {
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    let mut ordered_edges: Vec<(usize, usize)> = Vec::new();

    for cell in input.polys.iter() {
        let Some(ids) = valid_point_ids(cell, input.points.len()) else {
            continue;
        };
        let len: usize = ids.len();
        for j in 0..len {
            let a: usize = ids[j];
            let b: usize = ids[(j + 1) % len];
            let key = if a < b { (a, b) } else { (b, a) };
            let count = edge_count.entry(key).or_insert(0);
            if *count == 0 {
                ordered_edges.push((a, b));
            }
            *count += 1;
        }
    }

    let mut boundary = Vec::new();
    for (a, b) in ordered_edges {
        let key = if a < b { (a, b) } else { (b, a) };
        if edge_count[&key] == 1 {
            boundary.push((a, b));
        }
    }
    boundary
}

fn valid_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    fn make_single_triangle() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd
    }

    fn make_two_triangles() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd
    }

    #[test]
    fn extrude_single_triangle() {
        let tri = make_single_triangle();
        let result = extrude_along_normals(&tri, 1.0);

        // 3 original + 3 offset points
        assert_eq!(result.points.len(), 6);

        // 1 outer face + 1 inner face + 3 boundary edge side-wall quads
        assert_eq!(result.polys.num_cells(), 5);
        assert_eq!(result.polys.cell(1), &[3, 4, 5]);

        // Verify offset points are at z=1 (normal of XY triangle is +Z)
        for i in 3..6 {
            let p = result.points.get(i);
            assert!((p[2] - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn extrude_two_triangles_shared_edge() {
        let quad = make_two_triangles();
        let result = extrude_along_normals(&quad, 0.5);

        // 4 original + 4 offset
        assert_eq!(result.points.len(), 8);

        // 2 outer + 2 inner + 4 boundary edges (shared edge is interior)
        assert_eq!(result.polys.num_cells(), 8);
    }

    #[test]
    fn extrude_negative_distance() {
        let tri = make_single_triangle();
        let result = extrude_along_normals(&tri, -0.5);

        // Offset points should be at z = -0.5
        for i in 3..6 {
            let p = result.points.get(i);
            assert!((p[2] - (-0.5)).abs() < 1e-10);
        }
    }

    #[test]
    fn extrude_uses_supplied_point_normals() {
        let mut tri = make_single_triangle();
        tri.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));
        tri.point_data_mut().set_active_normals("Normals");

        let result = extrude_along_normals(&tri, 2.0);

        for i in 0..3 {
            let p = result.points.get(i);
            let q = result.points.get(i + 3);
            assert!((q[0] - (p[0] + 2.0)).abs() < 1e-10);
            assert!((q[1] - p[1]).abs() < 1e-10);
            assert!((q[2] - p[2]).abs() < 1e-10);
        }
    }
}
