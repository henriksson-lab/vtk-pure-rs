use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::data::{CellArray, Points, PolyData};

/// A 4x4 symmetric matrix stored as 10 unique elements (upper triangle).
#[derive(Clone, Copy, Debug)]
struct Quadric {
    a: [f64; 10],
}

impl Quadric {
    fn zero() -> Self {
        Quadric { a: [0.0; 10] }
    }

    /// Build a quadric from a plane equation ax + by + cz + d = 0.
    fn from_plane(nx: f64, ny: f64, nz: f64, d: f64) -> Self {
        Quadric {
            a: [
                nx * nx,
                nx * ny,
                nx * nz,
                nx * d,
                ny * ny,
                ny * nz,
                ny * d,
                nz * nz,
                nz * d,
                d * d,
            ],
        }
    }

    fn add(&self, other: &Quadric) -> Quadric {
        let mut result = Quadric::zero();
        for i in 0..10 {
            result.a[i] = self.a[i] + other.a[i];
        }
        result
    }

    /// Evaluate the quadric error for a given point [x, y, z].
    fn evaluate(&self, p: [f64; 3]) -> f64 {
        let x: f64 = p[0];
        let y: f64 = p[1];
        let z: f64 = p[2];
        // Q = [a0 a1 a2 a3]   v = [x y z 1]^T
        //     [a1 a4 a5 a6]   error = v^T Q v
        //     [a2 a5 a7 a8]
        //     [a3 a6 a8 a9]
        self.a[0] * x * x
            + 2.0 * self.a[1] * x * y
            + 2.0 * self.a[2] * x * z
            + 2.0 * self.a[3] * x
            + self.a[4] * y * y
            + 2.0 * self.a[5] * y * z
            + 2.0 * self.a[6] * y
            + self.a[7] * z * z
            + 2.0 * self.a[8] * z
            + self.a[9]
    }
}

#[derive(Clone, Debug)]
struct EdgeCollapse {
    cost: f64,
    v0: usize,
    v1: usize,
    target: [f64; 3],
}

impl PartialEq for EdgeCollapse {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for EdgeCollapse {}

impl PartialOrd for EdgeCollapse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EdgeCollapse {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: reverse ordering so smallest cost comes first.
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Decimate a triangle mesh using a simplified quadric error metric.
///
/// `target_ratio` is the fraction of faces to keep (0.0 to 1.0).
/// Only works on triangle meshes. Non-triangle polygons are skipped.
pub fn decimate_qem(input: &PolyData, target_ratio: f64) -> PolyData {
    let num_faces: usize = input.polys.num_cells();
    if num_faces == 0 {
        return input.clone();
    }

    let target_faces: usize = ((num_faces as f64 * target_ratio.clamp(0.0, 1.0)).ceil()) as usize;
    if target_faces >= num_faces {
        return input.clone();
    }

    let npts: usize = input.points.len();

    // Copy points into a mutable vec.
    let mut positions: Vec<[f64; 3]> = Vec::with_capacity(npts);
    for i in 0..npts {
        positions.push(input.points.get(i));
    }

    // Build face list (only triangles).
    let mut faces: Vec<[usize; 3]> = Vec::new();
    let mut face_alive: Vec<bool> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            faces.push([cell[0] as usize, cell[1] as usize, cell[2] as usize]);
            face_alive.push(true);
        }
    }

    let mut alive_count: usize = faces.len();
    if target_faces >= alive_count {
        return input.clone();
    }

    // Compute per-vertex quadrics from face planes.
    let mut quadrics: Vec<Quadric> = vec![Quadric::zero(); npts];
    for f in &faces {
        let p0 = positions[f[0]];
        let p1 = positions[f[1]];
        let p2 = positions[f[2]];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx: f64 = e1[1] * e2[2] - e1[2] * e2[1];
        let ny: f64 = e1[2] * e2[0] - e1[0] * e2[2];
        let nz: f64 = e1[0] * e2[1] - e1[1] * e2[0];
        let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-15 {
            continue;
        }
        let nx: f64 = nx / len;
        let ny: f64 = ny / len;
        let nz: f64 = nz / len;
        let d: f64 = -(nx * p0[0] + ny * p0[1] + nz * p0[2]);
        let q = Quadric::from_plane(nx, ny, nz, d);
        quadrics[f[0]] = quadrics[f[0]].add(&q);
        quadrics[f[1]] = quadrics[f[1]].add(&q);
        quadrics[f[2]] = quadrics[f[2]].add(&q);
    }

