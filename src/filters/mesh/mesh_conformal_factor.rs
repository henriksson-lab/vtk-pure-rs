//! Compute conformal factor (ratio of actual to ideal triangle area) per face.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn conformal_factor(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut factors = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            factors.push(1.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            factors.push(1.0);
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let ab =
            ((pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2) + (pb[2] - pa[2]).powi(2)).sqrt();
        let bc =
            ((pc[0] - pb[0]).powi(2) + (pc[1] - pb[1]).powi(2) + (pc[2] - pb[2]).powi(2)).sqrt();
        let ca =
            ((pa[0] - pc[0]).powi(2) + (pa[1] - pc[1]).powi(2) + (pa[2] - pc[2]).powi(2)).sqrt();
        // Actual area
        let s = (ab + bc + ca) / 2.0;
        let area = (s * (s - ab).max(0.0) * (s - bc).max(0.0) * (s - ca).max(0.0)).sqrt();
        // Ideal equilateral area with same perimeter
        let avg_edge = (ab + bc + ca) / 3.0;
        let ideal = 3.0f64.sqrt() / 4.0 * avg_edge * avg_edge;
        factors.push(if ideal > 1e-15 { area / ideal } else { 0.0 });
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConformalFactor",
            factors,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_conformal() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = conformal_factor(&mesh);
        let arr = r.cell_data().get_array("ConformalFactor").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 0.01);
    }
}
