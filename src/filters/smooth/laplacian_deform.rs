use crate::data::{Points, PolyData};

/// Simple Laplacian deformation: move a set of handle points and
/// propagate the deformation smoothly to the rest of the mesh.
///
/// `handles` is a list of (point_index, target_position) pairs.
/// `iterations` controls how many smoothing passes to run.
/// The handle points are pinned to their targets while other
/// points are iteratively smoothed toward neighborhood averages.
pub fn laplacian_deform(
    input: &PolyData,
    handles: &[(usize, [f64; 3])],
    iterations: usize,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Build adjacency
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Set handles
    let handle_set: std::collections::HashMap<usize, [f64; 3]> = handles.iter().cloned().collect();

    for h in handles {
        if h.0 < n {
            pts[h.0] = h.1;
        }
    }

    // Iterative smoothing with handle constraints
    for _ in 0..iterations {
        let mut new_pts = pts.clone();
        for i in 0..n {
            if handle_set.contains_key(&i) {
                continue;
            }
            if neighbors[i].is_empty() {
                continue;
            }

            let cnt = neighbors[i].len() as f64;
            let mut avg = [0.0; 3];
            for &j in &neighbors[i] {
                avg[0] += pts[j][0];
                avg[1] += pts[j][1];
                avg[2] += pts[j][2];
            }
            // Blend: 50% original + 50% neighbor average
            new_pts[i] = [
                0.5 * pts[i][0] + 0.5 * avg[0] / cnt,
                0.5 * pts[i][1] + 0.5 * avg[1] / cnt,
                0.5 * pts[i][2] + 0.5 * avg[2] / cnt,
            ];
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_stays_pinned() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = laplacian_deform(&pd, &[(0, [0.0, 5.0, 0.0])], 10);
        let p = result.points.get(0);
        assert_eq!(p[1], 5.0); // handle pinned
    }

    #[test]
    fn deformation_propagates() {
        let mut pd = PolyData::new();
        // Line of 5 points
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = laplacian_deform(&pd, &[(2, [2.0, 5.0, 0.0])], 20);
        // Neighbors should have moved somewhat
        let p1 = result.points.get(1);
        assert!(p1[1] > 0.0); // propagated
    }

    #[test]
    fn no_handles_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = laplacian_deform(&pd, &[], 10);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = laplacian_deform(&pd, &[(0, [1.0, 0.0, 0.0])], 5);
        assert_eq!(result.points.len(), 0);
    }
}
