//! Lightweight 3D math utilities.
//!
//! VTK origin: selected audited functions from `VTK/Common/Core/vtkMath.cxx`.

use std::sync::{Mutex, OnceLock};

use super::{
    any_array::AnyArray, minimal_standard_random_sequence::MinimalStandardRandomSequence,
    random_sequence::RandomSequence, vtk_type::VtkDataType,
};

const VTK_SMALL_NUMBER: f64 = 1.0e-12;
const VTK_FLOAT_MAX: f64 = f32::MAX as f64;
const VTK_MAX_ROTATIONS: i32 = 20;

static MATH_UNIFORM_RANDOM_SEQUENCE: OnceLock<Mutex<MinimalStandardRandomSequence>> =
    OnceLock::new();

fn math_uniform_random_sequence() -> &'static Mutex<MinimalStandardRandomSequence> {
    MATH_UNIFORM_RANDOM_SEQUENCE.get_or_init(|| {
        let mut sequence = MinimalStandardRandomSequence::new();
        sequence.set_seed_only(1177);
        Mutex::new(sequence)
    })
}

/// VTK: `vtkMath::Pi`.
pub fn pi() -> f64 {
    std::f64::consts::PI
}

/// VTK: `vtkMath::DYNAMIC_VECTOR_SIZE`.
pub fn dynamic_vector_size() -> i32 {
    0
}

/// VTK: `vtkMath::Random`.
pub fn random() -> f64 {
    let mut sequence = math_uniform_random_sequence()
        .lock()
        .expect("vtkMath uniform random sequence mutex poisoned");
    sequence.next();
    sequence.get_value()
}

/// VTK: `vtkMath::Random`.
pub fn random_range(min: f64, max: f64) -> f64 {
    let mut sequence = math_uniform_random_sequence()
        .lock()
        .expect("vtkMath uniform random sequence mutex poisoned");
    sequence.next();
    sequence.get_range_value(min, max)
}

/// VTK: `vtkMath::RandomSeed`.
pub fn random_seed(seed: i32) {
    math_uniform_random_sequence()
        .lock()
        .expect("vtkMath uniform random sequence mutex poisoned")
        .set_seed(seed);
}

/// VTK: `vtkMath::GetSeed`.
pub fn get_seed() -> i32 {
    math_uniform_random_sequence()
        .lock()
        .expect("vtkMath uniform random sequence mutex poisoned")
        .get_seed()
}

/// VTK: `vtkMath::RadiansFromDegrees`.
pub fn radians_from_degrees(degrees: f64) -> f64 {
    degrees * 0.017453292519943295
}

/// VTK: `vtkMath::DegreesFromRadians`.
pub fn degrees_from_radians(radians: f64) -> f64 {
    radians * 57.29577951308232
}

/// VTK: `vtkMath::Round`.
pub fn round(f: f64) -> i32 {
    (f + if f >= 0.0 { 0.5 } else { -0.5 }) as i32
}

/// VTK: `vtkMath::Dot`.
pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// VTK: `vtkMath::Dot2D`.
pub fn dot2d(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// VTK: `vtkMath::Outer`.
pub fn outer(a: [f64; 3], b: [f64; 3]) -> [[f64; 3]; 3] {
    [
        [a[0] * b[0], a[0] * b[1], a[0] * b[2]],
        [a[1] * b[0], a[1] * b[1], a[1] * b[2]],
        [a[2] * b[0], a[2] * b[1], a[2] * b[2]],
    ]
}

/// VTK: `vtkMath::Outer2D`.
pub fn outer2d(x: [f64; 2], y: [f64; 2]) -> [[f64; 2]; 2] {
    [[x[0] * y[0], x[0] * y[1]], [x[1] * y[0], x[1] * y[1]]]
}

/// VTK: `vtkMath::Cross`.
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// VTK: `vtkMath::SquaredNorm`.
pub fn squared_norm(v: [f64; 3]) -> f64 {
    dot(v, v)
}

/// VTK: `vtkMath::Norm`.
pub fn norm(v: &[f64]) -> f64 {
    norm2(v).sqrt()
}

fn norm2(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum()
}

/// VTK: `vtkMath::Norm2D`.
pub fn norm2d(v: [f64; 2]) -> f64 {
    dot2d(v, v).sqrt()
}

/// VTK: `vtkMath::Normalize2D`.
pub fn normalize2d(v: &mut [f64; 2]) -> f64 {
    let den = norm2d(*v);
    if den != 0.0 {
        for value in v {
            *value /= den;
        }
    }
    den
}

/// VTK: `vtkMath::Normalize`.
pub fn normalize(v: &mut [f64; 3]) -> f64 {
    let den = length(*v);
    if den != 0.0 {
        for value in v {
            *value /= den;
        }
    }
    den
}

/// VTK: `vtkMath::Perpendiculars`.
pub fn perpendiculars(v1: [f64; 3], theta: f64) -> ([f64; 3], [f64; 3]) {
    let v1sq = v1[0] * v1[0];
    let v2sq = v1[1] * v1[1];
    let v3sq = v1[2] * v1[2];
    let r = (v1sq + v2sq + v3sq).sqrt();

    let (dv1, dv2, dv3) = if v1sq > v2sq && v1sq > v3sq {
        (0, 1, 2)
    } else if v2sq > v3sq {
        (1, 2, 0)
    } else {
        (2, 0, 1)
    };

    let a = v1[dv1] / r;
    let b = v1[dv2] / r;
    let c = v1[dv3] / r;
    let tmp = (a * a + c * c).sqrt();

    let mut v2 = [0.0; 3];
    let mut v3 = [0.0; 3];
    if theta != 0.0 {
        let sintheta = theta.sin();
        let costheta = theta.cos();

        v2[dv1] = (c * costheta - a * b * sintheta) / tmp;
        v2[dv2] = sintheta * tmp;
        v2[dv3] = (-a * costheta - b * c * sintheta) / tmp;

        v3[dv1] = (-c * sintheta - a * b * costheta) / tmp;
        v3[dv2] = costheta * tmp;
        v3[dv3] = (a * sintheta - b * c * costheta) / tmp;
    } else {
        v2[dv1] = c / tmp;
        v2[dv2] = 0.0;
        v2[dv3] = -a / tmp;

        v3[dv1] = -a * b / tmp;
        v3[dv2] = tmp;
        v3[dv3] = -b * c / tmp;
    }

    (v2, v3)
}

#[cfg(test)]
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    length(subtract(b, a))
}

/// VTK: `vtkMath::Distance2BetweenPoints`.
pub fn distance2_between_points(a: [f64; 3], b: [f64; 3]) -> f64 {
    squared_norm(subtract(a, b))
}

/// VTK: `vtkMath::Distance2BetweenPoints2D`.
pub fn distance2_between_points2d(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// VTK: `vtkMath::Assign`.
///
/// Returns the output 3-vector that VTK writes through `b`.
pub fn assign(a: [f64; 3]) -> [f64; 3] {
    [a[0], a[1], a[2]]
}

/// VTK: `vtkMath::Add`.
///
/// Returns the output 3-vector that VTK writes through `c`.
pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// VTK: `vtkMath::Subtract`.
///
/// Returns the output 3-vector that VTK writes through `c`.
pub fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// VTK: `vtkMath::MultiplyScalar`.
pub fn multiply_scalar(a: &mut [f64; 3], s: f64) {
    for value in a {
        *value *= s;
    }
}

/// VTK: `vtkMath::MultiplyScalar2D`.
pub fn multiply_scalar2d(a: &mut [f64; 2], s: f64) {
    for value in a {
        *value *= s;
    }
}

fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[cfg(test)]
fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

/// VTK: `vtkMath::AngleBetweenVectors`.
pub fn angle_between_vectors(a: [f64; 3], b: [f64; 3]) -> f64 {
    let c = cross(a, b);
    length(c).atan2(dot(a, b))
}

/// VTK: `vtkMath::ProjectVector`.
pub fn project_vector(a: [f64; 3], b: [f64; 3]) -> (bool, [f64; 3]) {
    let b_len2 = dot(b, b);
    if b_len2 == 0.0 {
        return (false, [0.0; 3]);
    }
    (true, scale(b, dot(a, b) / b_len2))
}

/// VTK: `vtkMath::ProjectVector2D`.
pub fn project_vector2d(a: [f64; 2], b: [f64; 2]) -> (bool, [f64; 2]) {
    let b_squared = dot2d(b, b);
    if b_squared == 0.0 {
        return (false, [0.0; 2]);
    }

    let scale = dot2d(a, b) / b_squared;
    (true, [b[0] * scale, b[1] * scale])
}

/// VTK: `vtkMath::SignedAngleBetweenVectors`.
pub fn signed_angle_between_vectors(v1: [f64; 3], v2: [f64; 3], vn: [f64; 3]) -> f64 {
    let c = cross(v1, v2);
    let angle = length(c).atan2(dot(v1, v2));
    if dot(c, vn) >= 0.0 {
        angle
    } else {
        -angle
    }
}

#[cfg(test)]
fn reflect(v: [f64; 3], n: [f64; 3]) -> [f64; 3] {
    let d = 2.0 * dot(v, n);
    subtract(v, scale(n, d))
}

fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.clamp(min, max)
}

/// VTK: `vtkMath::ClampValue`.
pub fn clamp_value(value: f64, min: f64, max: f64) -> f64 {
    clamp(value, min, max)
}

/// VTK: `vtkMath::ClampAndNormalizeValue`.
pub fn clamp_and_normalize_value(value: f64, range: [f64; 2]) -> f64 {
    assert!(range[0] <= range[1], "valid_range");
    if range[0] == range[1] {
        0.0
    } else {
        (clamp_value(value, range[0], range[1]) - range[0]) / (range[1] - range[0])
    }
}

