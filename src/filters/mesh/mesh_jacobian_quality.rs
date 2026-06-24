//! Compute scaled Jacobian quality metric per triangle.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn jacobian_quality(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut qualities = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            qualities.push(0.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            qualities.push(0.0);
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let e0 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let e1 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let cross = [
            e0[1] * e1[2] - e0[2] * e1[1],
            e0[2] * e1[0] - e0[0] * e1[2],
            e0[0] * e1[1] - e0[1] * e1[0],
        ];
        let area2 = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let l0 = (e0[0] * e0[0] + e0[1] * e0[1] + e0[2] * e0[2]).sqrt();
        let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
        let e2 = [pc[0] - pb[0], pc[1] - pb[1], pc[2] - pb[2]];
        let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
        let max_l = l0.max(l1).max(l2).max(1e-15);
        // Scaled Jacobian: 2*area / (max_edge^2 * sqrt(3))
        let q = area2 / (max_l * max_l * 3.0f64.sqrt());
        qualities.push(q.clamp(0.0, 1.0));
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "JacobianQuality",
            qualities,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_jacobian() {
        // Equilateral triangle should have quality ~1.0
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = jacobian_quality(&mesh);
        let arr = r.cell_data().get_array("JacobianQuality").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 0.05);
    }
}
