use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Scatter vertices outward from centroid by a factor.
///
/// Each vertex is displaced away from the mesh centroid by `factor`
/// (1.0 = no change, >1 = expand, <1 = contract).
pub fn scatter_from_centroid(input: &PolyData, factor: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([
            cx + (p[0] - cx) * factor,
            cy + (p[1] - cy) * factor,
            cz + (p[2] - cz) * factor,
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Jitter vertices by random displacement within a sphere.
pub fn jitter_vertices(input: &PolyData, amount: f64, seed: u64) -> PolyData {
    let n = input.points.len();
    let mut rng = seed;
    let mut points = Points::<f64>::new();

    let next = |rng: &mut u64| -> f64 {
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*rng >> 33) as f64 / (1u64 << 31) as f64 * 2.0 - 1.0
    };

    for i in 0..n {
        let p = input.points.get(i);
        let dx = next(&mut rng) * amount;
        let dy = next(&mut rng) * amount;
        let dz = next(&mut rng) * amount;
        points.push([p[0] + dx, p[1] + dy, p[2] + dz]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Shrink mesh toward centroid. Convenience for scatter_from_centroid(0.9).
pub fn shrink_mesh(input: &PolyData, factor: f64) -> PolyData {
    scatter_from_centroid(input, 1.0 - factor.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);

        let result = scatter_from_centroid(&pd, 2.0);
        // Center is at 1.0, so 0->-1 and 2->3
        let p0 = result.points.get(0);
        assert!((p0[0] + 1.0).abs() < 1e-10);
    }

    #[test]
    fn contract() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);

        let result = scatter_from_centroid(&pd, 0.5);
        let p0 = result.points.get(0);
        let p1 = result.points.get(1);
        assert!(p1[0] - p0[0] < 2.0); // closer together
    }

    #[test]
    fn jitter_changes() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = jitter_vertices(&pd, 0.1, 42);
        // Points should be slightly different
        let p = result.points.get(0);
        assert!(p[0] != 0.0 || p[1] != 0.0 || p[2] != 0.0);
    }

    #[test]
    fn shrink_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);

        let result = shrink_mesh(&pd, 0.5);
        let d = (result.points.get(1)[0] - result.points.get(0)[0]).abs();
        assert!(d < 10.0); // shrunk
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(scatter_from_centroid(&pd, 2.0).points.len(), 0);
    }
}
