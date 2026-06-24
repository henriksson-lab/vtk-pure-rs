use crate::data::{CellArray, Points, PolyData};

/// Merge multiple PolyData into one. Bulk memcpy via flat slices for speed.
pub fn append(inputs: &[&PolyData]) -> PolyData {
    if inputs.is_empty() {
        return PolyData::new();
    }
    if inputs.len() == 1 {
        return inputs[0].clone();
    }

    let total_pts: usize = inputs.iter().map(|p| p.points.len()).sum();

    // Bulk copy point data via extend_from_slice on flat backing buffer.
    // Avoids per-point get()/push() overhead (single memcpy per input mesh).
    let mut pts_flat = Vec::with_capacity(total_pts * 3);
    for &input in inputs {
        pts_flat.extend_from_slice(input.points.as_flat_slice());
    }

    let polys = merge_cells(inputs, |p| &p.polys);
    let lines = merge_cells(inputs, |p| &p.lines);
    let verts = merge_cells(inputs, |p| &p.verts);

    let mut output = PolyData::new();
    output.points = Points::from_flat_vec(pts_flat);
    output.polys = polys;
    output.lines = lines;
    output.verts = verts;
    output
}

fn merge_cells(inputs: &[&PolyData], get: impl Fn(&PolyData) -> &CellArray) -> CellArray {
    let total_cells: usize = inputs.iter().map(|p| get(p).num_cells()).sum();
    if total_cells == 0 {
        return CellArray::new();
    }

    let total_conn: usize = inputs.iter().map(|p| get(p).connectivity_len()).sum();
    let mut offsets = Vec::with_capacity(total_cells + 1);
    let mut conn = Vec::with_capacity(total_conn);
    offsets.push(0i64);

    let mut pt_off: i64 = 0;
    for &input in inputs {
        let cells = get(input);
        let src_conn = cells.connectivity();
        if pt_off == 0 {
            conn.extend_from_slice(src_conn);
        } else {
            // Bulk offset: extend then add offset in-place
            let start = conn.len();
            conn.extend_from_slice(src_conn);
            for v in &mut conn[start..] {
                *v += pt_off;
            }
        }
        let base = *offsets.last().unwrap();
        let src_off = cells.offsets();
        if base == 0 {
            offsets.extend_from_slice(&src_off[1..]);
        } else {
            let start = offsets.len();
            offsets.extend_from_slice(&src_off[1..]);
            for v in &mut offsets[start..] {
                *v += base;
            }
        }
        pt_off += input.points.len() as i64;
    }
    CellArray::from_raw(offsets, conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn append_two() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = append(&[&a, &b]);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 2);
        assert_eq!(r.polys.cell(1), &[3, 4, 5]);
    }
}
