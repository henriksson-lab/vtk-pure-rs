use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extract crease/ridge lines from a mesh based on the dihedral angle
/// AND curvature direction. Returns lines along ridges and valleys.
pub fn extract_ridge_valley_lines(
    input: &PolyData,
    angle_threshold_deg: f64,
) -> (PolyData, PolyData) {
    let cos_thresh = angle_threshold_deg.to_radians().cos();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0; 3];
            }
            let v0 = input.points.get(c[0] as usize);
            let v1 = input.points.get(c[1] as usize);
            let v2 = input.points.get(c[2] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                [n[0] / l, n[1] / l, n[2] / l]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut ridge_pts = Points::<f64>::new();
    let mut ridge_lines = CellArray::new();
    let mut valley_pts = Points::<f64>::new();
    let mut valley_lines = CellArray::new();

    let mut r_map: HashMap<i64, i64> = HashMap::new();
    let mut v_map: HashMap<i64, i64> = HashMap::new();

    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let na = normals[faces[0]];
        let nb = normals[faces[1]];
        let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
        if dot >= cos_thresh {
            continue;
        } // not sharp enough

        // Determine ridge vs valley: is the dihedral convex or concave?
        let opp = cells[faces[1]].iter().find(|&&v| v != a && v != b);
        let is_ridge = if let Some(&opp_v) = opp {
            let p = input.points.get(opp_v as usize);
            let v0 = input.points.get(a as usize);
            let d = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];
            d[0] * na[0] + d[1] * na[1] + d[2] * na[2] < 0.0
        } else {
            true
        };

        if is_ridge {
            let ma = *r_map.entry(a).or_insert_with(|| {
                let i = ridge_pts.len() as i64;
                ridge_pts.push(input.points.get(a as usize));
                i
            });
            let mb = *r_map.entry(b).or_insert_with(|| {
                let i = ridge_pts.len() as i64;
                ridge_pts.push(input.points.get(b as usize));
                i
            });
            ridge_lines.push_cell(&[ma, mb]);
        } else {
            let ma = *v_map.entry(a).or_insert_with(|| {
                let i = valley_pts.len() as i64;
                valley_pts.push(input.points.get(a as usize));
                i
            });
            let mb = *v_map.entry(b).or_insert_with(|| {
                let i = valley_pts.len() as i64;
                valley_pts.push(input.points.get(b as usize));
                i
            });
            valley_lines.push_cell(&[ma, mb]);
        }
    }

    let mut ridges = PolyData::new();
    ridges.points = ridge_pts;
    ridges.lines = ridge_lines;
    let mut valleys = PolyData::new();
    valleys.points = valley_pts;
    valleys.lines = valley_lines;
    (ridges, valleys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ridge_on_fold() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.5]);
        pd.points.push([0.5, -1.0, 0.5]); // ridge fold
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let (ridges, valleys) = extract_ridge_valley_lines(&pd, 30.0);
        assert!(ridges.lines.num_cells() > 0 || valleys.lines.num_cells() > 0);
    }

    #[test]
    fn flat_no_ridges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let (ridges, valleys) = extract_ridge_valley_lines(&pd, 10.0);
        assert_eq!(ridges.lines.num_cells() + valleys.lines.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (r, v) = extract_ridge_valley_lines(&pd, 30.0);
        assert_eq!(r.lines.num_cells() + v.lines.num_cells(), 0);
    }
}
