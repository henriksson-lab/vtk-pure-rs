use crate::data::{CellArray, Points, PolyData};

/// Mirror a PolyData across a coordinate plane and optionally merge with original.
///
/// `plane`: 0=YZ (mirror X), 1=XZ (mirror Y), 2=XY (mirror Z).
/// If `merge` is true, appends the mirrored copy to the original.
pub fn mirror(input: &PolyData, plane: usize, merge: bool) -> PolyData {
    let n = input.points.len();
    let axis = plane.min(2);
    let mut mirrored_points = Points::<f64>::new();

    for i in 0..n {
        let p = input.points.get(i);
        let mut mp = p;
        mp[axis] = -mp[axis];
        mirrored_points.push(mp);
    }

    let mut mirrored_polys = CellArray::new();
    for cell in input.polys.iter() {
        let reflected = reflect_polygon_order(cell);
        if merge {
            let mapped: Vec<i64> = reflected.iter().map(|&id| id + n as i64).collect();
            mirrored_polys.push_cell(&mapped);
        } else {
            mirrored_polys.push_cell(&reflected);
        }
    }

    let mirrored_lines = reflect_lines(&input.lines, n as i64, merge);
    let mirrored_verts = shift_cells(&input.verts, n as i64, merge);
    let mirrored_strips = reflect_strips(&input.strips, n as i64, merge);

    if merge {
        let mut out_points = input.points.clone();
        for i in 0..n {
            out_points.push(mirrored_points.get(i));
        }
        let mut out_polys = input.polys.clone();
        for cell in mirrored_polys.iter() {
            out_polys.push_cell(cell);
        }
        let mut out_lines = input.lines.clone();
        for cell in mirrored_lines.iter() {
            out_lines.push_cell(cell);
        }
        let mut out_verts = input.verts.clone();
        for cell in mirrored_verts.iter() {
            out_verts.push_cell(cell);
        }
        let mut out_strips = input.strips.clone();
        for cell in mirrored_strips.iter() {
            out_strips.push_cell(cell);
        }

        let mut pd = PolyData::new();
        pd.points = out_points;
        pd.verts = out_verts;
        pd.lines = out_lines;
        pd.polys = out_polys;
        pd.strips = out_strips;
        pd
    } else {
        let mut pd = PolyData::new();
        pd.points = mirrored_points;
        pd.verts = mirrored_verts;
        pd.lines = mirrored_lines;
        pd.polys = mirrored_polys;
        pd.strips = mirrored_strips;
        pd
    }
}

fn reflect_polygon_order(cell: &[i64]) -> Vec<i64> {
    let n = cell.len();
    let mut reflected = vec![0; n];
    for (j, &id) in cell.iter().enumerate() {
        reflected[(n - j) % n] = id;
    }
    reflected
}

fn reflect_lines(input: &CellArray, offset: i64, merge: bool) -> CellArray {
    let mut output = CellArray::new();
    for cell in input.iter() {
        let reflected: Vec<i64> = if cell.len() > 2 {
            cell.iter().rev().copied().collect()
        } else {
            cell.to_vec()
        };
        let mapped: Vec<i64> = reflected
            .iter()
            .map(|&id| id + if merge { offset } else { 0 })
            .collect();
        output.push_cell(&mapped);
    }
    output
}

fn reflect_strips(input: &CellArray, offset: i64, merge: bool) -> CellArray {
    let mut output = CellArray::new();
    for cell in input.iter() {
        let reflected = reflect_polygon_order(cell);
        let mapped: Vec<i64> = reflected
            .iter()
            .map(|&id| id + if merge { offset } else { 0 })
            .collect();
        output.push_cell(&mapped);
    }
    output
}

fn shift_cells(input: &CellArray, offset: i64, merge: bool) -> CellArray {
    let mut output = CellArray::new();
    for cell in input.iter() {
        let mapped: Vec<i64> = cell
            .iter()
            .map(|&id| id + if merge { offset } else { 0 })
            .collect();
        output.push_cell(&mapped);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_x() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = mirror(&pd, 0, false);
        let p = result.points.get(0);
        assert_eq!(p[0], -1.0);
    }

    #[test]
    fn mirror_merge() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = mirror(&pd, 0, true);
        assert_eq!(result.points.len(), 6); // original + mirror
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn mirror_preserves_line_cells() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = mirror(&pd, 0, true);
        assert_eq!(result.lines.num_cells(), 2);
        assert_eq!(result.lines.cell(1), &[2, 3]);
    }

    #[test]
    fn mirror_z() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 5.0]);
        let result = mirror(&pd, 2, false);
        assert_eq!(result.points.get(0)[2], -5.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = mirror(&pd, 0, true);
        assert_eq!(result.points.len(), 0);
    }
}
