use crate::data::PolyData;
use std::collections::HashMap;
const EPSILON: f64 = 1e-10;

/// Check if a closed triangle mesh is convex.
///
/// A mesh is convex if all dihedral angles between adjacent faces
/// are <= 180 degrees (all edges are "valley" edges, not "ridge").
pub fn is_convex(input: &PolyData) -> bool {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    if has_concave_edge(input, &cells) {
        return false;
    }

    for cell in &cells {
        if cell.len() < 3 {
            continue;
        }
        let Some((origin, normal)) = face_plane(input, cell) else {
            continue;
        };

        let mut positive = false;
        let mut negative = false;
        for point_id in 0..input.points.len() {
            if cell.iter().any(|&id| id as usize == point_id) {
                continue;
            }
            let p = input.points.get(point_id);
            let side = dot(sub(p, origin), normal);
            if side > EPSILON {
                positive = true;
            } else if side < -EPSILON {
                negative = true;
            }
            if positive && negative {
                return false;
            }
        }
    }
    true
}

/// Compute convexity defect: fraction of edges that are concave.
pub fn convexity_defect(input: &PolyData) -> f64 {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let center = mesh_centroid(input);
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|cell| {
            face_plane(input, cell).map_or([0.0; 3], |(origin, normal)| {
                orient_normal(origin, normal, center)
            })
        })
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (face_id, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            edge_faces
                .entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(face_id);
        }
    }

    let mut total = 0;
    let mut concave = 0;
    for ((a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            total += 1;
            concave += 1;
            continue;
        }
        total += 1;
        if edge_is_concave(input, &cells, &normals, faces[0], faces[1], *a, *b) {
            concave += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        concave as f64 / total as f64
    }
}

fn has_concave_edge(input: &PolyData, cells: &[Vec<i64>]) -> bool {
    !concave_edge_faces(input, cells).is_empty()
}

fn concave_edge_faces(input: &PolyData, cells: &[Vec<i64>]) -> std::collections::HashSet<usize> {
    let center = mesh_centroid(input);
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|cell| {
            face_plane(input, cell).map_or([0.0; 3], |(origin, normal)| {
                orient_normal(origin, normal, center)
            })
        })
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (face_id, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            edge_faces
                .entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(face_id);
        }
    }

    let mut concave = std::collections::HashSet::new();
    for ((a, b), faces) in edge_faces {
        if faces.len() != 2 {
            concave.extend(faces);
            continue;
        }
        if edge_is_concave(input, cells, &normals, faces[0], faces[1], a, b) {
            concave.insert(faces[0]);
            concave.insert(faces[1]);
        }
    }
    concave
}

fn edge_is_concave(
    input: &PolyData,
    cells: &[Vec<i64>],
    normals: &[[f64; 3]],
    face_a: usize,
    face_b: usize,
    edge_a: i64,
    edge_b: i64,
) -> bool {
    point_is_in_front_of_face(input, cells, normals, face_a, face_b, edge_a, edge_b)
        || point_is_in_front_of_face(input, cells, normals, face_b, face_a, edge_a, edge_b)
}

fn point_is_in_front_of_face(
    input: &PolyData,
    cells: &[Vec<i64>],
    normals: &[[f64; 3]],
    plane_face: usize,
    point_face: usize,
    edge_a: i64,
    edge_b: i64,
) -> bool {
    let Some(&opposite_id) = cells[point_face]
        .iter()
        .find(|&&v| v != edge_a && v != edge_b)
    else {
        return false;
    };
    let Some(opposite_id) = valid_point_id(input, opposite_id) else {
        return false;
    };
    let Some(edge_a) = valid_point_id(input, edge_a) else {
        return false;
    };
    let p = input.points.get(opposite_id);
    let v0 = input.points.get(edge_a);
    dot(sub(p, v0), normals[plane_face]) > EPSILON
}

fn face_plane(input: &PolyData, cell: &[i64]) -> Option<([f64; 3], [f64; 3])> {
    if cell.len() < 3 || !valid_cell_points(input, cell) {
        return None;
    }

    let origin = input.points.get(cell[0] as usize);
    for i in 1..cell.len() - 1 {
        let v1 = input.points.get(cell[i] as usize);
        let v2 = input.points.get(cell[i + 1] as usize);
        let normal = cross(sub(v1, origin), sub(v2, origin));
        let length = dot(normal, normal).sqrt();
        if length > 1e-15 {
            return Some((
                origin,
                [normal[0] / length, normal[1] / length, normal[2] / length],
            ));
        }
    }
    None
}

fn mesh_centroid(input: &PolyData) -> [f64; 3] {
    if input.points.len() == 0 {
        return [0.0; 3];
    }

    let mut center = [0.0; 3];
    for point_id in 0..input.points.len() {
        let p = input.points.get(point_id);
        center[0] += p[0];
        center[1] += p[1];
        center[2] += p[2];
    }
    let inv_n = 1.0 / input.points.len() as f64;
    [center[0] * inv_n, center[1] * inv_n, center[2] * inv_n]
}

fn orient_normal(origin: [f64; 3], normal: [f64; 3], center: [f64; 3]) -> [f64; 3] {
    if dot(sub(center, origin), normal) > 0.0 {
        [-normal[0], -normal[1], -normal[2]]
    } else {
        normal
    }
}

fn valid_cell_points(input: &PolyData, cell: &[i64]) -> bool {
    cell.iter().all(|&id| valid_point_id(input, id).is_some())
}

fn valid_point_id(input: &PolyData, point_id: i64) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < input.points.len())
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convexity_basic() {
        // Simple test: just check the function runs without panic
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let d = convexity_defect(&pd);
        assert!(d >= 0.0 && d <= 1.0);
    }

    #[test]
    fn non_convex() {
        let mut pd = PolyData::new();
        // Create a concavity by pushing center vertex inward
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([1.0, 1.0, -1.0]); // concave vertex
        pd.polys.push_cell(&[0, 1, 4]);
        pd.polys.push_cell(&[1, 2, 4]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[3, 0, 4]);

        assert!(!is_convex(&pd));
        assert!(convexity_defect(&pd) > 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(is_convex(&pd));
    }

    #[test]
    fn convex_tetrahedron_allows_mixed_winding() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert!(is_convex(&pd));
        assert_eq!(convexity_defect(&pd), 0.0);
    }
}
