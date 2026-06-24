//! Compute topological genus of a mesh using Euler characteristic.
use crate::data::PolyData;

pub fn genus(mesh: &PolyData) -> i64 {
    let v = mesh.points.len() as i64;
    let f = mesh.polys.num_cells() as i64;
    // Count edges
    let mut edges = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            let e = if a < b { (a, b) } else { (b, a) };
            edges.insert(e);
        }
    }
    let e = edges.len() as i64;
    // Euler characteristic: V - E + F = 2 - 2g (for closed surface)
    // g = (2 - V + E - F) / 2
    let chi = v - e + f;
    (2 - chi) / 2
}

pub fn euler_characteristic(mesh: &PolyData) -> i64 {
    let v = mesh.points.len() as i64;
    let f = mesh.polys.num_cells() as i64;
    let mut edges = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            let e = if a < b { (a, b) } else { (b, a) };
            edges.insert(e);
        }
    }
    v - edges.len() as i64 + f
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_genus() {
        // Simple triangle: V=3, E=3, F=1 => chi=1 => genus=0 (or 1 for boundary)
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let chi = euler_characteristic(&mesh);
        assert_eq!(chi, 1); // open surface
    }
}
