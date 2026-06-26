//! Interpolated velocity field abstraction for flow visualization.
//!
//! Provides a trait and implementations for evaluating velocity at
//! arbitrary points in a vector field, used by stream tracers and
//! particle tracers.

use crate::data::{AnyDataArray, ImageData};

/// Trait for evaluating a velocity field at a point.
pub trait VelocityField {
    /// Evaluate the velocity at a 3D point. Returns None if out of bounds.
    fn evaluate(&self, point: [f64; 3]) -> Option<[f64; 3]>;

    /// Check if a point is within the field's domain.
    fn contains(&self, point: [f64; 3]) -> bool {
        self.evaluate(point).is_some()
    }
}

/// Velocity field from a single ImageData with trilinear interpolation.
pub struct ImageVelocityField<'a> {
    field: &'a ImageData,
}

impl<'a> ImageVelocityField<'a> {
    pub fn new(field: &'a ImageData) -> Self {
        Self { field }
    }
}

impl<'a> VelocityField for ImageVelocityField<'a> {
    fn evaluate(&self, point: [f64; 3]) -> Option<[f64; 3]> {
        let vectors = self.field.point_data().vectors()?;
        if vectors.num_components() != 3 {
            return None;
        }
        let dims = self.field.dimensions();
        let spacing = self.field.spacing();
        let origin = self.field.origin();

        if dims.iter().any(|&d| d == 0) || spacing.iter().any(|&s| s == 0.0) {
            return None;
        }

        for i in 0..3 {
            let end = origin[i] + (dims[i] as f64 - 1.0) * spacing[i];
            let lo = origin[i].min(end);
            let hi = origin[i].max(end);
            if point[i] < lo || point[i] > hi {
                return None;
            }
        }

        Some(trilinear_interp(vectors, point, origin, spacing, dims))
    }
}

/// Composite velocity field from multiple ImageData blocks.
///
/// Tries each block in order until one contains the query point.
pub struct CompositeVelocityField<'a> {
    fields: Vec<ImageVelocityField<'a>>,
}

impl<'a> CompositeVelocityField<'a> {
    pub fn new(fields: Vec<&'a ImageData>) -> Self {
        Self {
            fields: fields.into_iter().map(ImageVelocityField::new).collect(),
        }
    }

    pub fn num_blocks(&self) -> usize {
        self.fields.len()
    }
}

impl<'a> VelocityField for CompositeVelocityField<'a> {
    fn evaluate(&self, point: [f64; 3]) -> Option<[f64; 3]> {
        for field in &self.fields {
            if let Some(v) = field.evaluate(point) {
                return Some(v);
            }
        }
        None
    }
}

/// Integrate a streamline through any VelocityField using RK4.
pub fn integrate_streamline(
    field: &dyn VelocityField,
    seed: [f64; 3],
    step_size: f64,
    max_steps: usize,
    direction: f64,
) -> Vec<[f64; 3]> {
    let mut points = Vec::new();
    let mut pos = seed;
    let dt = step_size * direction;

    for _ in 0..max_steps {
        let k1 = match field.evaluate(pos) {
            Some(v) => v,
            None => break,
        };
        let speed = (k1[0] * k1[0] + k1[1] * k1[1] + k1[2] * k1[2]).sqrt();
        if speed < 1e-10 {
            break;
        }

        points.push(pos);

        let p2 = [
            pos[0] + 0.5 * dt * k1[0],
            pos[1] + 0.5 * dt * k1[1],
            pos[2] + 0.5 * dt * k1[2],
        ];
        let k2 = field.evaluate(p2).unwrap_or(k1);
        let p3 = [
            pos[0] + 0.5 * dt * k2[0],
            pos[1] + 0.5 * dt * k2[1],
            pos[2] + 0.5 * dt * k2[2],
        ];
        let k3 = field.evaluate(p3).unwrap_or(k2);
        let p4 = [
            pos[0] + dt * k3[0],
            pos[1] + dt * k3[1],
            pos[2] + dt * k3[2],
        ];
        let k4 = field.evaluate(p4).unwrap_or(k3);

        pos = [
            pos[0] + dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
            pos[1] + dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
            pos[2] + dt / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
        ];
    }

    points
}

