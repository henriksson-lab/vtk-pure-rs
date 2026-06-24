//! Find critical points of a scalar field on mesh (minima, maxima, saddles).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub struct CriticalPoint {
    pub vertex: usize,
    pub value: f64,
    pub kind: CriticalKind,
}
pub enum CriticalKind {
    Minimum,
    Maximum,
    Saddle,
    Regular,
}
pub fn find_critical_points(mesh: &PolyData, array_name: &str) -> Vec<CriticalPoint> {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return vec![],
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut result = Vec::new();
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let vi = vals[i];
        let lower = nb[i].iter().filter(|&&j| vals[j] < vi).count();
        let upper = nb[i].iter().filter(|&&j| vals[j] > vi).count();
        let kind = if lower == 0 && upper > 0 {
            CriticalKind::Minimum
        } else if upper == 0 && lower > 0 {
            CriticalKind::Maximum
        } else if lower > 0 && upper > 0 {
            // Check for saddle: count sign changes around ring
            let sorted_nb = &nb[i];
            let changes = sorted_nb
                .windows(2)
                .filter(|w| (vals[w[0]] > vi) != (vals[w[1]] > vi))
                .count();
            if changes >= 4 {
                CriticalKind::Saddle
            } else {
                CriticalKind::Regular
            }
        } else {
            CriticalKind::Regular
        };
        match kind {
            CriticalKind::Regular => {}
            _ => result.push(CriticalPoint {
                vertex: i,
                value: vi,
                kind,
            }),
        }
    }
    result
}
pub fn attach_critical_type(mesh: &PolyData, array_name: &str) -> PolyData {
    let crits = find_critical_points(mesh, array_name);
    let n = mesh.points.len();
    let mut data = vec![0.0f64; n]; // 0=regular
    for c in &crits {
        match c.kind {
            CriticalKind::Minimum => data[c.vertex] = -1.0,
            CriticalKind::Maximum => data[c.vertex] = 1.0,
            CriticalKind::Saddle => data[c.vertex] = 0.5,
            _ => {}
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalType",
            data,
            1,
        )));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let crits = find_critical_points(&m, "h");
        assert!(crits
            .iter()
            .any(|c| matches!(c.kind, CriticalKind::Maximum)));
    }
    #[test]
    fn test_attach() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = attach_critical_type(&m, "s");
        assert!(r.point_data().get_array("CriticalType").is_some());
    }
}
