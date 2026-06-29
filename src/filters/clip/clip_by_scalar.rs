use crate::data::{CellArray, Points, PolyData};

/// Clip a mesh by a scalar field, keeping the region where scalar > isovalue.
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

    let inside = |s: f64| -> bool {
        if invert {
            s <= isovalue
        } else {
            s > isovalue
        }
    };

    let mut points = input.points.clone();
    let mut point_locator = PointLocator::from_points(&points);
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();

    for cell in input.verts.iter() {
        for &id in cell {
            if inside(svals[id as usize]) {
                verts.push_cell(&[id]);
            }
        }
    }

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        clip_polyline(
            cell,
            &svals,
            isovalue,
            invert,
            &input.points,
            &mut points,
            &mut point_locator,
            &mut lines,
        );
    }

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
                let clipped = clip_triangle(
                    &ids,
                    &verts,
                    &sv,
                    isovalue,
                    invert,
                    &mut points,
                    &mut point_locator,
                );
                if clipped.len() >= 3 {
                    for i in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                    }
                }
            }
        }
    }

    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for ti in 0..strip.len() - 2 {
            let ids = if ti % 2 == 0 {
                [strip[ti], strip[ti + 1], strip[ti + 2]]
            } else {
                [strip[ti + 2], strip[ti + 1], strip[ti]]
            };
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
                let verts_in: Vec<[f64; 3]> = ids
                    .iter()
                    .map(|&id| input.points.get(id as usize))
                    .collect();
                let clipped = clip_triangle(
                    &ids,
                    &verts_in,
                    &sv,
                    isovalue,
                    invert,
                    &mut points,
                    &mut point_locator,
                );
                if clipped.len() >= 3 {
                    for i in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                    }
                }
            }
        }
    }

    compact_poly_data(points, verts, lines, polys)
}

#[derive(Default)]
struct PointLocator {
    points: Vec<[f64; 3]>,
}

impl PointLocator {
    fn from_points(points: &Points<f64>) -> Self {
        let mut locator = Self::default();
        for i in 0..points.len() {
            locator.points.push(points.get(i));
        }
        locator
    }

    fn insert_unique_point(&mut self, points: &mut Points<f64>, point: [f64; 3]) -> i64 {
        if let Some((id, _)) = self
            .points
            .iter()
            .enumerate()
            .find(|(_, existing)| same_point(**existing, point))
        {
            return id as i64;
        }

        let id = points.len() as i64;
        points.push(point);
        self.points.push(point);
        id
    }
}

fn same_point(a: [f64; 3], b: [f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= 1e-12 && (a[1] - b[1]).abs() <= 1e-12 && (a[2] - b[2]).abs() <= 1e-12
}

fn clip_triangle(
    ids: &[i64; 3],
    verts: &[[f64; 3]],
    scalars: &[f64; 3],
    isovalue: f64,
    invert: bool,
    points: &mut Points<f64>,
    point_locator: &mut PointLocator,
) -> Vec<i64> {
    let inside = |s: f64| -> bool {
        if invert {
            s <= isovalue
        } else {
            s > isovalue
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
                let idx = point_locator.insert_unique_point(points, p);
                result.push(idx);
            }
        }
    }
    result
}

fn clip_line_segment(
    ids: &[i64; 2],
    scalars: &[f64; 2],
    isovalue: f64,
    invert: bool,
    src_points: &Points<f64>,
    points: &mut Points<f64>,
    point_locator: &mut PointLocator,
) -> Vec<i64> {
    let inside = |s: f64| -> bool {
        if invert {
            s <= isovalue
        } else {
            s > isovalue
        }
    };

    let i_in = inside(scalars[0]);
    let j_in = inside(scalars[1]);
    match (i_in, j_in) {
        (true, true) => vec![ids[0], ids[1]],
        (false, false) => Vec::new(),
        _ => {
            let ds = scalars[1] - scalars[0];
            if ds.abs() <= 1e-15 {
                return Vec::new();
            }
            let t = ((isovalue - scalars[0]) / ds).clamp(0.0, 1.0);
            let pi = src_points.get(ids[0] as usize);
            let pj = src_points.get(ids[1] as usize);
            let p = [
                pi[0] + t * (pj[0] - pi[0]),
                pi[1] + t * (pj[1] - pi[1]),
                pi[2] + t * (pj[2] - pi[2]),
            ];
            let idx = point_locator.insert_unique_point(points, p);
            if i_in {
                vec![ids[0], idx]
            } else {
                vec![idx, ids[1]]
            }
        }
    }
}

fn clip_polyline(
    cell: &[i64],
    svals: &[f64],
    isovalue: f64,
    invert: bool,
    src_points: &Points<f64>,
    points: &mut Points<f64>,
    point_locator: &mut PointLocator,
    lines: &mut CellArray,
) {
    let mut current = Vec::new();

    for i in 0..cell.len() - 1 {
        let ids = [cell[i], cell[i + 1]];
        let scalars = [svals[ids[0] as usize], svals[ids[1] as usize]];
        let clipped = clip_line_segment(
            &ids,
            &scalars,
            isovalue,
            invert,
            src_points,
            points,
            point_locator,
        );

        if clipped.len() == 2 {
            if current.is_empty() {
                current.extend_from_slice(&clipped);
            } else if current.last() == Some(&clipped[0]) {
                current.push(clipped[1]);
            } else {
                if current.len() >= 2 {
                    lines.push_cell(&current);
                }
                current.clear();
                current.extend_from_slice(&clipped);
            }
        } else if current.len() >= 2 {
            lines.push_cell(&current);
            current.clear();
        }
    }

    if current.len() >= 2 {
        lines.push_cell(&current);
    }
}

fn compact_poly_data(
    points: Points<f64>,
    verts: CellArray,
    lines: CellArray,
    polys: CellArray,
) -> PolyData {
    let mut used = vec![false; points.len()];
    for cells in [&verts, &lines, &polys] {
        for cell in cells.iter() {
            for &id in cell {
                used[id as usize] = true;
            }
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

    let mut pd = PolyData::new();
    pd.points = compact_points;
    pd.verts = remap_cells(&verts, &point_map);
    pd.lines = remap_cells(&lines, &point_map);
    pd.polys = remap_cells(&polys, &point_map);
    pd
}

fn remap_cells(cells: &CellArray, point_map: &[i64]) -> CellArray {
    let mut remapped_cells = CellArray::new();
    for cell in cells.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        remapped_cells.push_cell(&remapped);
    }
    remapped_cells
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

        // Invert: keep s <= 0 -> nothing kept
        let result = clip_by_scalar(&pd, "s", 0.0, true);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn polyline_keeps_contiguous_segments_as_one_cell() {
        let mut pd =
            PolyData::from_polyline(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let result = clip_by_scalar(&pd, "s", 0.0, false);

        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.lines.cell(0).len(), 3);
    }

    #[test]
    fn strip_uses_vtk_odd_triangle_order() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        pd.strips.push_cell(&[0, 1, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 1.0, 1.0, 1.0],
                1,
            )));

        let result = clip_by_scalar(&pd, "s", 0.0, false);

        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[3, 2, 1]);
    }
}
