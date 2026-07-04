//! Color mesh faces by their normal direction.
use crate::data::{AnyDataArray, DataArray, PolyData};
/// Assign RGB color to each face based on its normal direction.
pub fn color_faces_by_normal(mesh: &PolyData) -> PolyData {
    let mut colors = Vec::new();
    for cell in mesh.polys.iter() {
        let Some(n) = polygon_normal(mesh, cell) else {
            colors.extend_from_slice(&[128.0, 128.0, 128.0]);
            continue;
        };
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

fn polygon_normal(mesh: &PolyData, cell: &[i64]) -> Option<[f64; 3]> {
    if cell.len() < 3 {
        return None;
    }
    let p0 = point(mesh, cell[0])?;
    let mut n = [0.0; 3];
    for i in 1..cell.len() - 1 {
        let p1 = point(mesh, cell[i])?;
        let p2 = point(mesh, cell[i + 1])?;
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        n[0] += e1[1] * e2[2] - e1[2] * e2[1];
        n[1] += e1[2] * e2[0] - e1[0] * e2[2];
        n[2] += e1[0] * e2[1] - e1[1] * e2[0];
    }
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l > 1e-15 {
        Some([n[0] / l, n[1] / l, n[2] / l])
    } else {
        None
    }
}

fn point(mesh: &PolyData, id: i64) -> Option<[f64; 3]> {
    usize::try_from(id)
        .ok()
        .filter(|&idx| idx < mesh.points.len())
        .map(|idx| mesh.points.get(idx))
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