    // Build edge set and vertex-to-face adjacency.
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); npts];
    for (fi, f) in faces.iter().enumerate() {
        for k in 0..3 {
            let a: usize = f[k];
            let b: usize = f[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
            vert_faces[a].push(fi);
        }
    }

    // Mapping from vertex to its representative (for collapsed vertices).
    let mut rep: Vec<usize> = (0..npts).collect();

    fn find_rep(rep: &mut Vec<usize>, v: usize) -> usize {
        let mut r: usize = v;
        while rep[r] != r {
            rep[r] = rep[rep[r]];
            r = rep[r];
        }
        r
    }

    // Build priority queue.
    let mut heap: BinaryHeap<EdgeCollapse> = BinaryHeap::new();
    for &(a, b) in &edges {
        let q = quadrics[a].add(&quadrics[b]);
        let mid = [
            (positions[a][0] + positions[b][0]) * 0.5,
            (positions[a][1] + positions[b][1]) * 0.5,
            (positions[a][2] + positions[b][2]) * 0.5,
        ];
        let cost: f64 = q.evaluate(mid).abs();
        heap.push(EdgeCollapse {
            cost,
            v0: a,
            v1: b,
            target: mid,
        });
    }

    // Collapse loop.
    while alive_count > target_faces {
        let collapse = match heap.pop() {
            Some(c) => c,
            None => break,
        };

        let ra: usize = find_rep(&mut rep, collapse.v0);
        let rb: usize = find_rep(&mut rep, collapse.v1);
        if ra == rb {
            continue; // Already merged.
        }

        // Collapse rb into ra.
        rep[rb] = ra;
        positions[ra] = collapse.target;
        quadrics[ra] = quadrics[ra].add(&quadrics[rb]);

        // Update faces: replace rb with ra, mark degenerate faces dead.
        let rb_faces: Vec<usize> = vert_faces[rb].clone();
        for &fi in &rb_faces {
            if !face_alive[fi] {
                continue;
            }
            for k in 0..3 {
                if find_rep(&mut rep, faces[fi][k]) == rb || faces[fi][k] == rb {
                    faces[fi][k] = ra;
                }
                let r = find_rep(&mut rep, faces[fi][k]);
                faces[fi][k] = r;
            }
            // Check for degenerate triangle (two or more identical vertices).
            if faces[fi][0] == faces[fi][1]
                || faces[fi][1] == faces[fi][2]
                || faces[fi][0] == faces[fi][2]
            {
                face_alive[fi] = false;
                alive_count -= 1;
            }
            vert_faces[ra].push(fi);
        }

        // Re-insert edges from ra to its neighbors.
        let neighbors: HashSet<usize> = vert_faces[ra]
            .iter()
            .filter(|&&fi| face_alive[fi])
            .flat_map(|&fi| faces[fi].iter().copied())
            .filter(|&v| v != ra)
            .collect();

        for nb in neighbors {
            let q = quadrics[ra].add(&quadrics[nb]);
            let mid = [
                (positions[ra][0] + positions[nb][0]) * 0.5,
                (positions[ra][1] + positions[nb][1]) * 0.5,
                (positions[ra][2] + positions[nb][2]) * 0.5,
            ];
            let cost: f64 = q.evaluate(mid).abs();
            heap.push(EdgeCollapse {
                cost,
                v0: ra,
                v1: nb,
                target: mid,
            });
        }
    }

    // Build output: compact vertices and faces.
    let mut used: HashMap<usize, usize> = HashMap::new();
    let mut out_points: Points<f64> = Points::new();
    let mut out_polys: CellArray = CellArray::new();

    for (fi, f) in faces.iter().enumerate() {
        if !face_alive[fi] {
            continue;
        }
        let mut tri = [0i64; 3];
        for k in 0..3 {
            let r: usize = find_rep(&mut rep, f[k]);
            let new_id = if let Some(&id) = used.get(&r) {
                id
            } else {
                let id: usize = out_points.len();
                out_points.push(positions[r]);
                used.insert(r, id);
                id
            };
            tri[k] = new_id as i64;
        }
        out_polys.push_cell(&tri);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad_mesh() -> PolyData {
        // A quad made of 2 triangles: 4 vertices, 2 faces.
        let mut points: Points<f64> = Points::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([1.0, 1.0, 0.0]);
        points.push([0.0, 1.0, 0.0]);
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        polys.push_cell(&[0, 2, 3]);
        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd
    }

    #[test]
    fn ratio_one_keeps_all() {
        let mesh = make_quad_mesh();
        let result = decimate_qem(&mesh, 1.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn decimation_reduces_faces() {
        // Build an 8-face mesh (octahedron-like).
        let mut points: Points<f64> = Points::new();
        points.push([0.0, 0.0, 1.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.0, 1.0, 0.0]);
        points.push([-1.0, 0.0, 0.0]);
        points.push([0.0, -1.0, 0.0]);
        points.push([0.0, 0.0, -1.0]);
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        polys.push_cell(&[0, 2, 3]);
        polys.push_cell(&[0, 3, 4]);
        polys.push_cell(&[0, 4, 1]);
        polys.push_cell(&[5, 2, 1]);
        polys.push_cell(&[5, 3, 2]);
        polys.push_cell(&[5, 4, 3]);
        polys.push_cell(&[5, 1, 4]);
        let mut mesh = PolyData::new();
        mesh.points = points;
        mesh.polys = polys;
        let result = decimate_qem(&mesh, 0.5);
        assert!(result.polys.num_cells() < 8);
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let mesh = PolyData::default();
        let result = decimate_qem(&mesh, 0.5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
