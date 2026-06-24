//! Compute ratio of shortest to longest edge per face.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn edge_ratio(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut ratios = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            ratios.push(0.0);
            continue;
        }
        let nc = cell.len();
        let mut min_e = f64::INFINITY;
        let mut max_e = 0.0f64;
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= n || b >= n {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            if d < min_e {
                min_e = d;
            }
            if d > max_e {
                max_e = d;
            }
        }
        ratios.push(if max_e > 1e-15 { min_e / max_e } else { 0.0 });
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "EdgeRatio",
            ratios,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_edge_ratio() {
        // Equilateral: ratio = 1.0
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = edge_ratio(&mesh);
        let arr = r.cell_data().get_array("EdgeRatio").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 0.01);
    }
}
