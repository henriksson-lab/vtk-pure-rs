//! Build topology dual: vertices become faces, faces become vertices.
use crate::data::{CellArray, Points, PolyData};
pub fn topology_dual(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nv = mesh.points.len();
    // Face centroids become new vertices
    let mut pts = Points::<f64>::new();
    for c in &cells {
        if c.is_empty() {
            pts.push([0.0, 0.0, 0.0]);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let mut count = 0usize;
        for &v in c {
            if v < 0 || v as usize >= nv {
                continue;
            }
            let p = mesh.points.get(v as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
            count += 1;
        }
        if count == 0 {
            pts.push([0.0, 0.0, 0.0]);
            continue;
        }
        let n = count as f64;
        pts.push([cx / n, cy / n, cz / n]);
    }
    // For each original vertex, find adjacent faces -> new face
    let mut vf: Vec<Vec<usize>> = vec![Vec::new(); nv];
    for (ci, c) in cells.iter().enumerate() {
        for &v in c {
            if v < 0 || v as usize >= nv {
                continue;
            }
            vf[v as usize].push(ci);
        }
    }
    let mut polys = CellArray::new();
    for (vi, faces) in vf.iter().enumerate() {
        if faces.len() < 3 {
            continue;
        }
        let mut ids: Vec<i64> = order_adjacent_faces_around_vertex(vi, faces, &cells)
            .into_iter()
            .map(|fi| fi as i64)
            .collect();
        if ids.len() != faces.len() {
            ids = faces.iter().map(|&fi| fi as i64).collect();
        }
        if ids.len() >= 3 {
            polys.push_cell(&ids);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

fn order_adjacent_faces_around_vertex(
    vertex: usize,
    faces: &[usize],
    cells: &[Vec<i64>],
) -> Vec<usize> {
    let mut edge_faces: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for &face in faces {
        let cell = &cells[face];
        let Some(pos) = cell.iter().position(|&v| v >= 0 && v as usize == vertex) else {
            continue;
        };
        let len = cell.len();
        let prev = cell[(pos + len - 1) % len];
        let next = cell[(pos + 1) % len];
        if prev >= 0 {
            edge_faces.entry(prev as usize).or_default().push(face);
        }
        if next >= 0 {
            edge_faces.entry(next as usize).or_default().push(face);
        }
    }

    let mut neighbors: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for incident_faces in edge_faces.values() {
        for i in 0..incident_faces.len() {
            for j in (i + 1)..incident_faces.len() {
                let a = incident_faces[i];
                let b = incident_faces[j];
                neighbors.entry(a).or_default().push(b);
                neighbors.entry(b).or_default().push(a);
            }
        }
    }

    let mut ordered = Vec::with_capacity(faces.len());
    let mut used = std::collections::HashSet::with_capacity(faces.len());
    let face_set: std::collections::HashSet<usize> = faces.iter().copied().collect();
    let mut cur = faces
        .iter()
        .copied()
        .find(|face| neighbors.get(face).map_or(0, Vec::len) <= 1)
        .unwrap_or(faces[0]);

    while ordered.len() < faces.len() && !used.contains(&cur) {
        ordered.push(cur);
        used.insert(cur);
        let next = neighbors
            .get(&cur)
            .into_iter()
            .flatten()
            .copied()
            .find(|f| face_set.contains(f) && !used.contains(f));
        match next {
            Some(next) => cur = next,
            None => break,
        }
    }

    ordered
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let d = topology_dual(&m);
        assert_eq!(d.points.len(), 3);
        assert!(d.polys.num_cells() >= 1);
    }
}