/// VTK: `vtkMath::ClampValues`.
pub fn clamp_values(values: &mut [f64], number_of_values: i32, range: [f64; 2]) {
    let number_of_values =
        usize::try_from(number_of_values).expect("number_of_values must be non-negative");
    assert!(
        number_of_values <= values.len(),
        "number_of_values exceeds values length"
    );
    for value in &mut values[..number_of_values] {
        *value = clamp_value(*value, range[0], range[1]);
    }
}

/// VTK: `vtkMath::ClampValues` copy overload.
pub fn clamp_values_copy(
    values: &[f64],
    number_of_values: i32,
    range: [f64; 2],
    clamped_values: &mut [f64],
) {
    let number_of_values =
        usize::try_from(number_of_values).expect("number_of_values must be non-negative");
    assert!(
        number_of_values <= values.len(),
        "number_of_values exceeds values length"
    );
    assert!(
        number_of_values <= clamped_values.len(),
        "number_of_values exceeds clamped_values length"
    );
    for (input, output) in values[..number_of_values]
        .iter()
        .zip(&mut clamped_values[..number_of_values])
    {
        *output = clamp_value(*input, range[0], range[1]);
    }
}

/// VTK: `vtkMath::GetScalarTypeFittingRange`.
pub fn get_scalar_type_fitting_range(
    mut range_min: f64,
    mut range_max: f64,
    scale: f64,
    shift: f64,
) -> i32 {
    const FLOAT_TYPES: &[(VtkDataType, f64, f64)] = &[
        (VtkDataType::Float, f32::MIN as f64, f32::MAX as f64),
        (VtkDataType::Double, f64::MIN, f64::MAX),
    ];
    const INT_TYPES: &[(VtkDataType, f64, f64)] = &[
        (VtkDataType::Bit, 0.0, 1.0),
        (VtkDataType::Char, i8::MIN as f64, i8::MAX as f64),
        (VtkDataType::SignedChar, i8::MIN as f64, i8::MAX as f64),
        (VtkDataType::UnsignedChar, u8::MIN as f64, u8::MAX as f64),
        (VtkDataType::Short, i16::MIN as f64, i16::MAX as f64),
        (VtkDataType::UnsignedShort, u16::MIN as f64, u16::MAX as f64),
        (VtkDataType::Int, i32::MIN as f64, i32::MAX as f64),
        (VtkDataType::UnsignedInt, u32::MIN as f64, u32::MAX as f64),
        (VtkDataType::Long, i64::MIN as f64, i64::MAX as f64),
        (VtkDataType::UnsignedLong, u64::MIN as f64, u64::MAX as f64),
        (VtkDataType::LongLong, i64::MIN as f64, i64::MAX as f64),
        (
            VtkDataType::UnsignedLongLong,
            u64::MIN as f64,
            u64::MAX as f64,
        ),
    ];

    let range_min_is_int = range_min.fract() == 0.0;
    let range_max_is_int = range_max.fract() == 0.0;
    let scale_is_int = scale.fract() == 0.0;
    let shift_is_int = shift.fract() == 0.0;

    range_min = range_min * scale + shift;
    range_max = range_max * scale + shift;

    if range_min_is_int && range_max_is_int && scale_is_int && shift_is_int {
        for &(data_type, min, max) in INT_TYPES {
            if min <= range_min && range_max <= max {
                return data_type.id();
            }
        }
    }

    for &(data_type, min, max) in FLOAT_TYPES {
        if min <= range_min && range_max <= max {
            return data_type.id();
        }
    }

    -1
}

/// VTK: `vtkMath::GetAdjustedScalarRange`.
pub fn get_adjusted_scalar_range(array: &AnyArray, comp: i32) -> Option<[f64; 2]> {
    if comp < 0 || comp >= array.get_number_of_components() {
        return None;
    }

    let mut range = [0.0; 2];
    if !array.compute_range(&mut range, comp) {
        return None;
    }

    match array.get_data_type() {
        VtkDataType::UnsignedChar => {
            range[0] = u8::MIN as f64;
            range[1] = u8::MAX as f64;
        }
        VtkDataType::UnsignedShort => {
            range[0] = u16::MIN as f64;
            if range[1] <= 4095.0 {
                if range[1] > u8::MAX as f64 {
                    range[1] = 4095.0;
                }
            } else {
                range[1] = u16::MAX as f64;
            }
        }
        _ => return None,
    }

    Some(range)
}

/// VTK: `vtkMath::TensorFromSymmetricTensor`.
pub fn tensor_from_symmetric_tensor(symm_tensor: [f64; 9]) -> [f64; 9] {
    [
        symm_tensor[0],
        symm_tensor[3],
        symm_tensor[5],
        symm_tensor[3],
        symm_tensor[1],
        symm_tensor[4],
        symm_tensor[5],
        symm_tensor[4],
        symm_tensor[2],
    ]
}

/// VTK: `vtkMath::TensorFromSymmetricTensor`.
pub fn tensor_from_symmetric_tensor_in_place(tensor: &mut [f64; 9]) {
    tensor[6] = tensor[5];
    tensor[7] = tensor[4];
    tensor[8] = tensor[2];
    tensor[4] = tensor[1];
    tensor[5] = tensor[7];
    tensor[2] = tensor[6];
    tensor[1] = tensor[3];
}

/// VTK: `vtkMath::Determinant2x2`.
pub fn determinant2x2(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * d - b * c
}

/// VTK: `vtkMath::Determinant3x3(const double c1[3], ...)`.
pub fn determinant3x3_from_columns(c1: [f64; 3], c2: [f64; 3], c3: [f64; 3]) -> f64 {
    c1[0] * c2[1] * c3[2] + c2[0] * c3[1] * c1[2] + c3[0] * c1[1] * c2[2]
        - c1[0] * c3[1] * c2[2]
        - c2[0] * c1[1] * c3[2]
        - c3[0] * c2[1] * c1[2]
}

/// VTK: `vtkMath::Determinant3x3`.
pub fn determinant3x3(matrix: [[f64; 3]; 3]) -> f64 {
    matrix[0][0] * matrix[1][1] * matrix[2][2]
        + matrix[1][0] * matrix[2][1] * matrix[0][2]
        + matrix[2][0] * matrix[0][1] * matrix[1][2]
        - matrix[0][0] * matrix[2][1] * matrix[1][2]
        - matrix[1][0] * matrix[0][1] * matrix[2][2]
        - matrix[2][0] * matrix[1][1] * matrix[0][2]
}

/// VTK: `vtkMath::Determinant3x3(double a1, ..., double c3)`.
#[allow(clippy::too_many_arguments)]
pub fn determinant3x3_from_values(
    a1: f64,
    a2: f64,
    a3: f64,
    b1: f64,
    b2: f64,
    b3: f64,
    c1: f64,
    c2: f64,
    c3: f64,
) -> f64 {
    a1 * b2 * c3 + b1 * c2 * a3 + c1 * a2 * b3 - a1 * c2 * b3 - b1 * a2 * c3 - c1 * b2 * a3
}

fn swap_vectors3<T>(a: &mut [[T; 3]; 3], i: usize, j: usize) {
    a.swap(i, j);
}

/// VTK: `vtkMath::LUFactor3x3`.
pub fn lu_factor3x3(mut a: [[f64; 3]; 3]) -> ([[f64; 3]; 3], [i32; 3]) {
    let mut scale = [0.0; 3];
    for i in 0..3 {
        let mut largest = a[i][0].abs();
        let tmp = a[i][1].abs();
        if tmp > largest {
            largest = tmp;
        }
        let tmp = a[i][2].abs();
        if tmp > largest {
            largest = tmp;
        }
        scale[i] = 1.0 / largest;
    }

    let mut index = [0; 3];

    let mut largest = scale[0] * a[0][0].abs();
    let mut max_i = 0;
    let tmp = scale[1] * a[1][0].abs();
    if tmp >= largest {
        largest = tmp;
        max_i = 1;
    }
    let tmp = scale[2] * a[2][0].abs();
    if tmp >= largest {
        max_i = 2;
    }
    if max_i != 0 {
        swap_vectors3(&mut a, max_i, 0);
        scale[max_i] = scale[0];
    }
    index[0] = max_i as i32;

    a[1][0] /= a[0][0];
    a[2][0] /= a[0][0];

    a[1][1] -= a[1][0] * a[0][1];
    a[2][1] -= a[2][0] * a[0][1];
    largest = scale[1] * a[1][1].abs();
    max_i = 1;
    let tmp = scale[2] * a[2][1].abs();
    if tmp >= largest {
        max_i = 2;
        swap_vectors3(&mut a, 2, 1);
    }
    index[1] = max_i as i32;
    a[2][1] /= a[1][1];

    a[1][2] -= a[1][0] * a[0][2];
    a[2][2] -= a[2][0] * a[0][2] + a[2][1] * a[1][2];
    index[2] = 2;

    (a, index)
}

/// VTK: `vtkMath::LUSolve3x3`.
pub fn lu_solve3x3(a: [[f64; 3]; 3], index: [i32; 3], mut x: [f64; 3]) -> [f64; 3] {
    let i0 = usize::try_from(index[0]).expect("index[0] must be non-negative");
    let sum = x[i0];
    x[i0] = x[0];
    x[0] = sum;

    let i1 = usize::try_from(index[1]).expect("index[1] must be non-negative");
    let sum = x[i1];
    x[i1] = x[1];
    x[1] = sum - a[1][0] * x[0];

    let i2 = usize::try_from(index[2]).expect("index[2] must be non-negative");
    let sum = x[i2];
    x[i2] = x[2];
    x[2] = sum - a[2][0] * x[0] - a[2][1] * x[1];

    x[2] /= a[2][2];
    x[1] = (x[1] - a[1][2] * x[2]) / a[1][1];
    x[0] = (x[0] - a[0][1] * x[1] - a[0][2] * x[2]) / a[0][0];
    x
}

