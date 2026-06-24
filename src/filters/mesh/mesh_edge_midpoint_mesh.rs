//! Create a new mesh from edge midpoints.
use crate::data::{CellArray, Points, PolyData};
pub fn edge_midpoint_mesh(mesh: &PolyData) -> PolyData {
    let mut em: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        let mids: Vec<usize> = (0..nc)
            .map(|i| {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                let key = (a.min(b), a.max(b));
                *em.entry(key).or_insert_with(|| {
                    let pa = mesh.points.get(a);
                    let pb = mesh.points.get(b);
                    let idx = pts.len();
                    pts.push([
                        (pa[0] + pb[0]) / 2.0,
                        (pa[1] + pb[1]) / 2.0,
                        (pa[2] + pb[2]) / 2.0,
                    ]);
                    idx
                })
            })
            .collect();
        polys.push_cell(&mids.iter().map(|&m| m as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = edge_midpoint_mesh(&m);
        assert_eq!(r.points.len(), 3);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
