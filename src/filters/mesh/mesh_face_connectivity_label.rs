//! Label faces by connected component.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn label_face_components(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let n = c.len();
        for i in 0..n {
            let a = c[i] as usize;
            let b = c[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut parent: Vec<usize> = (0..nc).collect();
    for (_, faces) in &ef {
        for i in 1..faces.len() {
            union(&mut parent, faces[0], faces[i]);
        }
    }
    let mut label_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut next = 0;
    let labels: Vec<f64> = (0..nc)
        .map(|i| {
            let root = find(&mut parent, i);
            *label_map.entry(root).or_insert_with(|| {
                let l = next;
                next += 1;
                l
            }) as f64
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Component",
            labels,
            1,
        )));
    r
}
pub fn num_face_components(mesh: &PolyData) -> usize {
    let r = label_face_components(mesh);
    let arr = r.cell_data().get_array("Component").unwrap();
    let mut buf = [0.0f64];
    let mut mx = 0.0f64;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        mx = mx.max(buf[0]);
    }
    mx as usize + 1
}
fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra != rb {
        p[rb] = ra;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert_eq!(num_face_components(&m), 1);
    }
    #[test]
    fn test_two() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        assert_eq!(num_face_components(&m), 2);
    }
}