/// VTK: `vtkMath::LinearSolve3x3`.
pub fn linear_solve3x3(a: [[f64; 3]; 3], x: [f64; 3]) -> [f64; 3] {
    let a1 = a[0][0];
    let b1 = a[0][1];
    let c1 = a[0][2];
    let a2 = a[1][0];
    let b2 = a[1][1];
    let c2 = a[1][2];
    let a3 = a[2][0];
    let b3 = a[2][1];
    let c3 = a[2][2];

    let d1 = determinant2x2(b2, b3, c2, c3);
    let d2 = -determinant2x2(a2, a3, c2, c3);
    let d3 = determinant2x2(a2, a3, b2, b3);
    let e1 = -determinant2x2(b1, b3, c1, c3);
    let e2 = determinant2x2(a1, a3, c1, c3);
    let e3 = -determinant2x2(a1, a3, b1, b3);
    let f1 = determinant2x2(b1, b2, c1, c2);
    let f2 = -determinant2x2(a1, a2, c1, c2);
    let f3 = determinant2x2(a1, a2, b1, b2);

    let det = a1 * d1 + b1 * d2 + c1 * d3;
    [
        (d1 * x[0] + e1 * x[1] + f1 * x[2]) / det,
        (d2 * x[0] + e2 * x[1] + f2 * x[2]) / det,
        (d3 * x[0] + e3 * x[1] + f3 * x[2]) / det,
    ]
}

/// VTK: `vtkMath::Multiply3x3`.
pub fn multiply3x3(a: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        a[0][0] * v[0] + a[0][1] * v[1] + a[0][2] * v[2],
        a[1][0] * v[0] + a[1][1] * v[1] + a[1][2] * v[2],
        a[2][0] * v[0] + a[2][1] * v[1] + a[2][2] * v[2],
    ]
}

/// VTK: `vtkMath::Multiply3x3`.
pub fn multiply3x3_matrix(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut d = [[0.0; 3]; 3];
    for i in 0..3 {
        d[0][i] = a[0][0] * b[0][i] + a[0][1] * b[1][i] + a[0][2] * b[2][i];
        d[1][i] = a[1][0] * b[0][i] + a[1][1] * b[1][i] + a[1][2] * b[2][i];
        d[2][i] = a[2][0] * b[0][i] + a[2][1] * b[1][i] + a[2][2] * b[2][i];
    }
    d
}

