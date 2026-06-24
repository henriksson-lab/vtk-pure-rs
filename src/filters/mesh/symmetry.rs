use crate::data::{DataSet, KdTree, PolyData};

/// Check if a mesh is approximately symmetric across a coordinate plane.
///
/// Tests bilateral symmetry by checking if each point has a corresponding
/// point on the other side of the plane within tolerance.
/// Returns the fraction of points with a match (1.0 = perfectly symmetric).
pub fn symmetry_score(input: &PolyData, plane: usize, tolerance: f64) -> f64 {
    let n = input.points.len();
    if n == 0 {
        return 1.0;
    }

    let bb = input.bounds();
    let center = match plane {
        0 => (bb.x_min + bb.x_max) * 0.5,
        1 => (bb.y_min + bb.y_max) * 0.5,
        _ => (bb.z_min + bb.z_max) * 0.5,
    };

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);
    let tol2 = tolerance * tolerance;

    let mut matched = 0;
    for i in 0..n {
        let p = pts[i];
        let mut mirror = p;
        mirror[plane.min(2)] = 2.0 * center - mirror[plane.min(2)];

        if let Some((_, d2)) = tree.nearest(mirror) {
            if d2 <= tol2 {
                matched += 1;
            }
        }
    }

    matched as f64 / n as f64
}

/// Find the best symmetry plane (X, Y, or Z) for a mesh.
///
/// Returns (plane_index, score) where plane_index is 0=YZ, 1=XZ, 2=XY.
pub fn best_symmetry_plane(input: &PolyData, tolerance: f64) -> (usize, f64) {
    let scores = [
        symmetry_score(input, 0, tolerance),
        symmetry_score(input, 1, tolerance),
        symmetry_score(input, 2, tolerance),
    ];
    let best = if scores[0] >= scores[1] && scores[0] >= scores[2] {
        0
    } else if scores[1] >= scores[2] {
        1
    } else {
        2
    };
    (best, scores[best])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symmetric_mesh() {
        let mut pd = PolyData::new();
        // Symmetric across YZ plane (X=0)
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);

        let score = symmetry_score(&pd, 0, 0.1);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn asymmetric_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]); // no mirror at -1 relative to center 1.5
        pd.points.push([0.5, 1.0, 0.0]); // no mirror

        let score = symmetry_score(&pd, 0, 0.01);
        assert!(score < 1.0, "score = {}", score);
    }

    #[test]
    fn best_plane() {
        let mut pd = PolyData::new();
        // Symmetric across XZ plane (Y=0) but not X or Z
        pd.points.push([0.0, -1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([3.0, -1.0, 0.0]);
        pd.points.push([3.0, 1.0, 0.0]);
        pd.points.push([5.0, -1.0, 0.0]);
        pd.points.push([5.0, 1.0, 0.0]);

        let (plane, score) = best_symmetry_plane(&pd, 0.1);
        assert_eq!(plane, 1); // Y symmetry
        assert_eq!(score, 1.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(symmetry_score(&pd, 0, 0.1), 1.0);
    }
}
