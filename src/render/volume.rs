use crate::render::ColorMap;

/// Transfer function mapping scalar values to color and opacity.
#[derive(Debug, Clone)]
pub struct TransferFunction {
    /// Color map for scalar-to-RGB mapping.
    pub color_map: ColorMap,
    /// Opacity control points: (scalar_value_normalized, opacity).
    /// Normalized to [0,1] range relative to the scalar range.
    opacity_points: Vec<(f64, f64)>,
}

impl TransferFunction {
    /// Create a transfer function with a color map and linear opacity ramp.
    pub fn linear(color_map: ColorMap) -> Self {
        Self {
            color_map,
            opacity_points: vec![(0.0, 0.0), (1.0, 1.0)],
        }
    }

    /// Create a transfer function with constant opacity.
    pub fn constant_opacity(color_map: ColorMap, opacity: f64) -> Self {
        Self {
            color_map,
            opacity_points: vec![(0.0, opacity), (1.0, opacity)],
        }
    }

    /// Create a transfer function with a Gaussian opacity peak.
    pub fn gaussian(color_map: ColorMap, center: f64, width: f64, peak_opacity: f64) -> Self {
        let mut points = Vec::new();
        let n = 64;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let d = (t - center) / width;
            let opacity = peak_opacity * (-0.5 * d * d).exp();
            points.push((t, opacity));
        }
        Self {
            color_map,
            opacity_points: points,
        }
    }

    /// Set custom opacity control points.
    pub fn set_opacity_points(&mut self, points: Vec<(f64, f64)>) {
        self.opacity_points = points;
    }

    /// Sample the transfer function at a normalized scalar value t in [0,1].
    /// Returns (r, g, b, a).
    pub fn sample(&self, t: f64) -> [f32; 4] {
        let color = self.color_map.map(t);
        let opacity = self.sample_opacity(t);
        [color[0], color[1], color[2], opacity as f32]
    }

    fn sample_opacity(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        if self.opacity_points.is_empty() {
            return 1.0;
        }
        if t <= self.opacity_points[0].0 {
            return self.opacity_points[0].1;
        }
        if t >= self.opacity_points.last().unwrap().0 {
            return self.opacity_points.last().unwrap().1;
        }
        for i in 0..self.opacity_points.len() - 1 {
            let (t0, o0) = self.opacity_points[i];
            let (t1, o1) = self.opacity_points[i + 1];
            if t >= t0 && t <= t1 {
                let frac = if (t1 - t0).abs() > 1e-10 {
                    (t - t0) / (t1 - t0)
                } else {
                    0.0
                };
                return o0 + frac * (o1 - o0);
            }
        }
        1.0
    }

    /// Generate a 1D RGBA lookup table (256 entries) for GPU use.
    pub fn to_lut_rgba(&self) -> Vec<u8> {
        let mut lut = Vec::with_capacity(256 * 4);
        for i in 0..256 {
            let t = i as f64 / 255.0;
            let rgba = self.sample(t);
            lut.push((rgba[0] * 255.0) as u8);
            lut.push((rgba[1] * 255.0) as u8);
            lut.push((rgba[2] * 255.0) as u8);
            lut.push((rgba[3] * 255.0) as u8);
        }
        lut
    }
}

/// A volume rendering actor for ImageData.
///
/// Renders volumetric data using ray casting with a transfer function
/// to map scalar values to color and opacity.
#[derive(Debug, Clone)]
pub struct VolumeActor {
    /// Scalar data for the volume (one value per voxel, x varies fastest).
    pub scalars: Vec<f64>,
    /// Volume dimensions [nx, ny, nz].
    pub dimensions: [usize; 3],
    /// Volume origin.
    pub origin: [f64; 3],
    /// Voxel spacing.
    pub spacing: [f64; 3],
    /// Scalar range [min, max].
    pub scalar_range: [f64; 2],
    /// Transfer function.
    pub transfer_function: TransferFunction,
    /// Number of ray marching steps. Default: 256
    pub num_steps: usize,
    /// Global opacity multiplier. Default: 1.0
    pub opacity_scale: f64,
}