/// VTK: `vtkMath::MultiplyMatrix`.
pub fn multiply_matrix(
    a: &[&[f64]],
    b: &[&[f64]],
    row_a: u32,
    col_a: u32,
    row_b: u32,
    col_b: u32,
) -> Vec<Vec<f64>> {
    let row_a = usize::try_from(row_a).expect("row_a must fit usize");
    let col_a = usize::try_from(col_a).expect("col_a must fit usize");
    let row_b = usize::try_from(row_b).expect("row_b must fit usize");
    let col_b = usize::try_from(col_b).expect("col_b must fit usize");

    assert!(row_a <= a.len(), "row_a exceeds A row count");
    if col_a != row_b {
        // VTK emits a warning but still runs the multiplication loop.
    }
    assert!(
        a[..row_a].iter().all(|row| col_a <= row.len()),
        "col_a exceeds A column count"
    );
    assert!(col_a <= b.len(), "col_a exceeds B row count");
    assert!(
        b[..col_a].iter().all(|row| col_b <= row.len()),
        "col_b exceeds B column count"
    );

    let mut c = vec![vec![0.0; col_b]; row_a];
    for i in 0..row_a {
        for j in 0..col_b {
            for k in 0..col_a {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    c
}

/// VTK: `vtkMath::Transpose3x3`.
pub fn transpose3x3(a: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [a[0][0], a[1][0], a[2][0]],
        [a[0][1], a[1][1], a[2][1]],
        [a[0][2], a[1][2], a[2][2]],
    ]
}

/// VTK: `vtkMath::QuaternionToMatrix3x3`.
pub fn quaternion_to_matrix3x3(quat: [f64; 4]) -> [[f64; 3]; 3] {
    let ww = quat[0] * quat[0];
    let wx = quat[0] * quat[1];
    let wy = quat[0] * quat[2];
    let wz = quat[0] * quat[3];

    let xx = quat[1] * quat[1];
    let yy = quat[2] * quat[2];
    let zz = quat[3] * quat[3];

    let xy = quat[1] * quat[2];
    let xz = quat[1] * quat[3];
    let yz = quat[2] * quat[3];

    let rr = xx + yy + zz;
    let mut f = 1.0 / (ww + rr);
    let s = (ww - rr) * f;
    f *= 2.0;

    [
        [xx * f + s, (xy - wz) * f, (xz + wy) * f],
        [(xy + wz) * f, yy * f + s, (yz - wx) * f],
        [(xz - wy) * f, (yz + wx) * f, zz * f + s],
    ]
}

/// VTK: `vtkMath::Matrix3x3ToQuaternion`.
pub fn matrix3x3_to_quaternion(a: [[f64; 3]; 3]) -> [f64; 4] {
    let n = [
        [
            a[0][0] + a[1][1] + a[2][2],
            a[2][1] - a[1][2],
            a[0][2] - a[2][0],
            a[1][0] - a[0][1],
        ],
        [
            a[2][1] - a[1][2],
            a[0][0] - a[1][1] - a[2][2],
            a[1][0] + a[0][1],
            a[0][2] + a[2][0],
        ],
        [
            a[0][2] - a[2][0],
            a[1][0] + a[0][1],
            -a[0][0] + a[1][1] - a[2][2],
            a[2][1] + a[1][2],
        ],
        [
            a[1][0] - a[0][1],
            a[0][2] + a[2][0],
            a[2][1] + a[1][2],
            -a[0][0] - a[1][1] + a[2][2],
        ],
    ];
    let rows: [&[f64]; 4] = [&n[0], &n[1], &n[2], &n[3]];
    let (_success, _mutated_n, _eigenvalues, eigenvectors) = jacobi_n(&rows, 4);
    [
        eigenvectors[0][0],
        eigenvectors[1][0],
        eigenvectors[2][0],
        eigenvectors[3][0],
    ]
}

/// VTK: `vtkMath::Invert3x3`.
pub fn invert3x3(a: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let a1 = a[0][0];
    let b1 = a[0][1];
    let c1 = a[0][2];
    let a2 = a[1][0];
    let b2 = a[1][1];
    let c2 = a[1][2];
    let a3 = a[2][0];
    let b3 = a[2][1];
    let c3 = a[2][2];

    let d1 = determinant2x2(b2, b3, c2, c3);
    let d2 = -determinant2x2(a2, a3, c2, c3);
    let d3 = determinant2x2(a2, a3, b2, b3);

    let e1 = -determinant2x2(b1, b3, c1, c3);
    let e2 = determinant2x2(a1, a3, c1, c3);
    let e3 = -determinant2x2(a1, a3, b1, b3);

    let f1 = determinant2x2(b1, b2, c1, c2);
    let f2 = -determinant2x2(a1, a2, c1, c2);
    let f3 = determinant2x2(a1, a2, b1, b2);

    let det = a1 * d1 + b1 * d2 + c1 * d3;

    [
        [d1 / det, e1 / det, f1 / det],
        [d2 / det, e2 / det, f2 / det],
        [d3 / det, e3 / det, f3 / det],
    ]
}

/// VTK: `vtkMath::Identity3x3`.
pub fn identity3x3() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// VTK: `vtkMath::MultiplyQuaternion`.
pub fn multiply_quaternion(q1: [f64; 4], q2: [f64; 4]) -> [f64; 4] {
    let ww = q1[0] * q2[0];
    let wx = q1[0] * q2[1];
    let wy = q1[0] * q2[2];
    let wz = q1[0] * q2[3];

    let xw = q1[1] * q2[0];
    let xx = q1[1] * q2[1];
    let xy = q1[1] * q2[2];
    let xz = q1[1] * q2[3];

    let yw = q1[2] * q2[0];
    let yx = q1[2] * q2[1];
    let yy = q1[2] * q2[2];
    let yz = q1[2] * q2[3];

    let zw = q1[3] * q2[0];
    let zx = q1[3] * q2[1];
    let zy = q1[3] * q2[2];
    let zz = q1[3] * q2[3];

    [
        ww - xx - yy - zz,
        wx + xw + yz - zy,
        wy - xz + yw + zx,
        wz + xy - yx + zw,
    ]
}

/// VTK: `vtkMath::RotateVectorByNormalizedQuaternion`.
pub fn rotate_vector_by_normalized_quaternion(v: [f64; 3], q: [f64; 4]) -> [f64; 3] {
    let f = (q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if f == 0.0 {
        return v;
    }

    let a = [q[1] / f, q[2] / f, q[3] / f];
    let t = 2.0 * f.atan2(q[0]);
    let cos_t = t.cos();
    let sin_t = t.sin();
    let dot_kv = dot(a, v);
    let cross_kv = cross(a, v);

    [
        v[0] * cos_t + cross_kv[0] * sin_t + a[0] * dot_kv * (1.0 - cos_t),
        v[1] * cos_t + cross_kv[1] * sin_t + a[1] * dot_kv * (1.0 - cos_t),
        v[2] * cos_t + cross_kv[2] * sin_t + a[2] * dot_kv * (1.0 - cos_t),
    ]
}

/// VTK: `vtkMath::RotateVectorByWXYZ`.
pub fn rotate_vector_by_wxyz(v: [f64; 3], q: [f64; 4]) -> [f64; 3] {
    let cos_t = q[0].cos();
    let sin_t = q[0].sin();
    let axis = [q[1], q[2], q[3]];
    let dot_kv = dot(axis, v);
    let cross_kv = cross(axis, v);

    [
        v[0] * cos_t + cross_kv[0] * sin_t + q[1] * dot_kv * (1.0 - cos_t),
        v[1] * cos_t + cross_kv[1] * sin_t + q[2] * dot_kv * (1.0 - cos_t),
        v[2] * cos_t + cross_kv[2] * sin_t + q[3] * dot_kv * (1.0 - cos_t),
    ]
}

/// VTK: `vtkMath::Orthogonalize3x3`.
pub fn orthogonalize3x3(a: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut b = a;
    let mut scale = [1.0; 3];
    let mut index = [0usize; 3];

    for i in 0..3 {
        let x1 = b[i][0].abs();
        let x2 = b[i][1].abs();
        let x3 = b[i][2].abs();
        let largest = x1.max(x2).max(x3);
        if largest != 0.0 {
            scale[i] /= largest;
        }
    }

    let x1 = b[0][0].abs() * scale[0];
    let x2 = b[1][0].abs() * scale[1];
    let x3 = b[2][0].abs() * scale[2];
    index[0] = 0;
    let mut largest = x1;
    if x2 >= largest {
        largest = x2;
        index[0] = 1;
    }
    if x3 >= largest {
        index[0] = 2;
    }
    if index[0] != 0 {
        b.swap(index[0], 0);
        scale[index[0]] = scale[0];
    }

    let y2 = b[1][1].abs() * scale[1];
    let y3 = b[2][1].abs() * scale[2];
    index[1] = 1;
    if y3 >= y2 {
        index[1] = 2;
        b.swap(2, 1);
    }

    index[2] = 2;

    let flip = determinant3x3(b) < 0.0;
    if flip {
        for row in &mut b {
            for value in row {
                *value = -*value;
            }
        }
    }

    let quat = matrix3x3_to_quaternion(b);
    b = quaternion_to_matrix3x3(quat);

    if flip {
        for row in &mut b {
            for value in row {
                *value = -*value;
            }
        }
    }

    if index[1] != 1 {
        b.swap(index[1], 1);
    }
    if index[0] != 0 {
        b.swap(index[0], 0);
    }

    b
}

/// VTK: `vtkMath::Diagonalize3x3`.
pub fn diagonalize3x3(a: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let rows: [&[f64]; 3] = [&a[0], &a[1], &a[2]];
    let (_success, _mutated_a, eigenvalues, eigenvectors) = jacobi_n(&rows, 3);
    let mut w = [eigenvalues[0], eigenvalues[1], eigenvalues[2]];
    let mut v = [
        [eigenvectors[0][0], eigenvectors[0][1], eigenvectors[0][2]],
        [eigenvectors[1][0], eigenvectors[1][1], eigenvectors[1][2]],
        [eigenvectors[2][0], eigenvectors[2][1], eigenvectors[2][2]],
    ];

    if w[0] == w[1] && w[0] == w[2] {
        return (w, identity3x3());
    }

    v = transpose3x3(v);

    for i in 0..3 {
        if w[(i + 1) % 3] == w[(i + 2) % 3] {
            let mut max_val = v[i][0].abs();
            let mut max_i = 0;
            for j in 1..3 {
                let tmp = v[i][j].abs();
                if max_val < tmp {
                    max_val = tmp;
                    max_i = j;
                }
            }

            if max_i != i {
                w.swap(max_i, i);
                v.swap(i, max_i);
            }

            if v[max_i][max_i] < 0.0 {
                for value in &mut v[max_i] {
                    *value = -*value;
                }
            }

            let j = (max_i + 1) % 3;
            let k = (max_i + 2) % 3;

            v[j] = [0.0; 3];
            v[j][j] = 1.0;
            v[k] = cross(v[max_i], v[j]);
            normalize(&mut v[k]);
            v[j] = cross(v[k], v[max_i]);

            return (w, transpose3x3(v));
        }
    }

    let mut max_val = v[0][0].abs();
    let mut max_i = 0;
    for i in 1..3 {
        let tmp = v[i][0].abs();
        if max_val < tmp {
            max_val = tmp;
            max_i = i;
        }
    }

    if max_i != 0 {
        w.swap(max_i, 0);
        v.swap(max_i, 0);
    }

    if v[1][1].abs() < v[2][1].abs() {
        w.swap(2, 1);
        v.swap(2, 1);
    }

    for i in 0..2 {
        if v[i][i] < 0.0 {
            for value in &mut v[i] {
                *value = -*value;
            }
        }
    }

    if determinant3x3(v) < 0.0 {
        for value in &mut v[2] {
            *value = -*value;
        }
    }

    (w, transpose3x3(v))
}

/// VTK: `vtkMath::SingularValueDecomposition3x3`.
pub fn singular_value_decomposition3x3(
    a: [[f64; 3]; 3],
) -> ([[f64; 3]; 3], [f64; 3], [[f64; 3]; 3]) {
    let mut b = a;

    let d = determinant3x3(b);
    if d < 0.0 {
        for row in &mut b {
            for value in row {
                *value = -*value;
            }
        }
    }

    let mut u = orthogonalize3x3(b);
    b = transpose3x3(b);
    let mut vt = multiply3x3_matrix(b, u);
    let (mut w, diagonalized_vt) = diagonalize3x3(vt);
    vt = diagonalized_vt;
    u = multiply3x3_matrix(u, vt);
    vt = transpose3x3(vt);

    if d < 0.0 {
        for value in &mut w {
            *value = -*value;
        }
    }

    (u, w, vt)
}

/// VTK: `vtkMath::SolveLinearSystemGEPP2x2`.
#[allow(clippy::too_many_arguments)]
pub fn solve_linear_system_gepp2x2(
    mut a00: f64,
    mut a01: f64,
    mut a10: f64,
    mut a11: f64,
    mut b0: f64,
    mut b1: f64,
) -> (bool, [f64; 2]) {
    let mut cols_swapped = false;
    if a00 == 0.0 || a01 == 0.0 || a10 == 0.0 || a11 == 0.0 {
        if a01 == 0.0 || a11 == 0.0 {
            std::mem::swap(&mut a00, &mut a01);
            std::mem::swap(&mut a10, &mut a11);
            cols_swapped = true;
        }
        if a00 == 0.0 {
            std::mem::swap(&mut a00, &mut a10);
            std::mem::swap(&mut a01, &mut a11);
            std::mem::swap(&mut b0, &mut b1);
        }
    } else {
        if a00.abs() < a10.abs() {
            std::mem::swap(&mut a00, &mut a10);
            std::mem::swap(&mut a01, &mut a11);
            std::mem::swap(&mut b0, &mut b1);
        }
        let f = -a10 / a00;
        a11 += a01 * f;
        b1 += b0 * f;
    }

    let eps = 256.0 * f64::EPSILON;
    if a11.abs() < eps {
        return (false, [0.0; 2]);
    }

    if a11 == 0.0 || a00 == 0.0 {
        return (false, [0.0; 2]);
    }
    let x1 = b1 / a11;
    let x0 = (b0 - a01 * x1) / a00;
    if !x0.is_finite() || !x1.is_finite() {
        return (false, [0.0; 2]);
    }
    if cols_swapped {
        (true, [x1, x0])
    } else {
        (true, [x0, x1])
    }
}

/// VTK: `vtkMath::LUFactorLinearSystem`.
pub fn lu_factor_linear_system(mut a: Vec<Vec<f64>>, size: i32) -> (bool, Vec<Vec<f64>>, Vec<i32>) {
    let size = usize::try_from(size).expect("size must be non-negative");
    assert!(size <= a.len(), "size exceeds A row count");
    assert!(
        a[..size].iter().all(|row| size <= row.len()),
        "size exceeds A column count"
    );

    let mut index = vec![0; size];
    let mut scale = vec![0.0; size];
    let mut max_i = 0;

    for i in 0..size {
        let mut largest = 0.0;
        for j in 0..size {
            let temp2 = a[i][j].abs();
            if temp2 > largest {
                largest = temp2;
            }
        }

        if largest == 0.0 {
            return (false, a, index);
        }
        scale[i] = 1.0 / largest;
    }

    for j in 0..size {
        for i in 0..j {
            let mut sum = a[i][j];
            for k in 0..i {
                sum -= a[i][k] * a[k][j];
            }
            a[i][j] = sum;
        }

        let mut largest = 0.0;
        for i in j..size {
            let mut sum = a[i][j];
            for k in 0..j {
                sum -= a[i][k] * a[k][j];
            }
            a[i][j] = sum;

            let temp1 = scale[i] * sum.abs();
            if temp1 >= largest {
                largest = temp1;
                max_i = i;
            }
        }

        if j != max_i {
            a.swap(max_i, j);
            scale[max_i] = scale[j];
        }

        index[j] = max_i as i32;

        if a[j][j].abs() <= VTK_SMALL_NUMBER {
            return (false, a, index);
        }

        if j != size - 1 {
            let temp1 = 1.0 / a[j][j];
            for row in a.iter_mut().take(size).skip(j + 1) {
                row[j] *= temp1;
            }
        }
    }

    (true, a, index)
}

/// VTK: `vtkMath::LUSolveLinearSystem`.
pub fn lu_solve_linear_system(
    a: &[Vec<f64>],
    index: &[i32],
    mut x: Vec<f64>,
    size: i32,
) -> Vec<f64> {
    let size = usize::try_from(size).expect("size must be non-negative");
    assert!(size <= a.len(), "size exceeds A row count");
    assert!(size <= index.len(), "size exceeds index length");
    assert!(size <= x.len(), "size exceeds x length");
    assert!(
        a[..size].iter().all(|row| size <= row.len()),
        "size exceeds A column count"
    );

    let mut ii = None;
    for i in 0..size {
        let idx = usize::try_from(index[i]).expect("pivot index must be non-negative");
        let mut sum = x[idx];
        x[idx] = x[i];

        if let Some(ii) = ii {
            for j in ii..i {
                sum -= a[i][j] * x[j];
            }
        } else if sum != 0.0 {
            ii = Some(i);
        }

        x[i] = sum;
    }

    for i in (0..size).rev() {
        let mut sum = x[i];
        for j in (i + 1)..size {
            sum -= a[i][j] * x[j];
        }
        x[i] = sum / a[i][i];
    }

    x
}

/// VTK: `vtkMath::SolveLinearSystem`.
pub fn solve_linear_system(
    mut a: Vec<Vec<f64>>,
    mut x: Vec<f64>,
    size: i32,
) -> (bool, Vec<Vec<f64>>, Vec<f64>) {
    let size_usize = usize::try_from(size).expect("size must be non-negative");
    assert!(size_usize <= a.len(), "size exceeds A row count");
    assert!(size_usize <= x.len(), "size exceeds x length");
    assert!(
        a[..size_usize].iter().all(|row| size_usize <= row.len()),
        "size exceeds A column count"
    );

    if size == 2 {
        let (success, solution) =
            solve_linear_system_gepp2x2(a[0][0], a[0][1], a[1][0], a[1][1], x[0], x[1]);
        if success {
            x[0] = solution[0];
            x[1] = solution[1];
        }
        return (success, a, x);
    } else if size == 1 {
        if a[0][0] == 0.0 {
            return (false, a, x);
        }
        x[0] /= a[0][0];
        return (true, a, x);
    }

    let (success, factored, index) = lu_factor_linear_system(a, size);
    a = factored;
    if !success {
        return (false, a, x);
    }
    x = lu_solve_linear_system(&a, &index, x, size);
    (true, a, x)
}

/// VTK: `vtkMath::InvertMatrix`.
pub fn invert_matrix(mut a: Vec<Vec<f64>>, size: i32) -> (bool, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let size_usize = usize::try_from(size).expect("size must be non-negative");
    assert!(size_usize <= a.len(), "size exceeds A row count");
    assert!(
        a[..size_usize].iter().all(|row| size_usize <= row.len()),
        "size exceeds A column count"
    );

    let mut inverse = vec![vec![0.0; size_usize]; size_usize];
    let (success, factored, index) = lu_factor_linear_system(a, size);
    a = factored;
    if !success {
        return (false, a, inverse);
    }

    for j in 0..size_usize {
        let mut column = vec![0.0; size_usize];
        column[j] = 1.0;
        column = lu_solve_linear_system(&a, &index, column, size);
        for i in 0..size_usize {
            inverse[i][j] = column[i];
        }
    }

    (true, a, inverse)
}

/// VTK: `vtkMath::EstimateMatrixCondition`.
pub fn estimate_matrix_condition(a: &[&[f64]], size: i32) -> f64 {
    let size = usize::try_from(size).expect("size must be non-negative");
    assert!(size <= a.len(), "size exceeds A row count");
    assert!(
        a[..size].iter().all(|row| size <= row.len()),
        "size exceeds A column count"
    );

    let mut min = VTK_FLOAT_MAX;
    let mut max = -VTK_FLOAT_MAX;

    for (i, row) in a.iter().enumerate().take(size) {
        for value in row.iter().take(size).skip(i) {
            max = value.abs().max(max);
        }
    }

    for (i, row) in a.iter().enumerate().take(size) {
        min = row[i].abs().min(min);
    }

    if min == 0.0 {
        VTK_FLOAT_MAX
    } else {
        max / min
    }
}

fn jacobi_rotate(a: &mut [Vec<f64>], i: usize, j: usize, k: usize, l: usize, s: f64, tau: f64) {
    let g = a[i][j];
    let h = a[k][l];
    a[i][j] = g - s * (h + g * tau);
    a[k][l] = h + s * (g - h * tau);
}

/// VTK: `vtkMath::JacobiN`.
pub fn jacobi_n(a: &[&[f64]], n: i32) -> (bool, Vec<Vec<f64>>, Vec<f64>, Vec<Vec<f64>>) {
    let n = usize::try_from(n).expect("n must be non-negative");
    assert!(n <= a.len(), "n exceeds A row count");
    assert!(
        a[..n].iter().all(|row| n <= row.len()),
        "n exceeds A column count"
    );

    let mut a: Vec<Vec<f64>> = a[..n].iter().map(|row| row[..n].to_vec()).collect();
    let mut v = vec![vec![0.0; n]; n];
    let mut w = vec![0.0; n];
    let mut b = vec![0.0; n];
    let mut z = vec![0.0; n];

    for ip in 0..n {
        v[ip][ip] = 1.0;
    }
    for ip in 0..n {
        b[ip] = a[ip][ip];
        w[ip] = a[ip][ip];
    }

    let mut rotations = 0;
    while rotations < VTK_MAX_ROTATIONS {
        let mut sm = 0.0;
        for ip in 0..n.saturating_sub(1) {
            for iq in ip + 1..n {
                sm += a[ip][iq].abs();
            }
        }
        if sm == 0.0 {
            break;
        }

        let tresh = if rotations < 3 {
            0.2 * sm / (n * n) as f64
        } else {
            0.0
        };

        for ip in 0..n.saturating_sub(1) {
            for iq in ip + 1..n {
                let g = 100.0 * a[ip][iq].abs();

                if rotations > 3
                    && (w[ip].abs() + g) == w[ip].abs()
                    && (w[iq].abs() + g) == w[iq].abs()
                {
                    a[ip][iq] = 0.0;
                } else if a[ip][iq].abs() > tresh {
                    let h = w[iq] - w[ip];
                    let t = if (h.abs() + g) == h.abs() {
                        a[ip][iq] / h
                    } else {
                        let theta = 0.5 * h / a[ip][iq];
                        let mut t = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
                        if theta < 0.0 {
                            t = -t;
                        }
                        t
                    };
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;
                    let tau = s / (1.0 + c);
                    let h = t * a[ip][iq];
                    z[ip] -= h;
                    z[iq] += h;
                    w[ip] -= h;
                    w[iq] += h;
                    a[ip][iq] = 0.0;

                    for j in 0..ip {
                        jacobi_rotate(&mut a, j, ip, j, iq, s, tau);
                    }
                    for j in ip + 1..iq {
                        jacobi_rotate(&mut a, ip, j, j, iq, s, tau);
                    }
                    for j in iq + 1..n {
                        jacobi_rotate(&mut a, ip, j, iq, j, s, tau);
                    }
                    for j in 0..n {
                        jacobi_rotate(&mut v, j, ip, j, iq, s, tau);
                    }
                }
            }
        }

        for ip in 0..n {
            b[ip] += z[ip];
            w[ip] = b[ip];
            z[ip] = 0.0;
        }

        rotations += 1;
    }

    if rotations >= VTK_MAX_ROTATIONS {
        return (false, a, w, v);
    }

    for j in 0..n.saturating_sub(1) {
        let mut k = j;
        let mut tmp = w[k];
        for (i, value) in w.iter().enumerate().skip(j + 1) {
            if *value >= tmp {
                k = i;
                tmp = *value;
            }
        }
        if k != j {
            w[k] = w[j];
            w[j] = tmp;
            for row in v.iter_mut().take(n) {
                row.swap(j, k);
            }
        }
    }

    let ceil_half_n = (n >> 1) + (n & 1);
    for j in 0..n {
        let mut num_pos = 0;
        for row in v.iter().take(n) {
            if row[j] >= 0.0 {
                num_pos += 1;
            }
        }
        if num_pos < ceil_half_n {
            for row in v.iter_mut().take(n) {
                row[j] *= -1.0;
            }
        }
    }

    (true, a, w, v)
}

/// VTK: `vtkMath::Jacobi`.
pub fn jacobi(a: [[f64; 3]; 3]) -> (bool, [[f64; 3]; 3], [f64; 3], [[f64; 3]; 3]) {
    let rows: [&[f64]; 3] = [&a[0], &a[1], &a[2]];
    let (success, a, w, v) = jacobi_n(&rows, 3);
    let mut a3 = [[0.0; 3]; 3];
    let mut w3 = [0.0; 3];
    let mut v3 = [[0.0; 3]; 3];
    for i in 0..3 {
        w3[i] = w[i];
        for j in 0..3 {
            a3[i][j] = a[i][j];
            v3[i][j] = v[i][j];
        }
    }
    (success, a3, w3, v3)
}

/// VTK: `vtkMath::SolveHomogeneousLeastSquares`.
pub fn solve_homogeneous_least_squares(
    number_of_samples: i32,
    xt: &[&[f64]],
    x_order: i32,
) -> (bool, Vec<Vec<f64>>) {
    let number_of_samples =
        usize::try_from(number_of_samples).expect("number_of_samples must be non-negative");
    let x_order = usize::try_from(x_order).expect("x_order must be non-negative");
    assert!(
        number_of_samples <= xt.len(),
        "number_of_samples exceeds X' row count"
    );
    assert!(
        xt[..number_of_samples]
            .iter()
            .all(|row| x_order <= row.len()),
        "x_order exceeds X' column count"
    );

    let mut mt = vec![vec![0.0; 1]; x_order];
    if number_of_samples < x_order {
        return (false, mt);
    }

    let mut xxt = vec![vec![0.0; x_order]; x_order];
    for row in xt.iter().take(number_of_samples) {
        for i in 0..x_order {
            for j in i..x_order {
                xxt[i][j] += row[i] * row[j];
            }
        }
    }

    for i in 0..x_order {
        for j in 0..i {
            xxt[i][j] = xxt[j][i];
        }
    }

    let xxt_rows: Vec<&[f64]> = xxt.iter().map(Vec::as_slice).collect();
    let (_success, _mutated_xxt, _eigenvalues, eigenvectors) = jacobi_n(&xxt_rows, x_order as i32);
    for i in 0..x_order {
        mt[i][0] = eigenvectors[i][x_order - 1];
    }

    (true, mt)
}

/// VTK: `vtkMath::SolveLeastSquares`.
pub fn solve_least_squares(
    number_of_samples: i32,
    xt: &[&[f64]],
    x_order: i32,
    yt: &[&[f64]],
    y_order: i32,
    check_homogeneous: i32,
) -> (bool, Vec<Vec<f64>>) {
    let number_of_samples =
        usize::try_from(number_of_samples).expect("number_of_samples must be non-negative");
    let x_order = usize::try_from(x_order).expect("x_order must be non-negative");
    let y_order = usize::try_from(y_order).expect("y_order must be non-negative");
    assert!(
        number_of_samples <= xt.len(),
        "number_of_samples exceeds X' row count"
    );
    assert!(
        number_of_samples <= yt.len(),
        "number_of_samples exceeds Y' row count"
    );
    assert!(
        xt[..number_of_samples]
            .iter()
            .all(|row| x_order <= row.len()),
        "x_order exceeds X' column count"
    );
    assert!(
        yt[..number_of_samples]
            .iter()
            .all(|row| y_order <= row.len()),
        "y_order exceeds Y' column count"
    );

    let mut mt = vec![vec![0.0; y_order]; x_order];
    if number_of_samples < x_order || number_of_samples < y_order {
        return (false, mt);
    }

    let mut some_homogeneous = false;
    let mut all_homogeneous = true;
    let mut homogeneous_result = vec![vec![0.0; 1]; x_order];
    let mut homogeneous_success = false;
    let mut homogeneous_flags = vec![0; y_order];

    if check_homogeneous != 0 {
        homogeneous_flags.fill(1);
        for row in yt.iter().take(number_of_samples) {
            for j in 0..y_order {
                if row[j].abs() > VTK_SMALL_NUMBER {
                    all_homogeneous = false;
                    homogeneous_flags[j] = 0;
                }
            }
        }

        if all_homogeneous && y_order == 1 {
            let (success, homogeneous_mt) =
                solve_homogeneous_least_squares(number_of_samples as i32, xt, x_order as i32);
            return (success, homogeneous_mt);
        }

        if all_homogeneous {
            some_homogeneous = true;
        } else {
            for flag in &homogeneous_flags {
                if *flag != 0 {
                    some_homogeneous = true;
                }
            }
        }
    }

    if some_homogeneous {
        let result = solve_homogeneous_least_squares(number_of_samples as i32, xt, x_order as i32);
        homogeneous_success = result.0;
        homogeneous_result = result.1;
    }

    let mut xxt = vec![vec![0.0; x_order]; x_order];
    let mut xyt = vec![vec![0.0; y_order]; x_order];
    for k in 0..number_of_samples {
        for i in 0..x_order {
            for j in i..x_order {
                xxt[i][j] += xt[k][i] * xt[k][j];
            }
            for j in 0..y_order {
                xyt[i][j] += xt[k][i] * yt[k][j];
            }
        }
    }

    for i in 0..x_order {
        for j in 0..i {
            xxt[i][j] = xxt[j][i];
        }
    }

    let (success_flag, _factored_xxt, xxt_inverse) = invert_matrix(xxt, x_order as i32);
    if success_flag {
        for i in 0..x_order {
            for j in 0..y_order {
                mt[i][j] = 0.0;
                for (k, row) in xyt.iter().enumerate().take(x_order) {
                    mt[i][j] += xxt_inverse[i][k] * row[j];
                }
            }
        }
    }

    if some_homogeneous {
        for j in 0..y_order {
            if homogeneous_flags[j] != 0 {
                for i in 0..x_order {
                    mt[i][j] = homogeneous_result[i][0];
                }
            }
        }
        (homogeneous_success && success_flag, mt)
    } else {
        (success_flag, mt)
    }
}

/// VTK: `vtkMath::QuadraticRoot`.
pub fn quadratic_root(a: f64, b: f64, c: f64, min: f64, max: f64) -> (i32, [f64; 2]) {
    let mut u = [0.0; 2];
    if a == 0.0 {
        if b != 0.0 {
            u[0] = -c / b;
            if u[0] > min && u[0] < max {
                return (1, u);
            }
        }
        return (0, u);
    }

    let d = b * b - 4.0 * a * c;
    if d <= 0.0 {
        if d == 0.0 {
            u[0] = -b / a;
            if u[0] > min && u[0] < max {
                return (1, u);
            }
        }
        return (0, u);
    }

    let q = -0.5 * (b + d.sqrt().copysign(b));
    u[0] = c / q;
    u[1] = q / a;

    if (u[0] > min && u[0] < max) && (u[1] > min && u[1] < max) {
        return (2, u);
    }
    if u[0] > min && u[0] < max {
        return (1, u);
    }
    if u[1] > min && u[1] < max {
        u.swap(0, 1);
        return (1, u);
    }
    (0, u)
}

/// VTK: `vtkMath::Factorial`.
pub fn factorial(n: i32) -> i64 {
    if n > 20 {
        return i64::MAX;
    }
    if n <= 0 {
        return 1;
    }
    i64::from(n) * factorial(n - 1)
}

/// VTK: `vtkMath::Binomial`.
pub fn binomial(m: i32, n: i32) -> i64 {
    let mut result = 1.0;
    for i in 1..=n {
        result *= f64::from(m - i + 1) / f64::from(i);
    }
    result as i64
}

/// VTK: `vtkMath::BeginCombination`.
pub fn begin_combination(m: i32, n: i32) -> Option<Vec<i32>> {
    if m < n {
        return None;
    }
    Some((0..n).collect())
}

/// VTK: `vtkMath::NextCombination`.
pub fn next_combination(m: i32, n: i32, combination: &mut [i32]) -> i32 {
    let mut i = n;
    while i > 0 {
        i -= 1;
        let index = usize::try_from(i).expect("combination index must be non-negative");
        if combination[index] < m - n + i {
            let mut j = combination[index] + 1;
            while i < n {
                let index = usize::try_from(i).expect("combination index must be non-negative");
                combination[index] = j;
                i += 1;
                j += 1;
            }
            return 1;
        }
    }
    0
}

/// VTK: `vtkMath::CeilLog2`.
pub fn ceil_log2(mut x: u64) -> i32 {
    const T: [u64; 6] = [
        0xffffffff00000000,
        0x00000000ffff0000,
        0x000000000000ff00,
        0x00000000000000f0,
        0x000000000000000c,
        0x0000000000000002,
    ];
    let mut j = 32;
    let mut y = if (x & x.wrapping_sub(1)) == 0 { 0 } else { 1 };
    for mask in T {
        let k = if (x & mask) == 0 { 0 } else { j };
        y += k;
        x >>= k;
        j >>= 1;
    }
    y
}

#[cfg(test)]
fn floor_log2(x: u64) -> Option<u32> {
    if x == 0 {
        None
    } else {
        Some(u64::BITS - 1 - x.leading_zeros())
    }
}

/// VTK: `vtkMath::IsPowerOfTwo`.
pub fn is_power_of_two(x: u64) -> bool {
    x != 0 && (x & (x - 1)) == 0
}

/// VTK: `vtkMath::NearestPowerOfTwo`.
pub fn nearest_power_of_two(x: i32) -> i32 {
    let mut z = if x > 0 { (x - 1) as u32 } else { 0 };
    z |= z >> 1;
    z |= z >> 2;
    z |= z >> 4;
    z |= z >> 8;
    z |= z >> 16;
    z.wrapping_add(1) as i32
}

/// VTK: `vtkMath::Floor`.
pub fn floor(x: f64) -> i32 {
    let i = x as i32;
    i - i32::from((i as f64) > x)
}

/// VTK: `vtkMath::Ceil`.
pub fn ceil(x: f64) -> i32 {
    let i = x as i32;
    i + i32::from((i as f64) < x)
}

/// VTK: `vtkMath::Min`.
pub fn min<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if b <= a {
        b
    } else {
        a
    }
}

/// VTK: `vtkMath::Max`.
pub fn max<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if b > a {
        b
    } else {
        a
    }
}

/// VTK: `vtkMath::Inf`.
pub fn inf() -> f64 {
    f64::INFINITY
}

/// VTK: `vtkMath::NegInf`.
pub fn neg_inf() -> f64 {
    f64::NEG_INFINITY
}

/// VTK: `vtkMath::Nan`.
pub fn nan() -> f64 {
    f64::NAN
}

/// VTK: `vtkMath::IsInf`.
pub fn is_inf(x: f64) -> bool {
    x.is_infinite()
}

/// VTK: `vtkMath::IsNan`.
pub fn is_nan(x: f64) -> bool {
    x.is_nan()
}

/// VTK: `vtkMath::IsFinite`.
pub fn is_finite(x: f64) -> bool {
    x.is_finite()
}

/// VTK: `vtkMath::GaussianAmplitude`.
pub fn gaussian_amplitude(variance: f64, distance_from_mean: f64) -> f64 {
    1.0 / (2.0 * std::f64::consts::PI * variance).sqrt()
        * (-(distance_from_mean * distance_from_mean) / (2.0 * variance)).exp()
}

/// VTK: `vtkMath::GaussianAmplitude`.
pub fn gaussian_amplitude_at(mean: f64, variance: f64, position: f64) -> f64 {
    gaussian_amplitude(variance, (mean - position).abs())
}

/// VTK: `vtkMath::GaussianWeight`.
pub fn gaussian_weight(variance: f64, distance_from_mean: f64) -> f64 {
    (-(distance_from_mean * distance_from_mean) / (2.0 * variance)).exp()
}

/// VTK: `vtkMath::GaussianWeight`.
pub fn gaussian_weight_at(mean: f64, variance: f64, position: f64) -> f64 {
    gaussian_weight(variance, (mean - position).abs())
}

/// VTK: `vtkMath::ExtentIsWithinOtherExtent`.
pub fn extent_is_within_other_extent(extent1: [i32; 6], extent2: [i32; 6]) -> bool {
    (0..6).step_by(2).all(|i| {
        extent1[i] >= extent2[i]
            && extent1[i] <= extent2[i + 1]
            && extent1[i + 1] >= extent2[i]
            && extent1[i + 1] <= extent2[i + 1]
    })
}

/// VTK: `vtkMath::BoundsIsWithinOtherBounds`.
pub fn bounds_is_within_other_bounds(
    bounds1: [f64; 6],
    bounds2: [f64; 6],
    delta: [f64; 3],
) -> bool {
    (0..6).step_by(2).all(|i| {
        let d = delta[i / 2];
        bounds1[i] + d >= bounds2[i]
            && bounds1[i] - d <= bounds2[i + 1]
            && bounds1[i + 1] + d >= bounds2[i]
            && bounds1[i + 1] - d <= bounds2[i + 1]
    })
}

/// VTK: `vtkMath::PointIsWithinBounds`.
pub fn point_is_within_bounds(point: [f64; 3], bounds: [f64; 6], delta: [f64; 3]) -> bool {
    point[0] + delta[0] >= bounds[0]
        && point[0] - delta[0] <= bounds[1]
        && point[1] + delta[1] >= bounds[2]
        && point[1] - delta[1] <= bounds[3]
        && point[2] + delta[2] >= bounds[4]
        && point[2] - delta[2] <= bounds[5]
}

/// VTK: `vtkMath::PlaneIntersectsAABB`.
pub fn plane_intersects_aabb(bounds: [f64; 6], normal: [f64; 3], point: [f64; 3]) -> i32 {
    let mut n_point = [0.0; 3];
    let mut p_point = [0.0; 3];

    if normal[0] >= 0.0 {
        n_point[0] = bounds[0];
        p_point[0] = bounds[1];
    } else {
        n_point[0] = bounds[1];
        p_point[0] = bounds[0];
    }

    if normal[1] >= 0.0 {
        n_point[1] = bounds[2];
        p_point[1] = bounds[3];
    } else {
        n_point[1] = bounds[3];
        p_point[1] = bounds[2];
    }

    if normal[2] >= 0.0 {
        n_point[2] = bounds[4];
        p_point[2] = bounds[5];
    } else {
        n_point[2] = bounds[5];
        p_point[2] = bounds[4];
    }

    let d = dot(normal, point);
    if dot(n_point, normal) - d > 0.0 {
        1
    } else if dot(p_point, normal) - d < 0.0 {
        -1
    } else {
        0
    }
}

/// VTK: `vtkMath::Solve3PointCircle`.
pub fn solve3_point_circle(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]) -> (f64, [f64; 3]) {
    let v21 = subtract(p1, p2);
    let v32 = subtract(p2, p3);
    let v13 = subtract(p3, p1);
    let v12 = scale(v21, -1.0);
    let v23 = scale(v32, -1.0);
    let v31 = scale(v13, -1.0);

    let norm12 = norm(&v12);
    let norm23 = norm(&v23);
    let norm13 = norm(&v13);

    let crossv21v32 = cross(v21, v32);
    let norm_cross = norm(&crossv21v32);

    let radius = (norm12 * norm23 * norm13) / (2.0 * norm_cross);
    let norm_cross2 = norm_cross * norm_cross;
    let alpha = ((norm23 * norm23) * dot(v21, v31)) / (2.0 * norm_cross2);
    let beta = ((norm13 * norm13) * dot(v12, v32)) / (2.0 * norm_cross2);
    let gamma = ((norm12 * norm12) * dot(v13, v23)) / (2.0 * norm_cross2);

    let center = [
        alpha * p1[0] + beta * p2[0] + gamma * p3[0],
        alpha * p1[1] + beta * p2[1] + gamma * p3[1],
        alpha * p1[2] + beta * p2[2] + gamma * p3[2],
    ];

    (radius, center)
}

