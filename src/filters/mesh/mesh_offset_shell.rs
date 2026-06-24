//! Offset mesh to create a shell (thicken surface into solid).
use crate::data::{CellArray, Points, PolyData};

pub fn offset_shell(mesh: &PolyData, thickness: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Compute vertex normals (average of face normals)
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        for &vi in &cell[..] {
            let vi = vi as usize;
            if vi < n {
                vnormals[vi][0] += nx;
                vnormals[vi][1] += ny;
                vnormals[vi][2] += nz;
            }
        }
    }
    for vn in &mut vnormals {
        let len = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if len > 1e-15 {
            vn[0] /= len;
            vn[1] /= len;
            vn[2] /= len;
        }
    }
    let mut pts = Points::<f64>::new();
    // Original points
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push(p.try_into().unwrap());
    }
    // Offset points
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + vnormals[i][0] * thickness,
            p[1] + vnormals[i][1] * thickness,
            p[2] + vnormals[i][2] * thickness,
        ]);
    }
    let mut polys = CellArray::new();
    // Original faces
    for cell in mesh.polys.iter() {
        polys.push_cell(&cell.iter().map(|&v| v).collect::<Vec<_>>());
    }
    // Offset faces (reversed winding)
    for cell in mesh.polys.iter() {
        let rev: Vec<i64> = cell.iter().rev().map(|&v| v + n as i64).collect();
        polys.push_cell(&rev);
    }
    // Side faces (connect boundary edges)
    let mut edge_count: std::collections::HashMap<(i64, i64), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            polys.push_cell(&[a, b, b + n as i64, a + n as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shell() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = offset_shell(&mesh, 0.1);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() >= 2);
    }
}
