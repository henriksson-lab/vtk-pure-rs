//! Compute circumscribed circle radius (circumradius) per triangle.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn circumradius(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut radii = Vec::new();
    let mut ratios = Vec::new();
    append_zero_cell_metrics(mesh.verts.num_cells(), &mut radii, &mut ratios);
    append_zero_cell_metrics(mesh.lines.num_cells(), &mut radii, &mut ratios);
    for cell in mesh.polys.iter() {
        if cell.len() != 3 || !cell_has_valid_points(cell, n) {
            radii.push(0.0);
            ratios.push(0.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let ab =
            ((pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2) + (pb[2] - pa[2]).powi(2)).sqrt();
        let bc =
            ((pc[0] - pb[0]).powi(2) + (pc[1] - pb[1]).powi(2) + (pc[2] - pb[2]).powi(2)).sqrt();
        let ca =
            ((pa[0] - pc[0]).powi(2) + (pa[1] - pc[1]).powi(2) + (pa[2] - pc[2]).powi(2)).sqrt();
        let s = (ab + bc + ca) * 0.5;
        let e1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let e2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let r = if area > 1e-15 {
            ab * bc * ca / (4.0 * area)
        } else {
            0.0
        };
        let inradius = if s > 1e-15 { area / s } else { 0.0 };
        let ratio = if inradius > 1e-15 { r / inradius } else { 0.0 };
        radii.push(r);
        ratios.push(ratio);
    }
    append_zero_cell_metrics(mesh.strips.num_cells(), &mut radii, &mut ratios);
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Circumradius",
            radii,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CircumInRatio",
            ratios,
            1,
        )));
    result
}

fn append_zero_cell_metrics(count: usize, radii: &mut Vec<f64>, ratios: &mut Vec<f64>) {
    radii.extend(std::iter::repeat(0.0).take(count));
    ratios.extend(std::iter::repeat(0.0).take(count));
}

fn cell_has_valid_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&pid| pid >= 0 && (pid as usize) < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_circumradius() {
        // Equilateral triangle: circumradius = side / sqrt(3)
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = circumradius(&mesh);
        let arr = r.cell_data().get_array("Circumradius").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        let expected = 1.0 / 3.0f64.sqrt();
        assert!((b[0] - expected).abs() < 0.01);
        let arr = r.cell_data().get_array("CircumInRatio").unwrap();
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 2.0).abs() < 0.01);
    }
}
