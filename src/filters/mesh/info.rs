use crate::data::PolyData;
use std::collections::{HashMap, HashSet};

/// Compute the Euler characteristic of a mesh: V - E + F.
///
/// For a closed orientable surface, this equals 2 - 2*genus.
/// A sphere has Euler characteristic 2, a torus has 0.
pub fn euler_characteristic(input: &PolyData) -> i64 {
    let v = input.points.len() as i64;
    let f = input.polys.num_cells() as i64;

    // Count unique edges
    let mut edges: HashSet<(i64, i64)> = HashSet::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
        }
    }
    let e = edges.len() as i64;

    v - e + f
}

/// Estimate the genus of a closed surface from its Euler characteristic.
/// genus = (2 - chi) / 2 where chi = V - E + F.
pub fn genus(input: &PolyData) -> i64 {
    let chi = euler_characteristic(input);
    (2 - chi) / 2
}

/// Count the number of boundary loops (holes) in a mesh.
pub fn count_boundary_loops(input: &PolyData) -> usize {
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Boundary edges
    let mut boundary: HashMap<i64, Vec<i64>> = HashMap::new();
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary.entry(a).or_default().push(b);
            boundary.entry(b).or_default().push(a);
        }
    }

    if boundary.is_empty() {
        return 0;
    }

    // Count connected loops
    let mut visited: HashSet<i64> = HashSet::new();
    let mut loops = 0;

    for &start in boundary.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut cur = start;
        loop {
            visited.insert(cur);
            let nexts = boundary.get(&cur);
            let next = nexts.and_then(|v| v.iter().find(|&&n| !visited.contains(&n)));
            match next {
                Some(&n) => cur = n,
                None => break,
            }
        }
        loops += 1;
    }
    loops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(euler_characteristic(&pd), 1); // V=3, E=3, F=1
    }

    #[test]
    fn tetrahedron_surface() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(euler_characteristic(&pd), 2); // sphere topology
        assert_eq!(genus(&pd), 0);
    }

    #[test]
    fn open_mesh_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(count_boundary_loops(&pd), 1);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert_eq!(euler_characteristic(&pd), 0);
        assert_eq!(count_boundary_loops(&pd), 0);
    }
}
