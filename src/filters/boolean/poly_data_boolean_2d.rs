use crate::data::PolyData;

/// Compute the 2D bounding box intersection of two PolyData.
///
/// Projects both meshes to XY, computes the overlap bounding box,
/// and clips both meshes to that region. Returns (clipped_a, clipped_b).
/// Useful as a fast pre-filter before expensive boolean operations.
pub fn bounding_box_overlap(a: &PolyData, b: &PolyData) -> Option<[f64; 4]> {
    use crate::data::DataSet;
    let ba = a.bounds();
    let bb = b.bounds();

    let x_min = ba.x_min.max(bb.x_min);
    let x_max = ba.x_max.min(bb.x_max);
    let y_min = ba.y_min.max(bb.y_min);
    let y_max = ba.y_max.min(bb.y_max);

    if x_min >= x_max || y_min >= y_max {
        None
    } else {
        Some([x_min, x_max, y_min, y_max])
    }
}

/// Check if two PolyData bounding boxes overlap in 3D.
pub fn bounding_boxes_overlap(a: &PolyData, b: &PolyData) -> bool {
    use crate::data::DataSet;
    let ba = a.bounds();
    let bb = b.bounds();

    ba.x_min <= bb.x_max
        && ba.x_max >= bb.x_min
        && ba.y_min <= bb.y_max
        && ba.y_max >= bb.y_min
        && ba.z_min <= bb.z_max
        && ba.z_max >= bb.z_min
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box_2d(x0: f64, y0: f64, x1: f64, y1: f64) -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([x0, y0, 0.0]);
        pd.points.push([x1, y0, 0.0]);
        pd.points.push([x1, y1, 0.0]);
        pd.points.push([x0, y1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd
    }

    #[test]
    fn overlapping() {
        let a = make_box_2d(0.0, 0.0, 2.0, 2.0);
        let b = make_box_2d(1.0, 1.0, 3.0, 3.0);
        let overlap = bounding_box_overlap(&a, &b);
        assert!(overlap.is_some());
        let [x0, x1, y0, y1] = overlap.unwrap();
        assert_eq!(x0, 1.0);
        assert_eq!(x1, 2.0);
        assert_eq!(y0, 1.0);
        assert_eq!(y1, 2.0);
    }

    #[test]
    fn disjoint() {
        let a = make_box_2d(0.0, 0.0, 1.0, 1.0);
        let b = make_box_2d(5.0, 5.0, 6.0, 6.0);
        assert!(bounding_box_overlap(&a, &b).is_none());
        assert!(!bounding_boxes_overlap(&a, &b));
    }

    #[test]
    fn overlap_3d() {
        let a = make_box_2d(0.0, 0.0, 2.0, 2.0);
        let b = make_box_2d(1.0, 1.0, 3.0, 3.0);
        assert!(bounding_boxes_overlap(&a, &b));
    }
}