fn trilinear_interp(
    arr: &AnyDataArray,
    pos: [f64; 3],
    origin: [f64; 3],
    spacing: [f64; 3],
    dims: [usize; 3],
) -> [f64; 3] {
    let fx = (pos[0] - origin[0]) / spacing[0];
    let fy = (pos[1] - origin[1]) / spacing[1];
    let fz = (pos[2] - origin[2]) / spacing[2];
    let ix = if dims[0] > 1 {
        (fx.floor() as usize).min(dims[0] - 2)
    } else {
        0
    };
    let iy = if dims[1] > 1 {
        (fy.floor() as usize).min(dims[1] - 2)
    } else {
        0
    };
    let iz = if dims[2] > 1 {
        (fz.floor() as usize).min(dims[2] - 2)
    } else {
        0
    };
    let tx = if dims[0] > 1 {
        (fx - ix as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let ty = if dims[1] > 1 {
        (fy - iy as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let tz = if dims[2] > 1 {
        (fz - iz as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut r = [0.0; 3];
    let mut buf = [0.0f64; 3];
    let nz = if dims[2] > 1 { 2 } else { 1 };
    let ny = if dims[1] > 1 { 2 } else { 1 };
    let nx = if dims[0] > 1 { 2 } else { 1 };
    for dz in 0..nz {
        for dy in 0..ny {
            for dx in 0..nx {
                let idx = (ix + dx) + (iy + dy) * dims[0] + (iz + dz) * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    let w = (if dx == 0 { 1.0 - tx } else { tx })
                        * (if dy == 0 { 1.0 - ty } else { ty })
                        * (if dz == 0 { 1.0 - tz } else { tz });
                    for c in 0..3 {
                        r[c] += w * buf[c];
                    }
                }
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    fn make_field() -> ImageData {
        let dims = [10, 10, 10];
        let n = dims[0] * dims[1] * dims[2];
        let mut v = Vec::with_capacity(n * 3);
        for _ in 0..n {
            v.push(1.0);
            v.push(0.5);
            v.push(0.0);
        }
        let mut f = ImageData::with_dimensions(dims[0], dims[1], dims[2]);
        f.set_spacing([1.0, 1.0, 1.0]);
        f.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vel", v, 3)));
        f.point_data_mut().set_active_vectors("vel");
        f
    }

    #[test]
    fn single_field() {
        let field = make_field();
        let vf = ImageVelocityField::new(&field);
        let v = vf.evaluate([5.0, 5.0, 5.0]).unwrap();
        assert!((v[0] - 1.0).abs() < 1e-10);
        assert!(vf.evaluate([20.0, 5.0, 5.0]).is_none()); // out of bounds
    }

    #[test]
    fn composite() {
        let f1 = make_field();
        let mut f2 = make_field();
        f2.set_origin([10.0, 0.0, 0.0]);
        let cvf = CompositeVelocityField::new(vec![&f1, &f2]);
        assert!(cvf.evaluate([5.0, 5.0, 5.0]).is_some());
        assert!(cvf.evaluate([15.0, 5.0, 5.0]).is_some());
        assert!(cvf.evaluate([25.0, 5.0, 5.0]).is_none());
    }

    #[test]
    fn negative_spacing_bounds() {
        let mut field = make_field();
        field.set_spacing([-1.0, 1.0, 1.0]);
        let vf = ImageVelocityField::new(&field);
        assert!(vf.evaluate([-5.0, 5.0, 5.0]).is_some());
        assert!(vf.evaluate([5.0, 5.0, 5.0]).is_none());
    }

    #[test]
    fn streamline_integration() {
        let field = make_field();
        let vf = ImageVelocityField::new(&field);
        let line = integrate_streamline(&vf, [2.0, 5.0, 5.0], 0.1, 30, 1.0);
        assert!(line.len() > 5);
        // Should move in +X direction
        assert!(line.last().unwrap()[0] > 2.5);
    }
}
