use crate::data::{Points, PolyData};

/// Spring-mass relaxation: treat edges as springs.
///
/// Each edge acts as a spring with rest length `rest_length`.
/// Vertices are iteratively displaced by spring forces with damping.
/// If `rest_length` is 0, uses the original edge lengths.
pub fn spring_relaxation(
    input: &PolyData,
    rest_length: f64,
    stiffness: f64,
    damping: f64,
    iterations: usize,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa = input.points.get(a);
                let pb = input.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                let rl = if rest_length > 0.0 { rest_length } else { d };
                edges.push((a, b, rl));
            }
        }
    }

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut vel = vec![[0.0f64; 3]; n];

    for _ in 0..iterations {
        let mut force = vec![[0.0f64; 3]; n];

        for &(a, b, rl) in &edges {
            let d = [
                pts[b][0] - pts[a][0],
                pts[b][1] - pts[a][1],
                pts[b][2] - pts[a][2],
            ];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if len < 1e-15 {
                continue;
            }
            let f = stiffness * (len - rl);
            let fx = f * d[0] / len;
            let fy = f * d[1] / len;
            let fz = f * d[2] / len;
            force[a][0] += fx;
            force[a][1] += fy;
            force[a][2] += fz;
            force[b][0] -= fx;
            force[b][1] -= fy;
            force[b][2] -= fz;
        }

        for i in 0..n {
            vel[i][0] = (vel[i][0] + force[i][0]) * damping;
            vel[i][1] = (vel[i][1] + force[i][1]) * damping;
            vel[i][2] = (vel[i][2] + force[i][2]) * damping;
            pts[i][0] += vel[i][0];
            pts[i][1] += vel[i][1];
            pts[i][2] += vel[i][2];
        }
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
    fn relaxation_moves() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([2.5, 4.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = spring_relaxation(&pd, 2.0, 0.1, 0.9, 20);
        assert_eq!(result.points.len(), 3);
        // Points should have moved somewhat
    }

    #[test]
    fn zero_stiffness_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1]);

        let result = spring_relaxation(&pd, 1.0, 0.0, 0.9, 10);
        assert!((result.points.get(0)[0]).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = spring_relaxation(&pd, 1.0, 0.1, 0.9, 10);
        assert_eq!(result.points.len(), 0);
    }
}
