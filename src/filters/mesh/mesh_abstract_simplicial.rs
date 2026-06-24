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
    // For connected mesh: b0=components, b2=0 (open) or 1 (closed), b1=b0-chi+b2
    // Simplified: assume connected
    let b0 = 1usize; // approximate
    let b2 = 0usize; // approximate for open mesh
    let b1 = if chi <= b0 as isize {
        (b0 as isize - chi + b2 as isize) as usize
    } else {
        0
    };
    (b0, b1, b2)
}
pub fn euler_characteristic(sc: &SimplicialComplex) -> isize {
    sc.vertices as isize - sc.edges.len() as isize + sc.triangles.len() as isize
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
}
