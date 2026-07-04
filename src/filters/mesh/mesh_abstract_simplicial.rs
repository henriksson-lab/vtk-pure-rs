//! Abstract simplicial complex operations on mesh.
use crate::data::PolyData;
pub struct SimplicialComplex {
    pub vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub triangles: Vec<(usize, usize, usize)>,
}

pub fn extract_simplicial_complex(mesh: &PolyData) -> SimplicialComplex {
    let n = mesh.points.len();
    let mut edges = std::collections::HashSet::new();
    let mut triangles = Vec::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 || !cell.iter().all(|&id| id >= 0 && (id as usize) < n) {
            continue;
        }
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edges.insert((a.min(b), a.max(b)));
        }
        if nc == 3 {
            let mut t = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            t.sort();
            triangles.push((t[0], t[1], t[2]));
        }
    }
    SimplicialComplex {
        vertices: n,
        edges: edges.into_iter().collect(),
        triangles,
    }
}

pub fn betti_numbers(sc: &SimplicialComplex) -> (usize, usize, usize) {
    let v = sc.vertices;
    let e = sc.edges.len();
    let f = sc.triangles.len();
    // Euler characteristic = V - E + F = b0 - b1 + b2
    let chi = v as isize - e as isize + f as isize;
    let components = connected_components(sc);
    let b0 = components.len();
    let edge_face_counts = triangle_edge_face_counts(sc);
    let b2 = components
        .iter()
        .filter(|component| {
            let has_triangle = sc
                .triangles
                .iter()
                .any(|&(a, b, c)| component[a] && component[b] && component[c]);
            has_triangle
                && edge_face_counts
                    .iter()
                    .filter(|(&(a, b), _)| component[a] && component[b])
                    .all(|(_, &count)| count == 2)
        })
        .count();
    let b1 = if chi <= (b0 + b2) as isize {
        (b0 as isize - chi + b2 as isize) as usize
    } else {
        0
    };
    (b0, b1, b2)
}

pub fn euler_characteristic(sc: &SimplicialComplex) -> isize {
    sc.vertices as isize - sc.edges.len() as isize + sc.triangles.len() as isize
}

fn connected_components(sc: &SimplicialComplex) -> Vec<Vec<bool>> {
    let mut adjacency = vec![Vec::new(); sc.vertices];
    for &(a, b) in &sc.edges {
        if a < sc.vertices && b < sc.vertices {
            adjacency[a].push(b);
            adjacency[b].push(a);
        }
    }

    let mut seen = vec![false; sc.vertices];
    let mut components = Vec::new();
    for start in 0..sc.vertices {
        if seen[start] {
            continue;
        }
        let mut component = vec![false; sc.vertices];
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(v) = stack.pop() {
            component[v] = true;
            for &next in &adjacency[v] {
                if !seen[next] {
                    seen[next] = true;
                    stack.push(next);
                }
            }
        }
        components.push(component);
    }
    components
}

fn triangle_edge_face_counts(
    sc: &SimplicialComplex,
) -> std::collections::HashMap<(usize, usize), usize> {
    let mut counts = std::collections::HashMap::new();
    for &(a, b, c) in &sc.triangles {
        for (u, v) in [(a, b), (b, c), (c, a)] {
            *counts.entry((u.min(v), u.max(v))).or_insert(0) += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let sc = extract_simplicial_complex(&m);
        assert_eq!(sc.vertices, 4);
        assert_eq!(sc.edges.len(), 5);
        assert_eq!(sc.triangles.len(), 2);
    }
    #[test]
    fn test_euler() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let sc = extract_simplicial_complex(&m);
        assert_eq!(euler_characteristic(&sc), 1);
    }
    #[test]
    fn test_betti() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let sc = extract_simplicial_complex(&m);
        let (b0, _, _) = betti_numbers(&sc);
        assert_eq!(b0, 1);
    }

    #[test]
    fn test_closed_component_betti_two() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let sc = extract_simplicial_complex(&m);
        assert_eq!(betti_numbers(&sc), (1, 0, 1));
    }

    #[test]
    fn test_invalid_cells_are_skipped() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.polys.push_cell(&[0, -1, 1]);
        m.polys.push_cell(&[0, 1, 42]);
        let sc = extract_simplicial_complex(&m);
        assert!(sc.edges.is_empty());
        assert!(sc.triangles.is_empty());
    }
}
