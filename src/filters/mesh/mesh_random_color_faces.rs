//! Assign distinct colors to faces for visualization.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn distinct_face_colors(mesh: &PolyData) -> PolyData {
    let nc = mesh.polys.num_cells();
    let palette: [[f64; 3]; 12] = [
        [228.0, 26.0, 28.0],
        [55.0, 126.0, 184.0],
        [77.0, 175.0, 74.0],
        [152.0, 78.0, 163.0],
        [255.0, 127.0, 0.0],
        [255.0, 255.0, 51.0],
        [166.0, 86.0, 40.0],
        [247.0, 129.0, 191.0],
        [153.0, 153.0, 153.0],
        [0.0, 128.0, 128.0],
        [128.0, 0.0, 0.0],
        [0.0, 0.0, 128.0],
    ];
    let data: Vec<f64> = (0..nc)
        .flat_map(|i| {
            let c = palette[i % 12];
            vec![c[0], c[1], c[2]]
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", data, 3)));
    r
}
pub fn face_colors_by_connectivity(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
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
    let labels: Vec<usize> = (0..nc)
        .map(|i| {
            let root = find(&mut parent, i);
            *label_map.entry(root).or_insert_with(|| {
                let l = next;
                next += 1;
                l
            })
        })
        .collect();
    let palette: [[f64; 3]; 8] = [
        [255.0, 0.0, 0.0],
        [0.0, 255.0, 0.0],
        [0.0, 0.0, 255.0],
        [255.0, 255.0, 0.0],
        [255.0, 0.0, 255.0],
        [0.0, 255.0, 255.0],
        [255.0, 128.0, 0.0],
        [128.0, 0.0, 255.0],
    ];
    let data: Vec<f64> = labels
        .iter()
        .flat_map(|&l| {
            let c = palette[l % 8];
            vec![c[0], c[1], c[2]]
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", data, 3)));
    r
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
    fn test_distinct() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = distinct_face_colors(&m);
        assert!(r.cell_data().get_array("Colors").is_some());
    }
    #[test]
    fn test_connectivity() {
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
        let r = face_colors_by_connectivity(&m);
        assert!(r.cell_data().get_array("Colors").is_some());
    }
}
