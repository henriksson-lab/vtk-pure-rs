use crate::data::{CellArray, PolyData};

/// Remove T-junctions by splitting edges at T-junction vertices.
///
/// A T-junction occurs when a vertex lies on an edge of another triangle
/// but is not a vertex of that triangle. This inserts the vertex into
/// the edge, splitting the triangle.
pub fn fix_t_junctions(input: &PolyData, tolerance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let t2 = tolerance * tolerance;

    // For each edge, check if any non-adjacent vertex is within tolerance
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let out_pts = input.points.clone();
    let mut new_cells = cells.clone();
    let mut modified = true;
    let mut pass = 0;

    while modified && pass < 3 {
        modified = false;
        pass += 1;
        let mut replacement: Vec<Option<Vec<Vec<i64>>>> = vec![None; new_cells.len()];

        for (fi, cell) in new_cells.iter().enumerate() {
            if !is_valid_triangle(cell, out_pts.len()) {
                continue;
            }
            for i in 0..cell.len() {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % cell.len()] as usize;
                let pa = out_pts.get(a);
                let pb = out_pts.get(b);

                // Check all vertices not in this cell
                for vi in 0..out_pts.len() {
                    if cell.contains(&(vi as i64)) {
                        continue;
                    }
                    let p = out_pts.get(vi);

                    // Point-to-segment distance
                    let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
                    let ap = [p[0] - pa[0], p[1] - pa[1], p[2] - pa[2]];
                    let ab_len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                    if ab_len2 < 1e-20 {
                        continue;
                    }
                    let t_param = (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab_len2;
                    if t_param < 0.01 || t_param > 0.99 {
                        continue;
                    }

                    let proj = [
                        pa[0] + t_param * ab[0],
                        pa[1] + t_param * ab[1],
                        pa[2] + t_param * ab[2],
                    ];
                    let d2 =
                        (p[0] - proj[0]).powi(2) + (p[1] - proj[1]).powi(2) + (p[2] - proj[2]).powi(2);
                    if d2 <= t2 {
                        // Split triangle at this edge
                        let vid = vi as i64;
                        let c_idx = (i + 2) % cell.len(); // opposite vertex
                        let opp = cell[c_idx];
                        let tri1 = vec![cell[i], vid, opp];
                        let tri2 = vec![vid, cell[(i + 1) % cell.len()], opp];
                        replacement[fi] = Some(vec![tri1, tri2]);
                        modified = true;
                        break;
                    }
                }
                if replacement[fi].is_some() {
                    break;
                }
            }
        }

        // Apply replacements
        if modified {
            let mut next_cells = Vec::new();
            for (fi, cell) in new_cells.iter().enumerate() {
                if let Some(ref rep) = replacement[fi] {
                    for r in rep {
                        next_cells.push(r.clone());
                    }
                } else {
                    next_cells.push(cell.clone());
                }
            }
            new_cells=next_cells;
        }
    }

    let mut out_polys = CellArray::new();
    for c in &new_cells {
        out_polys.push_cell(c);
    }

    let mut pd = input.clone();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

fn is_valid_triangle(cell: &[i64], n_points: usize) -> bool {
    cell.len() == 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_t_junctions() {
        let mut pd=PolyData::new();
        pd.points.push([0.0,0.0,0.0]);pd.points.push([1.0,0.0,0.0]);pd.points.push([0.5,1.0,0.0]);
        pd.polys.push_cell(&[0,1,2]);

        let result=fix_t_junctions(&pd,0.01);
        assert_eq!(result.polys.num_cells(),1);
    }

    #[test]
    fn runs_without_panic() {
        let mut pd=PolyData::new();
        pd.points.push([0.0,0.0,0.0]);pd.points.push([2.0,0.0,0.0]);pd.points.push([1.0,2.0,0.0]);
        pd.points.push([1.0,0.0,0.0]); // on edge 0-1
        pd.polys.push_cell(&[0,1,2]);

        let result=fix_t_junctions(&pd,0.01);
        assert!(result.polys.num_cells()>=1);
    }

    #[test]
    fn empty_input() {
        let pd=PolyData::new();
        assert_eq!(fix_t_junctions(&pd,0.01).polys.num_cells(),0);
    }
}
