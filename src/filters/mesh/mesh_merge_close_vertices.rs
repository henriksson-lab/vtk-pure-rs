//! Merge vertices that are closer than a threshold distance.
use crate::data::{CellArray, Points, PolyData};

pub fn merge_close_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let tol2 = tolerance * tolerance;
    let mut map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let mut merged = false;
        for j in 0..pts.len() {
            let q = pts.get(j);
            if (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) < tol2 {
                map[i] = j;
                merged = true;
                break;
            }
        }
        if !merged {
            map[i] = pts.len();
            pts.push(p.try_into().unwrap());
        }
    }
    let remap_cells = |cells: &CellArray| -> CellArray {
        let mut out = CellArray::new();
        for cell in cells.iter() {
            let mapped: Vec<i64> = cell.iter().map(|&v| map[v as usize] as i64).collect();
            let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
            if unique.len() >= cell.len().min(3) {
                out.push_cell(&mapped);
            }
        }
        out
    };
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = remap_cells(&mesh.polys);
    result.lines = remap_cells(&mesh.lines);
    result.verts = remap_cells(&mesh.verts);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_merge() {
        let mut mesh = PolyData::new();
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([0.001, 0.0, 0.0]); // very close
        pts.push([1.0, 0.0, 0.0]);
        pts.push([0.5, 1.0, 0.0]);
        mesh.points = pts;
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 2, 3]);
        polys.push_cell(&[1, 2, 3]);
        mesh.polys = polys;
        let r = merge_close_vertices(&mesh, 0.01);
        assert_eq!(r.points.len(), 3); // 0 and 1 merged
    }
}
