//! Discrete Gauss map (map mesh to unit sphere via normals).
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn gauss_map(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        pts.push(nm[i]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = mesh.polys.clone();
    r
}
pub fn gauss_map_area_ratio(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let nm = calc_nm(mesh);
    let mut data = Vec::with_capacity(nc);
    for c in &cells {
        if c.len() < 3 {
            data.push(0.0);
            continue;
        }
        // Original triangle area
        let a = mesh.points.get(c[0] as usize);
        let b = mesh.points.get(c[1] as usize);
        let cc2 = mesh.points.get(c[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [cc2[0] - a[0], cc2[1] - a[1], cc2[2] - a[2]];
        let orig_area = 0.5
            * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
        // Gauss map triangle area
        let na = nm[c[0] as usize];
        let nb2 = nm[c[1] as usize];
        let nc2 = nm[c[2] as usize];
        let ge1 = [nb2[0] - na[0], nb2[1] - na[1], nb2[2] - na[2]];
        let ge2 = [nc2[0] - na[0], nc2[1] - na[1], nc2[2] - na[2]];
        let gauss_area = 0.5
            * ((ge1[1] * ge2[2] - ge1[2] * ge2[1]).powi(2)
                + (ge1[2] * ge2[0] - ge1[0] * ge2[2]).powi(2)
                + (ge1[0] * ge2[1] - ge1[1] * ge2[0]).powi(2))
            .sqrt();
        data.push(if orig_area > 1e-15 {
            gauss_area / orig_area
        } else {
            0.0
        });
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussMapRatio",
            data,
            1,
        )));
    r
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_map() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let g = gauss_map(&m);
        assert_eq!(g.points.len(), 4);
        // All normals should be on unit sphere
        for i in 0..4 {
            let p = g.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 1.0).abs() < 0.1);
        }
    }
    #[test]
    fn test_ratio() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = gauss_map_area_ratio(&m);
        assert!(r.cell_data().get_array("GaussMapRatio").is_some());
    }
}