/// VTK: `vtkMath::RGBToHSV`.
pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let mut cmax = r;
    let mut cmin = r;
    if g > cmax {
        cmax = g;
    } else if g < cmin {
        cmin = g;
    }
    if b > cmax {
        cmax = b;
    } else if b < cmin {
        cmin = b;
    }

    let v = cmax;
    let s = if v > 0.0 { (cmax - cmin) / cmax } else { 0.0 };

    let h = if s > 0.0 {
        let mut h = if r == cmax {
            (1.0 / 6.0) * (g - b) / (cmax - cmin)
        } else if g == cmax {
            (1.0 / 3.0) + (1.0 / 6.0) * (b - r) / (cmax - cmin)
        } else {
            (2.0 / 3.0) + (1.0 / 6.0) * (r - g) / (cmax - cmin)
        };
        if h < 0.0 {
            h += 1.0;
        }
        h
    } else {
        0.0
    };

    (h, s, v)
}

/// VTK: `vtkMath::HSVToRGB`.
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let onethird = 1.0 / 3.0;
    let onesixth = 1.0 / 6.0;
    let twothird = 2.0 / 3.0;
    let fivesixth = 5.0 / 6.0;

    let (mut r, mut g, mut b) = if h > onesixth && h <= onethird {
        ((onethird - h) / onesixth, 1.0, 0.0)
    } else if h > onethird && h <= 0.5 {
        (0.0, 1.0, (h - onethird) / onesixth)
    } else if h > 0.5 && h <= twothird {
        (0.0, (twothird - h) / onesixth, 1.0)
    } else if h > twothird && h <= fivesixth {
        ((h - twothird) / onesixth, 0.0, 1.0)
    } else if h > fivesixth && h <= 1.0 {
        (1.0, 0.0, (1.0 - h) / onesixth)
    } else {
        (1.0, h / onesixth, 0.0)
    };

    r = s * r + (1.0 - s);
    g = s * g + (1.0 - s);
    b = s * b + (1.0 - s);

    (r * v, g * v, b * v)
}