impl VolumeActor {
    /// Create a volume actor from ImageData scalars.
    pub fn new(
        scalars: Vec<f64>,
        dimensions: [usize; 3],
        origin: [f64; 3],
        spacing: [f64; 3],
        transfer_function: TransferFunction,
    ) -> Self {
        let (mut smin, mut smax) = (f64::INFINITY, f64::NEG_INFINITY);
        for &v in &scalars {
            smin = smin.min(v);
            smax = smax.max(v);
        }
        Self {
            scalars,
            dimensions,
            origin,
            spacing,
            scalar_range: [smin, smax],
            transfer_function,
            num_steps: 256,
            opacity_scale: 1.0,
        }
    }

    /// Sample the volume at a world-space position using trilinear interpolation.
    /// Returns None if the position is outside the volume.
    pub fn sample(&self, pos: [f64; 3]) -> Option<f64> {
        let [nx, ny, nz] = self.dimensions;
        let fi = (pos[0] - self.origin[0]) / self.spacing[0];
        let fj = (pos[1] - self.origin[1]) / self.spacing[1];
        let fk = (pos[2] - self.origin[2]) / self.spacing[2];

        if fi < 0.0 || fj < 0.0 || fk < 0.0 {
            return None;
        }

        let i0 = fi as usize;
        let j0 = fj as usize;
        let k0 = fk as usize;

        if i0 + 1 >= nx || j0 + 1 >= ny || k0 + 1 >= nz {
            return None;
        }

        let u = fi - i0 as f64;
        let v = fj - j0 as f64;
        let w = fk - k0 as f64;

        let idx = |i: usize, j: usize, k: usize| k * nx * ny + j * nx + i;

        let c000 = self.scalars[idx(i0, j0, k0)];
        let c100 = self.scalars[idx(i0 + 1, j0, k0)];
        let c010 = self.scalars[idx(i0, j0 + 1, k0)];
        let c110 = self.scalars[idx(i0 + 1, j0 + 1, k0)];
        let c001 = self.scalars[idx(i0, j0, k0 + 1)];
        let c101 = self.scalars[idx(i0 + 1, j0, k0 + 1)];
        let c011 = self.scalars[idx(i0, j0 + 1, k0 + 1)];
        let c111 = self.scalars[idx(i0 + 1, j0 + 1, k0 + 1)];

        let c00 = c000 * (1.0 - u) + c100 * u;
        let c10 = c010 * (1.0 - u) + c110 * u;
        let c01 = c001 * (1.0 - u) + c101 * u;
        let c11 = c011 * (1.0 - u) + c111 * u;

        let c0 = c00 * (1.0 - v) + c10 * v;
        let c1 = c01 * (1.0 - v) + c11 * v;

        Some(c0 * (1.0 - w) + c1 * w)
    }

    /// CPU-side ray casting: cast a ray through the volume and composite.
    ///
    /// Returns the front-to-back composited RGBA color.
    pub fn cast_ray(&self, origin: [f64; 3], direction: [f64; 3]) -> [f32; 4] {
        let [nx, ny, nz] = self.dimensions;
        let bound_max = [
            self.origin[0] + (nx - 1) as f64 * self.spacing[0],
            self.origin[1] + (ny - 1) as f64 * self.spacing[1],
            self.origin[2] + (nz - 1) as f64 * self.spacing[2],
        ];

        // Find ray-AABB intersection
        let (t_enter, t_exit) = match ray_aabb(origin, direction, self.origin, bound_max) {
            Some(t) => t,
            None => return [0.0, 0.0, 0.0, 0.0],
        };

        let step_size = (t_exit - t_enter) / self.num_steps as f64;
        let srange = self.scalar_range[1] - self.scalar_range[0];

        let mut color = [0.0f64; 3];
        let mut alpha = 0.0f64;

        for step in 0..self.num_steps {
            if alpha >= 0.99 {
                break;
            }
            let t = t_enter + (step as f64 + 0.5) * step_size;
            let pos = [
                origin[0] + t * direction[0],
                origin[1] + t * direction[1],
                origin[2] + t * direction[2],
            ];
            if let Some(scalar) = self.sample(pos) {
                let normalized = if srange.abs() > 1e-20 {
                    (scalar - self.scalar_range[0]) / srange
                } else {
                    0.5
                };
                let rgba = self.transfer_function.sample(normalized);
                let sample_alpha = rgba[3] as f64 * self.opacity_scale * step_size * 100.0;
                let sample_alpha = sample_alpha.clamp(0.0, 1.0);

                // Front-to-back compositing
                color[0] += (1.0 - alpha) * sample_alpha * rgba[0] as f64;
                color[1] += (1.0 - alpha) * sample_alpha * rgba[1] as f64;
                color[2] += (1.0 - alpha) * sample_alpha * rgba[2] as f64;
                alpha += (1.0 - alpha) * sample_alpha;
            }
        }

        [
            color[0] as f32,
            color[1] as f32,
            color[2] as f32,
            alpha as f32,
        ]
    }
}

