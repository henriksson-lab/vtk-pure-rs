use crate::data::PolyData;

/// Find the closest pair of points in a point set.
///
/// Returns (index_a, index_b, distance). Uses brute force for
/// small sets, which is O(n²) but simple and correct.
pub fn closest_pair(input: &PolyData) -> Option<(usize, usize, f64)> {
    let n = input.points.len();
    if n < 2 {
        return None;
    }

    let mut best_d2 = f64::MAX;
    let mut best_a = 0;
    let mut best_b = 1;

    for i in 0..n {
        let pi = input.points.get(i);
        for j in i + 1..n {
            let pj = input.points.get(j);
            let d2 = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            if d2 < best_d2 {
                best_d2 = d2;
                best_a = i;
                best_b = j;
            }
        }
    }

    Some((best_a, best_b, best_d2.sqrt()))
}

/// Find the farthest pair of points (diameter of point set).
pub fn farthest_pair(input: &PolyData) -> Option<(usize, usize, f64)> {
    let n = input.points.len();
    if n < 2 {
        return None;
    }

    let mut best_d2 = 0.0f64;
    let mut best_a = 0;
    let mut best_b = 1;

    for i in 0..n {
        let pi = input.points.get(i);
        for j in i + 1..n {
            let pj = input.points.get(j);
            let d2 = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            if d2 > best_d2 {
                best_d2 = d2;
                best_a = i;
                best_b = j;
            }
        }
    }

    Some((best_a, best_b, best_d2.sqrt()))
}

/// Compute the diameter of the point set (max pairwise distance).
pub fn point_set_diameter(input: &PolyData) -> f64 {
    farthest_pair(input).map(|(_, _, d)| d).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]); // closest to point 0

        let (a, b, d) = closest_pair(&pd).unwrap();
        assert!((d - 0.1).abs() < 1e-10);
        assert!((a == 0 && b == 2) || (a == 2 && b == 0));
    }

    #[test]
    fn farthest_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let (_, _, d) = farthest_pair(&pd).unwrap();
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn diameter() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        assert!((point_set_diameter(&pd) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        assert!(closest_pair(&pd).is_none());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(closest_pair(&pd).is_none());
        assert_eq!(point_set_diameter(&pd), 0.0);
    }
}