fn multiply_homogeneous_row_by_transposed_matrix(row: [f64; 4], matrix: [[f64; 4]; 4]) -> [f64; 4] {
    [
        row[0] * matrix[0][0]
            + row[1] * matrix[0][1]
            + row[2] * matrix[0][2]
            + row[3] * matrix[0][3],
        row[0] * matrix[1][0]
            + row[1] * matrix[1][1]
            + row[2] * matrix[1][2]
            + row[3] * matrix[1][3],
        row[0] * matrix[2][0]
            + row[1] * matrix[2][1]
            + row[2] * matrix[2][2]
            + row[3] * matrix[2][3],
        row[0] * matrix[3][0]
            + row[1] * matrix[3][1]
            + row[2] * matrix[3][2]
            + row[3] * matrix[3][3],
    ]
}

/// VTK: `vtkMath::ProLabToXYZ`.
pub fn pro_lab_to_xyz(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let qi = [
        [
            0.00137063282117354,
            0.00138738203138321,
            0.000816068851107095,
            0.0,
        ],
        [
            0.00137063282117354,
            -0.000243154854293407,
            0.000965329194924993,
            0.0,
        ],
        [
            0.00137063282117354,
            8.08345942991924e-05,
            -0.00317481896776885,
            0.0,
        ],
        [
            -0.00862936717882646,
            -0.000243154854293407,
            0.000965329194924994,
            1.0,
        ],
    ];
    let xyz_homog = multiply_homogeneous_row_by_transposed_matrix([l, a, b, 1.0], qi);

    let var_x = xyz_homog[0] / xyz_homog[3];
    let var_y = xyz_homog[1] / xyz_homog[3];
    let var_z = xyz_homog[2] / xyz_homog[3];

    let ref_x = 0.9505;
    let ref_y = 1.000;
    let ref_z = 1.089;
    (var_x * ref_x, var_y * ref_y, var_z * ref_z)
}

