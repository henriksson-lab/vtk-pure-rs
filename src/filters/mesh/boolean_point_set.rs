use crate::data::{CellArray, KdTree, Points, PolyData};

/// Compute the intersection of two point sets: points in A that have
/// a neighbor in B within tolerance.
pub fn point_set_intersection(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let na = a.points.len();
    let nb = b.points.len();
    if na == 0 || nb == 0 {
        return PolyData::new();
    }

    let b_pts: Vec<[f64; 3]> = (0..nb).map(|i| b.points.get(i)).collect();
    let tree = KdTree::build(&b_pts);
    let t2 = tolerance * tolerance;

    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for i in 0..na {
        let p = a.points.get(i);
        if let Some((_, d2)) = tree.nearest(p) {
            if d2 <= t2 {
                let idx = out_pts.len() as i64;
                out_pts.push(p);
                out_verts.push_cell(&[idx]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd
}

/// Compute the difference A \ B: points in A that do NOT have
/// a neighbor in B within tolerance.
pub fn point_set_difference(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let na = a.points.len();
    let nb = b.points.len();
    if na == 0 {
        return PolyData::new();
    }
    if nb == 0 {
        return a.clone();
    }

    let b_pts: Vec<[f64; 3]> = (0..nb).map(|i| b.points.get(i)).collect();
    let tree = KdTree::build(&b_pts);
    let t2 = tolerance * tolerance;

    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for i in 0..na {
        let p = a.points.get(i);
        let keep = match tree.nearest(p) {
            Some((_, d2)) => d2 > t2,
            None => true,
        };
        if keep {
            let idx = out_pts.len() as i64;
            out_pts.push(p);
            out_verts.push_cell(&[idx]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd
}

/// Compute the union of two point sets, removing duplicates within tolerance.
pub fn point_set_union(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let diff = point_set_difference(b, a, tolerance);
    // A + (B \ A)
    let na = a.points.len();
    let nd = diff.points.len();
    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for i in 0..na {
        let idx = out_pts.len() as i64;
        out_pts.push(a.points.get(i));
        out_verts.push_cell(&[idx]);
    }
    for i in 0..nd {
        let idx = out_pts.len() as i64;
        out_pts.push(diff.points.get(i));
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_overlap() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([2.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);
        b.points.push([3.0, 0.0, 0.0]);

        let result = point_set_intersection(&a, &b, 0.01);
        assert_eq!(result.points.len(), 2); // points 1.0 and 2.0
    }

    #[test]
    fn difference_removes() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]);

        let result = point_set_difference(&a, &b, 0.01);
        assert_eq!(result.points.len(), 1); // only 0.0 remains
    }

    #[test]
    fn union_deduplicates() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);

        let result = point_set_union(&a, &b, 0.01);
        assert_eq!(result.points.len(), 3); // 0, 1, 2
    }

    #[test]
    fn empty_inputs() {
        let a = PolyData::new();
        let b = PolyData::new();
        assert_eq!(point_set_intersection(&a, &b, 0.1).points.len(), 0);
        assert_eq!(point_set_difference(&a, &b, 0.1).points.len(), 0);
        assert_eq!(point_set_union(&a, &b, 0.1).points.len(), 0);
    }
}
