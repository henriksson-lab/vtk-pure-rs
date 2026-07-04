//! Compute triangle aspect ratio quality metric.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn aspect_ratio(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut ratios = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            ratios.push(0.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            ratios.push(0.0);
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
        let e1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let e2 = [pc[0] - pb[0], pc[1] - pb[1], pc[2] - pb[2]];
        let denominator = ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
            + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
            + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
        .sqrt();
        let longest = ab.max(bc).max(ca);
        let ratio = if denominator >= 2.2204460492503131e-15 {
            (3.0f64.sqrt() / 6.0) * longest * (ab + bc + ca) / denominator
        } else {
            1.0e30
        }
        .clamp(-1.0e30, 1.0e30);
        ratios.push(ratio);
    }
    let mut result = mesh.clone();
    let cell_data = result.cell_data_mut();
    cell_data.add_array(AnyDataArray::F64(DataArray::from_vec(
        "AspectRatio",
        ratios,
        1,
    )));
    cell_data.set_active_scalars("AspectRatio");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_aspect() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = aspect_ratio(&mesh);
        let arr = r.cell_data().get_array("AspectRatio").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 0.01); // equilateral = perfect ratio
    }
}
