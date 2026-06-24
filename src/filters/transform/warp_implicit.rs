use crate::data::PolyData;
use crate::types::ImplicitFunction;

/// Warp points along the gradient of an implicit function.
///
/// Each point is displaced along the gradient of `func` by the function
/// value times `scale_factor`. This pushes points toward (or away from)
/// the zero-level set of the implicit function.
pub fn warp_by_implicit<F: ImplicitFunction>(
    input: &PolyData,
    func: &F,
    scale_factor: f64,
) -> PolyData {
    let mut pd = input.clone();

    for i in 0..pd.points.len() {
        let p = pd.points.get(i);
        let val = func.evaluate(p[0], p[1], p[2]);
        let grad = func.gradient(p[0], p[1], p[2]);
        let glen = (grad[0] * grad[0] + grad[1] * grad[1] + grad[2] * grad[2]).sqrt();

        if glen > 1e-20 {
            let factor = val * scale_factor / glen;
            pd.points.set(
                i,
                [
                    p[0] - factor * grad[0],
                    p[1] - factor * grad[1],
                    p[2] - factor * grad[2],
                ],
            );
        }
    }

    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ImplicitSphere;

    #[test]
    fn warp_toward_sphere() {
        let mut pd = PolyData::new();
        pd.points.push([2.0, 0.0, 0.0]); // outside unit sphere
        pd.points.push([0.5, 0.0, 0.0]); // inside unit sphere

        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let result = warp_by_implicit(&pd, &sphere, 0.1);

        // Outside point should move inward
        let p0 = result.points.get(0);
        assert!(p0[0] < 2.0, "should move inward, got x={}", p0[0]);

        // Inside point should move outward (negative value * negative direction)
        let p1 = result.points.get(1);
        assert!(p1[0] > 0.5, "should move outward, got x={}", p1[0]);
    }

    #[test]
    fn zero_scale_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let result = warp_by_implicit(&pd, &sphere, 0.0);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }
}