/// VTK: `vtkMath::XYZToProLab`.
pub fn xyz_to_pro_lab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let ref_x = 0.9505;
    let ref_y = 1.000;
    let ref_z = 1.089;
    let var_x = x / ref_x;
    let var_y = y / ref_y;
    let var_z = z / ref_z;

    let q = [
        [75.54, 486.66, 167.39, 0.0],
        [617.72, -595.45, -22.27, 0.0],
        [48.34, 194.94, -243.28, 0.0],
        [0.7554, 3.8666, 1.6739, 1.0],
    ];
    let prolab_homog = multiply_homogeneous_row_by_transposed_matrix([var_x, var_y, var_z, 1.0], q);

    (
        prolab_homog[0] / prolab_homog[3],
        prolab_homog[1] / prolab_homog[3],
        prolab_homog[2] / prolab_homog[3],
    )
}

/// VTK: `vtkMath::LabToXYZ`.
pub fn lab_to_xyz(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let mut var_y = (l + 16.0) / 116.0;
    let mut var_x = a / 500.0 + var_y;
    let mut var_z = var_y - b / 200.0;

    if var_y.powi(3) > 0.008856 {
        var_y = var_y.powi(3);
    } else {
        var_y = (var_y - 16.0 / 116.0) / 7.787;
    }

    if var_x.powi(3) > 0.008856 {
        var_x = var_x.powi(3);
    } else {
        var_x = (var_x - 16.0 / 116.0) / 7.787;
    }

    if var_z.powi(3) > 0.008856 {
        var_z = var_z.powi(3);
    } else {
        var_z = (var_z - 16.0 / 116.0) / 7.787;
    }

    let ref_x = 0.9505;
    let ref_y = 1.000;
    let ref_z = 1.089;
    (ref_x * var_x, ref_y * var_y, ref_z * var_z)
}

