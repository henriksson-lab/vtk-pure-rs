use crate::data::{CellArray, Points, PolyData};

/// Mirror a PolyData across a coordinate plane and optionally merge with original.
///
/// `plane`: 0=YZ (mirror X), 1=XZ (mirror Y), 2=XY (mirror Z).
/// If `merge` is true, appends the mirrored copy to the original.
pub fn mirror(input: &PolyData, plane: usize, merge: bool) -> PolyData {
    let n = input.points.len();
    let mut mirrored_points = Points::<f64>::new();

    for i in 0..n {
        let p = input.points.get(i);
        let mut mp = p;
        mp[plane.min(2)] = -mp[plane.min(2)];
        mirrored_points.push(mp);
    }

    // Reverse winding for mirrored cells
    let mut mirrored_polys = CellArray::new();
    for cell in input.polys.iter() {
        let mut rev: Vec<i64> = cell.to_vec();
        rev.reverse();
        if merge {
            // Offset indices
            let mapped: Vec<i64> = rev.iter().map(|&id| id + n as i64).collect();
            mirrored_polys.push_cell(&mapped);
        } else {
            mirrored_polys.push_cell(&rev);
        }
    }

    if merge {
        let mut out_points = input.points.clone();
        for i in 0..n {
            out_points.push(mirrored_points.get(i));
        }
        let mut out_polys = input.polys.clone();
        for cell in mirrored_polys.iter() {
            out_polys.push_cell(cell);
        }

        let mut pd = PolyData::new();
        pd.points = out_points;
        pd.polys = out_polys;
        pd
    } else {
        let mut pd = PolyData::new();
        pd.points = mirrored_points;
        pd.polys = mirrored_polys;
        pd
    }
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
