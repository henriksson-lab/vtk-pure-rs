//! Remove isolated vertices (vertices not referenced by any cell).
use crate::data::{CellArray, Points, PolyData};

pub fn remove_isolated_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut used = vec![false; n];
    for cell in mesh.polys.iter() {
        for &v in &cell[..] {
            let v = v as usize;
            if v < n {
                used[v] = true;
            }
        }
    }
    for cell in mesh.lines.iter() {
        for &v in &cell[..] {
            let v = v as usize;
            if v < n {
                used[v] = true;
            }
        }
    }
    for cell in mesh.verts.iter() {
        for &v in &cell[..] {
            let v = v as usize;
            if v < n {
                used[v] = true;
            }
        }
    }
    let mut pt_map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i).try_into().unwrap());
        }
    }
    let remap = |cells: &CellArray| -> CellArray {
        let mut out = CellArray::new();
        for cell in cells.iter() {
            let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
            out.push_cell(&mapped);
        }
        out
    };
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = remap(&mesh.polys);
    result.lines = remap(&mesh.lines);
    result.verts = remap(&mesh.verts);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_isolated() {
        let mut mesh = PolyData::new();
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([1.0, 0.0, 0.0]);
        pts.push([0.5, 1.0, 0.0]);
        pts.push([99.0, 99.0, 99.0]); // isolated
        mesh.points = pts;
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        mesh.polys = polys;
        let r = remove_isolated_vertices(&mesh);
        assert_eq!(r.points.len(), 3);
    }
}