/// VTK: `vtkMath::XYZToLab`.
pub fn xyz_to_lab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let ref_x = 0.9505;
    let ref_y = 1.000;
    let ref_z = 1.089;
    let mut var_x = x / ref_x;
    let mut var_y = y / ref_y;
    let mut var_z = z / ref_z;

    if var_x > 0.008856 {
        var_x = var_x.powf(1.0 / 3.0);
    } else {
        var_x = (7.787 * var_x) + (16.0 / 116.0);
    }
    if var_y > 0.008856 {
        var_y = var_y.powf(1.0 / 3.0);
    } else {
        var_y = (7.787 * var_y) + (16.0 / 116.0);
    }
    if var_z > 0.008856 {
        var_z = var_z.powf(1.0 / 3.0);
    } else {
        var_z = (7.787 * var_z) + (16.0 / 116.0);
    }

    (
        (116.0 * var_y) - 16.0,
        500.0 * (var_x - var_y),
        200.0 * (var_y - var_z),
    )
}

/// VTK: `vtkMath::XYZToRGB`.
pub fn xyz_to_rgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let mut r = x * 3.2406 + y * -1.5372 + z * -0.4986;
    let mut g = x * -0.9689 + y * 1.8758 + z * 0.0415;
    let mut b = x * 0.0557 + y * -0.2040 + z * 1.0570;

    if r > 0.0031308 {
        r = 1.055 * r.powf(1.0 / 2.4) - 0.055;
    } else {
        r *= 12.92;
    }
    if g > 0.0031308 {
        g = 1.055 * g.powf(1.0 / 2.4) - 0.055;
    } else {
        g *= 12.92;
    }
    if b > 0.0031308 {
        b = 1.055 * b.powf(1.0 / 2.4) - 0.055;
    } else {
        b *= 12.92;
    }

    let max_val = r.max(g).max(b);
    if max_val > 1.0 {
        r /= max_val;
        g /= max_val;
        b /= max_val;
    }

    (r.max(0.0), g.max(0.0), b.max(0.0))
}

/// VTK: `vtkMath::RGBToXYZ`.
pub fn rgb_to_xyz(mut r: f64, mut g: f64, mut b: f64) -> (f64, f64, f64) {
    if r > 0.04045 {
        r = ((r + 0.055) / 1.055).powf(2.4);
    } else {
        r /= 12.92;
    }
    if g > 0.04045 {
        g = ((g + 0.055) / 1.055).powf(2.4);
    } else {
        g /= 12.92;
    }
    if b > 0.04045 {
        b = ((b + 0.055) / 1.055).powf(2.4);
    } else {
        b /= 12.92;
    }

    (
        r * 0.4124 + g * 0.3576 + b * 0.1805,
        r * 0.2126 + g * 0.7152 + b * 0.0722,
        r * 0.0193 + g * 0.1192 + b * 0.9505,
    )
}

/// VTK: `vtkMath::RGBToProLab`.
pub fn rgb_to_pro_lab(red: f64, green: f64, blue: f64) -> (f64, f64, f64) {
    let (x, y, z) = rgb_to_xyz(red, green, blue);
    xyz_to_pro_lab(x, y, z)
}

/// VTK: `vtkMath::ProLabToRGB`.
pub fn pro_lab_to_rgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = pro_lab_to_xyz(l, a, b);
    xyz_to_rgb(x, y, z)
}

/// VTK: `vtkMath::RGBToLab`.
pub fn rgb_to_lab(red: f64, green: f64, blue: f64) -> (f64, f64, f64) {
    let (x, y, z) = rgb_to_xyz(red, green, blue);
    xyz_to_lab(x, y, z)
}

/// VTK: `vtkMath::LabToRGB`.
pub fn lab_to_rgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = lab_to_xyz(l, a, b);
    xyz_to_rgb(x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "{a} != {b}");
    }

    #[test]
    fn vector_norms_and_distances_match_vtk_formulas() {
        assert_close(dot2d([2.0, -3.0], [4.0, 5.0]), -7.0);
        assert_close(squared_norm([2.0, -3.0, 6.0]), 49.0);
        assert_close(norm(&[2.0, -3.0, 6.0]), 7.0);
        assert_close(norm2(&[2.0, -3.0, 6.0]), 49.0);
        assert_close(norm2d([3.0, 4.0]), 5.0);
        assert_close(
            distance2_between_points([1.0, 2.0, 3.0], [4.0, 6.0, 3.0]),
            25.0,
        );
        assert_close(distance2_between_points2d([1.0, 2.0], [4.0, 6.0]), 25.0);
    }

    #[test]
    fn determinants_and_linear_solves_work() {
        assert_close(determinant2x2(1.0, 2.0, 3.0, 4.0), -2.0);
        assert_close(
            determinant3x3([[1.0, 2.0, 3.0], [0.0, 4.0, 5.0], [1.0, 0.0, 6.0]]),
            22.0,
        );
        assert_close(
            determinant3x3_from_columns([1.0, 0.0, 1.0], [2.0, 4.0, 0.0], [3.0, 5.0, 6.0]),
            22.0,
        );
        assert_close(
            determinant3x3_from_values(1.0, 0.0, 1.0, 2.0, 4.0, 0.0, 3.0, 5.0, 6.0),
            22.0,
        );

        let (ok, y2) = solve_linear_system_gepp2x2(2.0, 1.0, 1.0, -1.0, 7.0, 2.0);
        assert!(ok);
        assert_close(y2[0], 3.0);
        assert_close(y2[1], 1.0);
        assert!(!solve_linear_system_gepp2x2(1.0, 2.0, 2.0, 4.0, 1.0, 1.0).0);

        let y3 = linear_solve3x3(
            [[3.0, 2.0, -1.0], [2.0, -2.0, 4.0], [-1.0, 0.5, -1.0]],
            [1.0, -2.0, 0.0],
        );
        assert_close(y3[0], 1.0);
        assert_close(y3[1], -2.0);
        assert_close(y3[2], -2.0);
    }

    #[test]
    fn combinatorics_and_log2_helpers_match_expected_edges() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(21), i64::MAX);
        assert_eq!(binomial(5, 2), 10);
        assert_eq!(binomial(5, 3), 10);

        assert_eq!(ceil_log2(0), 0);
        assert_eq!(ceil_log2(1), 0);
        assert_eq!(ceil_log2(2), 1);
        assert_eq!(ceil_log2(3), 2);
        assert_eq!(ceil_log2(8), 3);
        assert_eq!(floor_log2(0), None);
        assert_eq!(floor_log2(9), Some(3));
        assert!(is_power_of_two(8));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(12));
    }

    #[test]
    fn clamp_gaussian_and_containment_helpers_match_vtk_conditions() {
        let mut values = [-2.0, 0.5, 4.0];
        clamp_values(&mut values, 3, [0.0, 1.0]);
        assert_eq!(values, [0.0, 0.5, 1.0]);
        let mut clamped = [0.0; 3];
        clamp_values_copy(&[-2.0, 0.5, 4.0], 3, [0.0, 1.0], &mut clamped);
        assert_eq!(clamped, [0.0, 0.5, 1.0]);

        assert_close(gaussian_weight(4.0, 2.0), (-0.5f64).exp());
        assert_close(
            gaussian_amplitude(4.0, 0.0),
            1.0 / (8.0 * std::f64::consts::PI).sqrt(),
        );

        assert!(extent_is_within_other_extent(
            [1, 3, 2, 4, 0, 0],
            [0, 5, 0, 5, 0, 1]
        ));
        assert!(!extent_is_within_other_extent(
            [1, 6, 2, 4, 0, 0],
            [0, 5, 0, 5, 0, 1]
        ));
        assert!(bounds_is_within_other_bounds(
            [1.0, 3.0, 2.0, 4.0, 0.0, 0.5],
            [0.0, 5.0, 0.0, 5.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
        ));
        assert!(point_is_within_bounds(
            [1.0, 2.0, 3.0],
            [0.0, 1.5, 0.0, 2.5, 0.0, 3.5],
            [0.0, 0.0, 0.0],
        ));
        assert!(!point_is_within_bounds(
            [1.0, 2.0, 4.0],
            [0.0, 1.5, 0.0, 2.5, 0.0, 3.5],
            [0.0, 0.0, 0.0],
        ));
    }
}
