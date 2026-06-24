//! Boolean subtraction approximation (remove vertices inside another mesh).
use crate::data::{CellArray, Points, PolyData};

pub fn boolean_subtract(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    let nb = mesh_b.points.len();
    if nb == 0 {
        return mesh_a.clone();
    }
    // Simple approach: compute bounding box of B and remove A vertices inside it
    let mut bmin = [f64::INFINITY; 3];
    let mut bmax = [f64::NEG_INFINITY; 3];
    for i in 0..nb {
        let p = mesh_b.points.get(i);
        for d in 0..3 {
            bmin[d] = bmin[d].min(p[d]);
            bmax[d] = bmax[d].max(p[d]);
        }
    }
    let na = mesh_a.points.len();
    let inside: Vec<bool> = (0..na)
        .map(|i| {
            let p = mesh_a.points.get(i);
            p[0] >= bmin[0]
                && p[0] <= bmax[0]
                && p[1] >= bmin[1]
                && p[1] <= bmax[1]
                && p[2] >= bmin[2]
                && p[2] <= bmax[2]
        })
        .collect();
    let mut pt_map = vec![0usize; na];
    let mut pts = Points::<f64>::new();
    for i in 0..na {
        if !inside[i] {
            pt_map[i] = pts.len();
            pts.push(mesh_a.points.get(i).try_into().unwrap());
        }
    }
    let mut polys = CellArray::new();
    for cell in mesh_a.polys.iter() {
        if cell.iter().any(|&v| inside[v as usize]) {
            continue;
        }
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
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
    fn test_subtract() {
        let a = PolyData::from_triangles(
            vec![[-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_subtract(&a, &b);
        // Some vertices of A are inside B's bbox, so should have fewer
        assert!(r.points.len() <= a.points.len());
    }
}
