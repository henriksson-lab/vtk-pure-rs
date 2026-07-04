//! Group coplanar adjacent faces.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn label_coplanar_groups(mesh: &PolyData, angle_tolerance: f64) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let cos_t = angle_tolerance.to_radians().cos();
    let normals: Vec<[f64; 3]> = cells.iter().map(|cell| fnorm(cell, mesh)).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % n], mesh.points.len()) else {
                continue;
            };
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut labels = vec![0usize; nc];
    let mut next_label = 1;
    let mut visited = vec![false; nc];
    for start in 0..nc {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        labels[start] = next_label;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(ci) = queue.pop_front() {
            let cell = &cells[ci];
            let n_c = cell.len();
            for i in 0..n_c {
                let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                    continue;
                };
                let Some(b) = valid_point_id(cell[(i + 1) % n_c], mesh.points.len()) else {
                    continue;
                };
                if let Some(nbs) = ef.get(&(a.min(b), a.max(b))) {
                    for &ni in nbs {
                        if visited[ni] {
                            continue;
                        }
                        let dot = normals[ci][0] * normals[ni][0]
                            + normals[ci][1] * normals[ni][1]
                            + normals[ci][2] * normals[ni][2];
                        if dot.abs() >= cos_t {
                            visited[ni] = true;
                            labels[ni] = next_label;
                            queue.push_back(ni);
                        }
                    }
                }
            }
        }
        next_label += 1;
    }
    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CoplanarGroup",
            data,
            1,
        )));
    r
}
fn fnorm(cell: &[i64], mesh: &PolyData) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let Some(ai) = valid_point_id(cell[0], mesh.points.len()) else {
        return [0.0, 0.0, 1.0];
    };
    let Some(bi) = valid_point_id(cell[1], mesh.points.len()) else {
        return [0.0, 0.0, 1.0];
    };
    let Some(ci) = valid_point_id(cell[2], mesh.points.len()) else {
        return [0.0, 0.0, 1.0];
    };
    let a = mesh.points.get(ai);
    let b = mesh.points.get(bi);
    let c = mesh.points.get(ci);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / l, n[1] / l, n[2] / l]
    }
}
fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
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
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = label_coplanar_groups(&m, 5.0);
        let arr = r.cell_data().get_array("CoplanarGroup").unwrap();
        let mut b1 = [0.0];
        let mut b2 = [0.0];
        arr.tuple_as_f64(0, &mut b1);
        arr.tuple_as_f64(1, &mut b2);
        assert_eq!(b1[0], b2[0]);
    } // coplanar -> same group
}
