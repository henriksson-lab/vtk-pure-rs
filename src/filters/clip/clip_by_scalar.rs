use crate::data::{CellArray, Points, PolyData};

/// Clip a mesh by a scalar field, keeping the region where scalar >= isovalue.
///
/// Triangles that straddle the isovalue are split by linear interpolation,
/// producing new vertices on the isovalue boundary. This is similar to
/// `clip_by_plane` but uses an arbitrary scalar field.
pub fn clip_by_scalar(input: &PolyData, scalars: &str, isovalue: f64, invert: bool) -> PolyData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    let mut svals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (i, v) in svals.iter_mut().enumerate() {
        arr.tuple_as_f64(i, &mut buf);
        *v = buf[0];
    }

    let mut points = input.points.clone();
    let mut polys = CellArray::new();

    let inside = |s: f64| -> bool {
        if invert {
            s <= isovalue
        } else {
            s >= isovalue
        }
    };

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Fan-triangulate
        for ti in 1..cell.len() - 1 {
            let ids = [cell[0], cell[ti], cell[ti + 1]];
            let sv = [
                svals[ids[0] as usize],
                svals[ids[1] as usize],
                svals[ids[2] as usize],
            ];
            let all_in = ids.iter().all(|&id| inside(svals[id as usize]));
            let all_out = ids.iter().all(|&id| !inside(svals[id as usize]));

            if all_in {
                polys.push_cell(&ids);
            } else if !all_out {
                // Clip triangle
                let verts: Vec<[f64; 3]> = ids
                    .iter()
                    .map(|&id| input.points.get(id as usize))
                    .collect();
                let clipped = clip_triangle(&ids, &verts, &sv, isovalue, invert, &mut points);
                if clipped.len() >= 3 {
                    for i in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                    }
                }
            }
        }
    }

    compact_poly_data(points, polys)
}

fn clip_triangle(
    ids: &[i64; 3],
    verts: &[[f64; 3]],
    scalars: &[f64; 3],
    isovalue: f64,
    invert: bool,
    points: &mut Points<f64>,
) -> Vec<i64> {
    let inside = |s: f64| -> bool {
        if invert {
            s <= isovalue
        } else {
            s >= isovalue
        }
    };

    let mut result = Vec::new();
    for i in 0..3 {
        let j = (i + 1) % 3;
        let si = scalars[i];
        let sj = scalars[j];

        if inside(si) {
            result.push(ids[i]);
        }

        if inside(si) != inside(sj) {
            let ds = sj - si;
            if ds.abs() > 1e-15 {
                let t = ((isovalue - si) / ds).clamp(0.0, 1.0);
                let p = [
                    verts[i][0] + t * (verts[j][0] - verts[i][0]),
                    verts[i][1] + t * (verts[j][1] - verts[i][1]),
                    verts[i][2] + t * (verts[j][2] - verts[i][2]),
                ];
                let idx = points.len() as i64;
                points.push(p);
                result.push(idx);
            }
        }
    }
    result
}

fn compact_poly_data(points: Points<f64>, polys: CellArray) -> PolyData {
    let mut used = vec![false; points.len()];
    for cell in polys.iter() {
        for &id in cell {
            used[id as usize] = true;
        }
    }

    let mut point_map = vec![0i64; points.len()];
    let mut compact_points = Points::new();
    for (old_id, is_used) in used.into_iter().enumerate() {
        if is_used {
            point_map[old_id] = compact_points.len() as i64;
            compact_points.push(points.get(old_id));
        }
    }

    let mut compact_polys = CellArray::new();
    for cell in polys.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        compact_polys.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = compact_points;
    pd.polys = compact_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn clip_half() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 0.5],
                1,
            )));

        let result = clip_by_scalar(&pd, "s", 0.5, false);
        assert!(result.polys.num_cells() >= 1);
    }

    #[test]
    fn all_inside() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![5.0, 5.0, 5.0],
                1,
            )));

        let result = clip_by_scalar(&pd, "s", 0.0, false);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn all_outside() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![-1.0, -1.0, -1.0],
                1,
            )));

        let result = clip_by_scalar(&pd, "s", 0.0, false);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn invert_clip() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![5.0, 5.0, 5.0],
                1,
            )));

        // Invert: keep s < 0 -> nothing kept
        let result = clip_by_scalar(&pd, "s", 0.0, true);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
