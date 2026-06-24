//! Split edges longer than a threshold by inserting midpoints.
use crate::data::{CellArray, Points, PolyData};

pub fn edge_split(mesh: &PolyData, max_edge_length: f64) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.is_empty() {
        return mesh.clone();
    }
    let max2 = max_edge_length * max_edge_length;
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        pts.push(mesh.points.get(i).try_into().unwrap());
    }
    let mut midpoints: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut get_mid = |pts: &mut Points<f64>, a: usize, b: usize| -> usize {
        let e = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = midpoints.get(&e) {
            return idx;
        }
        let pa = pts.get(a);
        let pb = pts.get(b);
        let idx = pts.len();
        pts.push([
            (pa[0] + pb[0]) / 2.0,
            (pa[1] + pb[1]) / 2.0,
            (pa[2] + pb[2]) / 2.0,
        ]);
        midpoints.insert(e, idx);
        idx
    };
    let mut polys = CellArray::new();
    for &[a, b, c] in &tris {
        if a >= n || b >= n || c >= n {
            polys.push_cell(&[a as i64, b as i64, c as i64]);
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let d_ab = (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);
        let d_bc = (pb[0] - pc[0]).powi(2) + (pb[1] - pc[1]).powi(2) + (pb[2] - pc[2]).powi(2);
        let d_ca = (pc[0] - pa[0]).powi(2) + (pc[1] - pa[1]).powi(2) + (pc[2] - pa[2]).powi(2);
        let split_ab = d_ab > max2;
        let split_bc = d_bc > max2;
        let split_ca = d_ca > max2;
        match (split_ab, split_bc, split_ca) {
            (false, false, false) => {
                polys.push_cell(&[a as i64, b as i64, c as i64]);
            }
            (true, false, false) => {
                let m = get_mid(&mut pts, a, b);
                polys.push_cell(&[a as i64, m as i64, c as i64]);
                polys.push_cell(&[m as i64, b as i64, c as i64]);
            }
            (false, true, false) => {
                let m = get_mid(&mut pts, b, c);
                polys.push_cell(&[a as i64, b as i64, m as i64]);
                polys.push_cell(&[a as i64, m as i64, c as i64]);
            }
            (false, false, true) => {
                let m = get_mid(&mut pts, c, a);
                polys.push_cell(&[a as i64, b as i64, m as i64]);
                polys.push_cell(&[m as i64, b as i64, c as i64]);
            }
            _ => {
                // Multiple splits: do full midpoint subdivision
                let mab = get_mid(&mut pts, a, b);
                let mbc = get_mid(&mut pts, b, c);
                let mca = get_mid(&mut pts, c, a);
                polys.push_cell(&[a as i64, mab as i64, mca as i64]);
                polys.push_cell(&[mab as i64, b as i64, mbc as i64]);
                polys.push_cell(&[mca as i64, mbc as i64, c as i64]);
                polys.push_cell(&[mab as i64, mbc as i64, mca as i64]);
            }
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_split() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = edge_split(&mesh, 6.0); // all edges > 6
        assert!(r.polys.num_cells() > 1);
        assert!(r.points.len() > 3);
    }
}
