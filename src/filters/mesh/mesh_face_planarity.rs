//! Measure planarity of mesh faces (deviation from best-fit plane).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_planarity(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() <= 3 {
                return 0.0;
            } // triangles are always planar
            if cell
                .iter()
                .any(|&v| v < 0 || v as usize >= mesh.points.len())
            {
                return 0.0;
            }
            let pts: Vec<[f64; 3]> = cell.iter().map(|&v| mesh.points.get(v as usize)).collect();
            let centroid = centroid(&pts);
            let Some(nn) = best_fit_normal(&pts, centroid) else {
                return 0.0;
            };
            // Max distance from plane
            let mut max_d = 0.0f64;
            for p in &pts {
                let d = ((p[0] - centroid[0]) * nn[0]
                    + (p[1] - centroid[1]) * nn[1]
                    + (p[2] - centroid[2]) * nn[2])
                    .abs();
                max_d = max_d.max(d);
            }
            max_d
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Planarity", data, 1)));
    r
}
pub fn non_planar_face_count(mesh: &PolyData, tolerance: f64) -> usize {
    let r = face_planarity(mesh);
    let arr = r.cell_data().get_array("Planarity").unwrap();
    let mut buf = [0.0f64];
    let mut count = 0;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] > tolerance {
            count += 1;
        }
    }
    count
}

fn centroid(pts: &[[f64; 3]]) -> [f64; 3] {
    let mut c = [0.0; 3];
    for p in pts {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = pts.len() as f64;
    [c[0] / n, c[1] / n, c[2] / n]
}

fn best_fit_normal(pts: &[[f64; 3]], c: [f64; 3]) -> Option<[f64; 3]> {
    let mut a = [[0.0; 3]; 3];
    for p in pts {
        let x = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for i in 0..3 {
            for j in i..3 {
                a[i][j] += x[i] * x[j];
            }
        }
    }
    a[1][0] = a[0][1];
    a[2][0] = a[0][2];
    a[2][1] = a[1][2];

    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..16 {
        let mut p = 0;
        let mut q = 1;
        let mut max = a[0][1].abs();
        for (i, j) in [(0, 2), (1, 2)] {
            if a[i][j].abs() > max {
                max = a[i][j].abs();
                p = i;
                q = j;
            }
        }
        if max < 1e-15 {
            break;
        }

        let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let cs = 1.0 / (1.0 + t * t).sqrt();
        let sn = t * cs;

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        a[p][p] = app - t * apq;
        a[q][q] = aqq + t * apq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                a[r][p] = cs * arp - sn * arq;
                a[p][r] = a[r][p];
                a[r][q] = sn * arp + cs * arq;
                a[q][r] = a[r][q];
            }
        }
        for row in &mut v {
            let vrp = row[p];
            let vrq = row[q];
            row[p] = cs * vrp - sn * vrq;
            row[q] = sn * vrp + cs * vrq;
        }
    }

    let mut min_i = 0;
    if a[1][1] < a[min_i][min_i] {
        min_i = 1;
    }
    if a[2][2] < a[min_i][min_i] {
        min_i = 2;
    }
    let normal = [v[0][min_i], v[1][min_i], v[2][min_i]];
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len > 1e-15 {
        Some([normal[0] / len, normal[1] / len, normal[2] / len])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tri() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_planarity(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Planarity")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1e-10);
    }
    #[test]
    fn test_quad() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([1.0, 1.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.polys.push_cell(&[0, 1, 2, 3]);
        assert_eq!(non_planar_face_count(&m, 1e-10), 0);
    }
}
