use crate::data::{Points, PolyData};

/// Morph between two meshes with the same topology.
///
/// Linearly interpolates vertex positions at parameter t (0=source, 1=target).
/// Also supports non-linear easing via `ease` parameter.
pub fn mesh_morph(source: &PolyData, target: &PolyData, t: f64, ease: MorphEasing) -> PolyData {
    let n = source.points.len();
    if n == 0 || n != target.points.len() {
        return source.clone();
    }

    let t = t.clamp(0.0, 1.0);
    let s = match ease {
        MorphEasing::Linear => t,
        MorphEasing::SmoothStep => t * t * (3.0 - 2.0 * t),
        MorphEasing::EaseIn => t * t,
        MorphEasing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
    };

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let a = source.points.get(i);
        let b = target.points.get(i);
        points.push([
            a[0] + s * (b[0] - a[0]),
            a[1] + s * (b[1] - a[1]),
            a[2] + s * (b[2] - a[2]),
        ]);
    }

    let mut pd = source.clone();
    pd.points = points;
    pd
}

/// Easing function for morphing.
#[derive(Debug, Clone, Copy)]
pub enum MorphEasing {
    Linear,
    SmoothStep,
    EaseIn,
    EaseOut,
}

/// Generate a sequence of morph frames.
pub fn mesh_morph_sequence(
    source: &PolyData,
    target: &PolyData,
    num_frames: usize,
    ease: MorphEasing,
) -> Vec<PolyData> {
    (0..=num_frames)
        .map(|i| {
            let t = i as f64 / num_frames.max(1) as f64;
            mesh_morph(source, target, t, ease)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_midpoint() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([2.0, 0.0, 0.0]);
        let result = mesh_morph(&a, &b, 0.5, MorphEasing::Linear);
        assert_eq!(result.points.get(0), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn morph_t0_is_source() {
        let mut a = PolyData::new();
        a.points.push([1.0, 2.0, 3.0]);
        let mut b = PolyData::new();
        b.points.push([4.0, 5.0, 6.0]);
        let result = mesh_morph(&a, &b, 0.0, MorphEasing::Linear);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn morph_t1_is_target() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([10.0, 20.0, 30.0]);
        let result = mesh_morph(&a, &b, 1.0, MorphEasing::SmoothStep);
        let p = result.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn sequence_correct_count() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]);
        let seq = mesh_morph_sequence(&a, &b, 4, MorphEasing::Linear);
        assert_eq!(seq.len(), 5); // 0,1,2,3,4
    }

    #[test]
    fn mismatched_sizes() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);
        let result = mesh_morph(&a, &b, 0.5, MorphEasing::Linear);
        assert_eq!(result.points.len(), 1); // falls back to source
    }

    #[test]
    fn empty_input() {
        let a = PolyData::new();
        let b = PolyData::new();
        assert_eq!(mesh_morph(&a, &b, 0.5, MorphEasing::Linear).points.len(), 0);
    }
}
