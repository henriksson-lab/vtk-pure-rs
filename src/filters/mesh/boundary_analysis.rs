//! Boundary analysis: extract boundary loops, measure perimeters.

use crate::data::{CellArray, Points, PolyData};

/// Extract all boundary loops, one `PolyData` per loop.
///
/// Thin wrapper over the single loop-tracing implementation in
/// [`crate::filters::mesh::extract_boundary::extract_boundary_loops`], which
/// returns every loop as a line cell of one `PolyData`; this entry point splits
/// that result into one `PolyData` per loop.
pub fn extract_boundary_loops(mesh: &PolyData) -> Vec<PolyData> {
    let traced = crate::filters::mesh::extract_boundary::extract_boundary_loops(mesh);
    (0..traced.lines.num_cells())
        .map(|c| {
            let cell = traced.lines.cell(c);
            // Traced loops repeat the first vertex as the last one; drop it so
            // each output holds the loop vertices exactly once.
            let verts = if cell.len() >= 2 && cell.first() == cell.last() {
                &cell[..cell.len() - 1]
            } else {
                cell
            };

            let mut pts = Points::<f64>::new();
            let mut lines = CellArray::new();
            let ids: Vec<i64> = verts
                .iter()
                .enumerate()
                .map(|(i, &vi)| {
                    pts.push(traced.points.get(vi as usize));
                    i as i64
                })
                .collect();
            if ids.len() >= 2 {
                let mut closed = ids.clone();
                closed.push(ids[0]);
                lines.push_cell(&closed);
            }
            let mut m = PolyData::new();
            m.points = pts;
            m.lines = lines;
            m
        })
        .collect()
}

/// Compute the perimeter of each boundary loop.
pub fn boundary_perimeters(mesh: &PolyData) -> Vec<f64> {
    let loops = find_boundary_loops(mesh);
    loops
        .iter()
        .map(|loop_v| {
            let mut perim = 0.0;
            for i in 0..loop_v.len() {
                let a = mesh.points.get(loop_v[i]);
                let b = mesh.points.get(loop_v[(i + 1) % loop_v.len()]);
                perim +=
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            }
            perim
        })
        .collect()
}

/// Classify boundary type: "open" (has boundary) or "closed" (no boundary).
pub fn boundary_classification(mesh: &PolyData) -> &'static str {
    if find_boundary_loops(mesh).is_empty() {
        "closed"
    } else {
        "open"
    }
}

/// Add a "BoundaryDistance" point data: distance from the nearest boundary vertex.
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::mesh::mesh_distance_field_to_scalar::boundary_distance_field`],
/// re-labelling its output array to this module's `"BoundaryDistance"` name.
pub fn boundary_distance_field(mesh: &PolyData) -> PolyData {
    let mut result =
        crate::filters::mesh::mesh_distance_field_to_scalar::boundary_distance_field(mesh);
    if let Some(mut arr) = result.point_data_mut().remove_array("BoundaryDist") {
        arr.set_name("BoundaryDistance");
        result.point_data_mut().add_array(arr);
        result
            .point_data_mut()
            .set_active_scalars("BoundaryDistance");
    }
    result
}

fn find_boundary_loops(mesh: &PolyData) -> Vec<Vec<usize>> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= mesh.points.len() || b >= mesh.points.len() {
                continue;
            }
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let bnd: Vec<(usize, usize)> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    if bnd.is_empty() {
        return Vec::new();
    }
    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &bnd {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    for nbs in adj.values_mut() {
        nbs.sort_unstable();
        nbs.dedup();
    }

    let mut visited_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    let mut loops = Vec::new();
    for &(a, b) in &bnd {
        let key = edge_key(a, b);
        if visited_edges.contains(&key) {
            continue;
        }
        let mut path = vec![a, b];
        visited_edges.insert(key);
        extend_boundary_path(&adj, &mut visited_edges, &mut path, false);
        extend_boundary_path(&adj, &mut visited_edges, &mut path, true);
        if path.len() > 1 && path[0] == *path.last().unwrap() {
            path.pop();
        }
        if path.len() >= 3 {
            loops.push(path);
        }
    }
    loops
}

fn extend_boundary_path(
    adj: &std::collections::HashMap<usize, Vec<usize>>,
    visited_edges: &mut std::collections::HashSet<(usize, usize)>,
    path: &mut Vec<usize>,
    prepend: bool,
) {
    loop {
        let (prev, cur) = if prepend {
            (path[1], path[0])
        } else {
            (path[path.len() - 2], path[path.len() - 1])
        };
        let Some(nbs) = adj.get(&cur) else {
            break;
        };
        let next = nbs
            .iter()
            .copied()
            .find(|&nb| nb != prev && !visited_edges.contains(&edge_key(cur, nb)));
        let Some(nb) = next else {
            break;
        };
        visited_edges.insert(edge_key(cur, nb));
        if prepend {
            path.insert(0, nb);
        } else {
            path.push(nb);
        }
    }
}

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_tri_boundary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let loops = extract_boundary_loops(&mesh);
        assert_eq!(loops.len(), 1);
        let perimeters = boundary_perimeters(&mesh);
        assert!(perimeters[0] > 2.0);
        assert_eq!(boundary_classification(&mesh), "open");
    }
    #[test]
    fn closed_tet() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        assert_eq!(boundary_classification(&mesh), "closed");
        assert!(extract_boundary_loops(&mesh).is_empty());
    }
    #[test]
    fn distance_field() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = boundary_distance_field(&mesh);
        let arr = result.point_data().get_array("BoundaryDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // corner is boundary
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] > 0.0); // center
    }
}
