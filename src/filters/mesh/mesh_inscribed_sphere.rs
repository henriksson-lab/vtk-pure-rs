//! Compute inscribed circle radius (inradius) per triangle face.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn inscribed_radius(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut radii = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            radii.push(0.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            radii.push(0.0);
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
        let s = (ab + bc + ca) / 2.0;
        let area = (s * (s - ab).max(0.0) * (s - bc).max(0.0) * (s - ca).max(0.0)).sqrt();
        radii.push(if s > 1e-15 { area / s } else { 0.0 });
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Inradius", radii, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inradius() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = inscribed_radius(&mesh);
        let arr = r.cell_data().get_array("Inradius").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        // Equilateral triangle inradius = side / (2*sqrt(3))
        let expected = 1.0 / (2.0 * 3.0f64.sqrt());
        assert!((b[0] - expected).abs() < 0.01);
    }
}
