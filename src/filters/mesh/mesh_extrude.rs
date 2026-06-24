//! Mesh extrusion along normals or directions.

use crate::data::{CellArray, Points, PolyData};

/// Extrude mesh along a direction vector, creating a solid shell.
pub fn extrude_direction(mesh: &PolyData, direction: [f64; 3], distance: f64) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Original points
    for i in 0..n {
        pts.push(mesh.points.get(i));
    }
    // Offset points
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + direction[0] * distance,
            p[1] + direction[1] * distance,
            p[2] + direction[2] * distance,
        ]);
    }

    // Original faces (bottom)
    for cell in mesh.polys.iter() {
        polys.push_cell(cell);
    }
    // Top faces (reversed winding, offset indices)
    for cell in mesh.polys.iter() {
        let mut top: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        top.reverse();
        polys.push_cell(&top);
    }
    // Side faces from boundary edges
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            polys.push_cell(&[a as i64, b as i64, (b + n) as i64, (a + n) as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Extrude edges of a mesh radially from centroid.
pub fn extrude_radial(mesh: &PolyData, distance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..n {
        pts.push(mesh.points.get(i));
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-15);
        pts.push([
            p[0] + dx / len * distance,
            p[1] + dy / len * distance,
            p[2] + dz / len * distance,
        ]);
    }

    for cell in mesh.polys.iter() {
        polys.push_cell(cell);
    }
    for cell in mesh.polys.iter() {
        let mut top: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        top.reverse();
        polys.push_cell(&top);
    }
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            polys.push_cell(&[a as i64, b as i64, (b + n) as i64, (a + n) as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extrude_dir() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extrude_direction(&mesh, [0.0, 0.0, 1.0], 2.0);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() > 2); // top + bottom + sides
    }
    #[test]
    fn test_extrude_radial() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extrude_radial(&mesh, 0.5);
        assert_eq!(r.points.len(), 6);
    }
}
