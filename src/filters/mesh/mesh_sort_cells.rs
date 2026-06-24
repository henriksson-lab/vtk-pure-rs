//! Sort mesh cells by various criteria.
use crate::data::{CellArray, PolyData};
pub fn sort_by_area_ascending(mesh: &PolyData) -> PolyData {
    sort_cells(mesh, false)
}
pub fn sort_by_area_descending(mesh: &PolyData) -> PolyData {
    sort_cells(mesh, true)
}
fn sort_cells(mesh: &PolyData, descending: bool) -> PolyData {
    let mut cells: Vec<(Vec<i64>, f64)> = mesh
        .polys
        .iter()
        .map(|cell| {
            let c = cell.to_vec();
            let area = if c.len() < 3 {
                0.0
            } else {
                let a = mesh.points.get(c[0] as usize);
                let b = mesh.points.get(c[1] as usize);
                let cc = mesh.points.get(c[2] as usize);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
                0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                    + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                    + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                .sqrt()
            };
            (c, area)
        })
        .collect();
    if descending {
        cells.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        cells.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    let mut polys = CellArray::new();
    for (c, _) in &cells {
        polys.push_cell(c);
    }
    let mut r = mesh.clone();
    r.polys = polys;
    r
}
pub fn sort_by_centroid_z(mesh: &PolyData) -> PolyData {
    let mut cells: Vec<(Vec<i64>, f64)> = mesh
        .polys
        .iter()
        .map(|cell| {
            let c = cell.to_vec();
            if c.is_empty() {
                return (c, 0.0);
            }
            let z: f64 = c
                .iter()
                .map(|&v| mesh.points.get(v as usize)[2])
                .sum::<f64>()
                / c.len() as f64;
            (c, z)
        })
        .collect();
    cells.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut polys = CellArray::new();
    for (c, _) in &cells {
        polys.push_cell(c);
    }
    let mut r = mesh.clone();
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_area() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, 10.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = sort_by_area_ascending(&m);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_z() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 10.0],
                [1.0, 0.0, 10.0],
                [0.5, 1.0, 10.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = sort_by_centroid_z(&m);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
