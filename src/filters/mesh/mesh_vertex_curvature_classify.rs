//! Classify vertices by curvature type (elliptic, hyperbolic, parabolic, flat).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn classify_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut angle_sum = vec![0.0f64; n];
    let mut area_sum = vec![0.0f64; n];
    let cells = surface_cells(mesh);
    for cell in &cells {
        if cell.len() < 3 {
            continue;
        }
        let Some(first) = valid_point_index(cell[0], n) else {
            continue;
        };
        let nc = cell.len();
        for i in 1..nc - 1 {
            let Some(second) = valid_point_index(cell[i], n) else {
                continue;
            };
            let Some(third) = valid_point_index(cell[i + 1], n) else {
                continue;
            };
            let a = mesh.points.get(first);
            let b = mesh.points.get(second);
            let c = mesh.points.get(third);
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            area_sum[first] += area;
            area_sum[second] += area;
            area_sum[third] += area;
        }
        for i in 0..nc {
            let Some(vi) = valid_point_index(cell[i], n) else {
                continue;
            };
            let Some(prev) = valid_point_index(cell[(i + nc - 1) % nc], n) else {
                continue;
            };
            let Some(next) = valid_point_index(cell[(i + 1) % nc], n) else {
                continue;
            };
            let p = mesh.points.get(vi);
            let a = mesh.points.get(prev);
            let b = mesh.points.get(next);
            let va = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
            let vb = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
            let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
            let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
            if la > 1e-15 && lb > 1e-15 {
                let cos =
                    ((va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]) / (la * lb)).clamp(-1.0, 1.0);
                angle_sum[vi] += cos.acos();
            }
        }
    }
    let gauss: Vec<f64> = angle_sum
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let defect = 2.0 * std::f64::consts::PI - s;
            if area_sum[i] > 0.0 {
                3.0 * defect / area_sum[i]
            } else {
                0.0
            }
        })
        .collect();
    // Classify: >eps=elliptic(1), <-eps=hyperbolic(-1), ~0=parabolic/flat(0)
    let eps = 0.01;
    let data: Vec<f64> = gauss
        .iter()
        .map(|&g| {
            if g > eps {
                1.0
            } else if g < -eps {
                -1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureType",
            data,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurv",
            gauss,
            1,
        )));
    r
}

fn surface_cells(mesh: &PolyData) -> Vec<Vec<i64>> {
    let mut cells: Vec<Vec<i64>> = mesh.polys.iter().map(|cell| cell.to_vec()).collect();
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                cells.push(vec![tri[0], tri[1], tri[2]]);
            } else {
                cells.push(vec![tri[1], tri[0], tri[2]]);
            }
        }
    }
    cells
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = classify_curvature(&m);
        assert!(r.point_data().get_array("CurvatureType").is_some());
        assert!(r.point_data().get_array("GaussianCurv").is_some());
    }
}
