use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract cross-section areas along an axis.
///
/// Slices the mesh at `num_slices` evenly-spaced planes perpendicular
/// to the given axis, computes the approximate area of each slice,
/// and returns a PolyData with slice center points and "Area" scalars.
pub fn cross_section_areas(
    input: &PolyData,
    axis: usize, // 0=X, 1=Y, 2=Z
    num_slices: usize,
) -> PolyData {
    use crate::data::DataSet;
    let bb = input.bounds();
    let (lo, hi) = match axis {
        0 => (bb.x_min, bb.x_max),
        1 => (bb.y_min, bb.y_max),
        _ => (bb.z_min, bb.z_max),
    };

    let range = (hi - lo).max(1e-15);
    let num = num_slices.max(2);

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut areas = Vec::with_capacity(num);
    let mut positions = Vec::with_capacity(num);

    for i in 0..num {
        let t = lo + range * (i as f64 + 0.5) / num as f64;
        let slice_lo = lo + range * i as f64 / num as f64;
        let slice_hi = lo + range * (i + 1) as f64 / num as f64;

        // Sum triangle areas that intersect this slice
        let mut slice_area = 0.0;
        for cell in input.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let v0 = input.points.get(cell[0] as usize);
            for j in 1..cell.len() - 1 {
                let v1 = input.points.get(cell[j] as usize);
                let v2 = input.points.get(cell[j + 1] as usize);
                let vals = [v0[axis], v1[axis], v2[axis]];
                let min_v = vals[0].min(vals[1]).min(vals[2]);
                let max_v = vals[0].max(vals[1]).max(vals[2]);
                if max_v >= slice_lo && min_v <= slice_hi {
                    // Approximate: fraction of triangle in slice
                    let tri_range = (max_v - min_v).max(1e-15);
                    let overlap = (max_v.min(slice_hi) - min_v.max(slice_lo)).max(0.0);
                    let frac = overlap / tri_range;
                    slice_area += triangle_area(v0, v1, v2) * frac;
                }
            }
        }

        let mut center = [0.0; 3];
        center[axis] = t;
        let idx = out_points.len() as i64;
        out_points.push(center);
        out_verts.push_cell(&[idx]);
        areas.push(slice_area);
        positions.push(t);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Area", areas, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Position", positions, 1,
        )));
    pd
}

fn triangle_area(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_triangle_cross_section() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cross_section_areas(&pd, 0, 5); // along X
        assert_eq!(result.points.len(), 5);
        assert!(result.point_data().get_array("Area").is_some());
        assert!(result.point_data().get_array("Position").is_some());
    }

    #[test]
    fn areas_sum_reasonable() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cross_section_areas(&pd, 0, 10);
        let arr = result.point_data().get_array("Area").unwrap();
        let mut buf = [0.0f64];
        let mut total = 0.0;
        for i in 0..10 {
            arr.tuple_as_f64(i, &mut buf);
            total += buf[0];
            assert!(buf[0] >= 0.0);
        }
        assert!(total > 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = cross_section_areas(&pd, 2, 5);
        assert_eq!(result.points.len(), 5);
    }
}
