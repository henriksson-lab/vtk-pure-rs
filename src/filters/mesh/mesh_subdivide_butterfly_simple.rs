//! Butterfly subdivision (interpolating).
use crate::data::{CellArray, Points, PolyData};
pub fn subdivide_butterfly(mesh: &PolyData) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut new_polys = CellArray::new();
    let mut em: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    for cell in &cells {
        if cell.len() != 3 {
            new_polys.push_cell(cell);
            continue;
        }
        let v = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let m01 = get_butterfly_mid(&mut pts, &mut em, &ef, &cells, v[0], v[1]);
        let m12 = get_butterfly_mid(&mut pts, &mut em, &ef, &cells, v[1], v[2]);
        let m20 = get_butterfly_mid(&mut pts, &mut em, &ef, &cells, v[2], v[0]);
        new_polys.push_cell(&[v[0] as i64, m01 as i64, m20 as i64]);
        new_polys.push_cell(&[v[1] as i64, m12 as i64, m01 as i64]);
        new_polys.push_cell(&[v[2] as i64, m20 as i64, m12 as i64]);
        new_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
    }
    let mut new_pts = Points::<f64>::new();
    for p in &pts {
        new_pts.push(*p);
    }
    let mut r = PolyData::new();
    r.points = new_pts;
    r.polys = new_polys;
    r
}
fn get_butterfly_mid(
    pts: &mut Vec<[f64; 3]>,
    cache: &mut std::collections::HashMap<(usize, usize), usize>,
    ef: &std::collections::HashMap<(usize, usize), Vec<usize>>,
    cells: &[Vec<i64>],
    a: usize,
    b: usize,
) -> usize {
    let key = (a.min(b), a.max(b));
    *cache.entry(key).or_insert_with(|| {
        let pa = pts[a];
        let pb = pts[b];
        let mut mid = [
            (pa[0] + pb[0]) / 2.0,
            (pa[1] + pb[1]) / 2.0,
            (pa[2] + pb[2]) / 2.0,
        ];
        // Butterfly: add 1/8 of opposite vertices
        if let Some(faces) = ef.get(&key) {
            if faces.len() == 2 {
                for &fi in faces {
                    if let Some(&opp) = cells[fi]
                        .iter()
                        .find(|&&v| v as usize != a && v as usize != b)
                    {
                        let po = pts[opp as usize];
                        mid[0] += po[0] / 8.0;
                        mid[1] += po[1] / 8.0;
                        mid[2] += po[2] / 8.0;
                    }
                }
                mid[0] -= (pa[0] + pb[0]) / 8.0;
                mid[1] -= (pa[1] + pb[1]) / 8.0;
                mid[2] -= (pa[2] + pb[2]) / 8.0;
            }
        }
        let i = pts.len();
        pts.push(mid);
        i
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = subdivide_butterfly(&m);
        assert_eq!(r.polys.num_cells(), 4);
    }
}
