//! Color mesh faces by their normal direction.
use crate::data::{AnyDataArray, DataArray, PolyData};
/// Assign RGB color to each face based on its normal direction.
pub fn color_faces_by_normal(mesh: &PolyData) -> PolyData {
    let mut colors = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            colors.extend_from_slice(&[128.0, 128.0, 128.0]);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let mut n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 1e-15 {
            n[0] /= l;
            n[1] /= l;
            n[2] /= l;
        }
        colors.push((n[0] * 0.5 + 0.5) * 255.0);
        colors.push((n[1] * 0.5 + 0.5) * 255.0);
        colors.push((n[2] * 0.5 + 0.5) * 255.0);
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalColor",
            colors,
            3,
        )));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_color() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = color_faces_by_normal(&mesh);
        assert!(r.cell_data().get_array("NormalColor").is_some());
    }
}