fn ray_aabb(
    origin: [f64; 3],
    dir: [f64; 3],
    aabb_min: [f64; 3],
    aabb_max: [f64; 3],
) -> Option<(f64, f64)> {
    let mut tmin = f64::NEG_INFINITY;
    let mut tmax = f64::INFINITY;
    for i in 0..3 {
        if dir[i].abs() < 1e-15 {
            if origin[i] < aabb_min[i] || origin[i] > aabb_max[i] {
                return None;
            }
        } else {
            let inv = 1.0 / dir[i];
            let mut t1 = (aabb_min[i] - origin[i]) * inv;
            let mut t2 = (aabb_max[i] - origin[i]) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
        }
    }
    if tmin > tmax || tmax < 0.0 {
        None
    } else {
        Some((tmin.max(0.0), tmax))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_volume() -> VolumeActor {
        // 4x4x4 volume with a sphere
        let dims = [4, 4, 4];
        let mut scalars = Vec::new();
        for k in 0..4 {
            for j in 0..4 {
                for i in 0..4 {
                    let x = i as f64 / 3.0 - 0.5;
                    let y = j as f64 / 3.0 - 0.5;
                    let z = k as f64 / 3.0 - 0.5;
                    scalars.push(1.0 - (x * x + y * y + z * z).sqrt());
                }
            }
        }
        let tf = TransferFunction::linear(ColorMap::jet());
        VolumeActor::new(
            scalars,
            dims,
            [0.0, 0.0, 0.0],
            [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            tf,
        )
    }

    #[test]
    fn transfer_function_linear() {
        let tf = TransferFunction::linear(ColorMap::jet());
        let rgba = tf.sample(0.5);
        assert!(rgba[3] > 0.0); // non-zero opacity
    }

    #[test]
    fn transfer_function_constant() {
        let tf = TransferFunction::constant_opacity(ColorMap::jet(), 0.3);
        let rgba = tf.sample(0.5);
        assert!((rgba[3] - 0.3).abs() < 0.01);
    }

    #[test]
    fn transfer_function_lut() {
        let tf = TransferFunction::linear(ColorMap::jet());
        let lut = tf.to_lut_rgba();
        assert_eq!(lut.len(), 256 * 4);
    }

    #[test]
    fn volume_sample() {
        let vol = make_test_volume();
        // Sample at center
        let val = vol.sample([0.5, 0.5, 0.5]);
        assert!(val.is_some());
    }

    #[test]
    fn volume_sample_outside() {
        let vol = make_test_volume();
        assert!(vol.sample([10.0, 10.0, 10.0]).is_none());
    }

    #[test]
    fn volume_ray_cast() {
        let vol = make_test_volume();
        let rgba = vol.cast_ray([0.5, 0.5, -1.0], [0.0, 0.0, 1.0]);
        // Ray passes through volume — should have some opacity
        assert!(rgba[3] > 0.0);
    }

    #[test]
    fn volume_ray_miss() {
        let vol = make_test_volume();
        let rgba = vol.cast_ray([10.0, 10.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(rgba[3], 0.0);
    }

    #[test]
    fn transfer_function_gaussian() {
        let tf = TransferFunction::gaussian(ColorMap::jet(), 0.5, 0.1, 0.8);
        let at_center = tf.sample(0.5);
        let at_edge = tf.sample(0.0);
        assert!(at_center[3] > at_edge[3]);
    }
}
