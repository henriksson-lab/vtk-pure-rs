//! Thicken a surface mesh into a solid shell.
use crate::data::{CellArray, Points, PolyData};
pub fn thicken(mesh: &PolyData, thickness: f64) -> PolyData {
    let n = mesh.points.len();
    let normals = compute_normals(mesh);
    let half = thickness / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        pts.push([
            p[0] - nm[0] * half,
            p[1] - nm[1] * half,
            p[2] - nm[2] * half,
        ]);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        pts.push([
            p[0] + nm[0] * half,
            p[1] + nm[1] * half,
            p[2] + nm[2] * half,
        ]);
    }
    for cell in mesh.polys.iter() {
        polys.push_cell(cell);
        let mut top: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        top.reverse();
        polys.push_cell(&top);
    }
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    for (&(a, b), &c) in &ec {
        if c == 1 {
            polys.push_cell(&[a as i64, b as i64, (b + n) as i64, (a + n) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
fn compute_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_thicken() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = thicken(&m, 0.5);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() > 2);
    }
}
