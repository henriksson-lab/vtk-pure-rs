//! DensifyPointCloudFilter - add interpolated points between distant neighbors.

use crate::data::{AnyDataArray, Points, PolyData};

const MAXIMUM_NUMBER_OF_ITERATIONS: usize = 3;

/// Repeatedly bisect point pairs in a radius neighborhood that are farther
/// apart than `target_distance`, following vtkDensifyPointCloudFilter's
/// radius-neighborhood path.
pub fn densify_point_cloud(input: &PolyData, radius: f64, target_distance: f64) -> PolyData {
    let num_pts = input.points.len();
    let mut new_pts = Points::<f64>::new();

    // Copy original points
    for i in 0..num_pts {
        new_pts.push(input.points.get(i));
    }
    let mut point_arrays: Vec<AnyDataArray> = input.point_data().iter().cloned().collect();

    let radius2 = radius * radius;
    let target_distance2 = target_distance * target_distance;

    for _ in 0..MAXIMUM_NUMBER_OF_ITERATIONS {
        let num_in_pts = new_pts.len();
        let pts: Vec<[f64; 3]> = (0..num_in_pts).map(|i| new_pts.get(i)).collect();
        let mut generated = Vec::new();

        for point_id in 0..num_in_pts {
            let px = pts[point_id];
            for (id, py) in pts.iter().enumerate() {
                if id <= point_id {
                    continue;
                }

                let dx = px[0] - py[0];
                let dy = px[1] - py[1];
                let dz = px[2] - py[2];
                let d2 = dx * dx + dy * dy + dz * dz;

                if d2 <= radius2 && d2 >= target_distance2 {
                    generated.push((
                        point_id,
                        id,
                        [
                            (px[0] + py[0]) * 0.5,
                            (px[1] + py[1]) * 0.5,
                            (px[2] + py[2]) * 0.5,
                        ],
                    ));
                }
            }
        }

        if generated.is_empty() {
            break;
        }

        for array in &mut point_arrays {
            append_interpolated_tuples(array, &generated);
        }
        for (_, _, point) in generated {
            new_pts.push(point);
        }
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    for array in point_arrays {
        result.point_data_mut().add_array(array);
    }
    result
}

fn append_interpolated_tuples(array: &mut AnyDataArray, edges: &[(usize, usize, [f64; 3])]) {
    macro_rules! append {
        ($arr:expr) => {{
            for &(a, b, _) in edges {
                let ta = $arr.tuple(a).to_vec();
                let tb = $arr.tuple(b).to_vec();
                let values: Vec<_> = ta
                    .iter()
                    .zip(tb.iter())
                    .map(|(&va, &vb)| average_scalar(va, vb))
                    .collect();
                $arr.push_tuple(&values);
            }
        }};
    }

    match array {
        AnyDataArray::F32(a) => append!(a),
        AnyDataArray::F64(a) => append!(a),
        AnyDataArray::I8(a) => append!(a),
        AnyDataArray::I16(a) => append!(a),
        AnyDataArray::I32(a) => append!(a),
        AnyDataArray::I64(a) => append!(a),
        AnyDataArray::U8(a) => append!(a),
        AnyDataArray::U16(a) => append!(a),
        AnyDataArray::U32(a) => append!(a),
        AnyDataArray::U64(a) => append!(a),
    }
}

trait AverageScalar: Copy {
    fn average(a: Self, b: Self) -> Self;
}

macro_rules! impl_average_float {
    ($($ty:ty),* $(,)?) => {
        $(
            impl AverageScalar for $ty {
                fn average(a: Self, b: Self) -> Self {
                    (a + b) * 0.5
                }
            }
        )*
    };
}

macro_rules! impl_average_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl AverageScalar for $ty {
                fn average(a: Self, b: Self) -> Self {
                    ((a as i128 + b as i128) / 2) as Self
                }
            }
        )*
    };
}

macro_rules! impl_average_uint {
    ($($ty:ty),* $(,)?) => {
        $(
            impl AverageScalar for $ty {
                fn average(a: Self, b: Self) -> Self {
                    ((a as u128 + b as u128) / 2) as Self
                }
            }
        )*
    };
}

impl_average_float!(f32, f64);
impl_average_int!(i8, i16, i32, i64);
impl_average_uint!(u8, u16, u32, u64);

fn average_scalar<T: AverageScalar>(a: T, b: T) -> T {
    T::average(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_midpoints() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);

        // radius=3, target_distance=1: the pair is within radius and above target distance
        let result = densify_point_cloud(&pd, 3.0, 1.0);
        assert!(result.points.len() > 2);
        let mid = result.points.get(2);
        assert!((mid[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn no_midpoint_for_close_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);

        // target_distance=1: pair is too close
        let result = densify_point_cloud(&pd, 10.0, 1.0);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn iterates_until_target_distance() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([4.0, 0.0, 0.0]);

        let result = densify_point_cloud(&pd, 5.0, 1.0);
        assert!(result.points.len() > 2);
    }
}
