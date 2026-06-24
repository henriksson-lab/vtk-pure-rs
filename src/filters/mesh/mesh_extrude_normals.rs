//! Extrude mesh surface along vertex normals to create a thickened shell.
use crate::data::{CellArray, Points, PolyData};

pub fn extrude_along_normals(mesh: &PolyData, distance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut vnorm = vec![[0.0f64; 3]; n];
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
                vnorm[vi][0] += nx;
                vnorm[vi][1] += ny;
                vnorm[vi][2] += nz;
            }
        }
    }
    for vn in &mut vnorm {
        let l = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if l > 1e-15 {
            vn[0] /= l;
            vn[1] /= l;
            vn[2] /= l;
        }
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0], p[1], p[2]]);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + vnorm[i][0] * distance,
            p[1] + vnorm[i][1] * distance,
            p[2] + vnorm[i][2] * distance,
        ]);
    }
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        polys.push_cell(&cell.to_vec());
    }
    for cell in mesh.polys.iter() {
        let rev: Vec<i64> = cell.iter().rev().map(|&v| v + n as i64).collect();
        polys.push_cell(&rev);
    }
    // Side faces for boundary edges
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
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
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
    fn test_extrude() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extrude_along_normals(&mesh, 0.5);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() >= 2); // top + bottom + sides
    }
}
