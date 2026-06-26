use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate random scalar values as point data.
///
/// Adds a scalar array with values uniformly distributed in `[min_val, max_val]`.
pub fn random_point_scalars(
    input: &PolyData,
    name: &str,
    min_val: f64,
    max_val: f64,
    seed: u64,
) -> PolyData {
    let n = input.points.len();
    let mut state = seed_state(seed);
    let range = max_val - min_val;

    let values: Vec<f64> = (0..n)
        .map(|_| min_val + range * next_random(&mut state))
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, values, 1)));
    pd.point_data_mut().set_active_scalars(name);
    pd
}

/// Generate random vector values (3-component) as point data.
pub fn random_point_vectors(
    input: &PolyData,
    name: &str,
    min_val: f64,
    max_val: f64,
    seed: u64,
) -> PolyData {
    let n = input.points.len();
    let mut state = seed_state(seed);
    let range = max_val - min_val;

    let values: Vec<f64> = (0..n * 3)
        .map(|_| min_val + range * next_random(&mut state))
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, values, 3)));
    pd
}

/// Generate random cell scalars.
pub fn random_cell_scalars(
    input: &PolyData,
    name: &str,
    min_val: f64,
    max_val: f64,
    seed: u64,
) -> PolyData {
    let n = input.total_cells();
    let mut state = seed_state(seed);
    let range = max_val - min_val;

    let values: Vec<f64> = (0..n)
        .map(|_| min_val + range * next_random(&mut state))
        .collect();

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, values, 1)));
    pd.cell_data_mut().set_active_scalars(name);
    pd
}

fn seed_state(seed: u64) -> u64 {
    if seed == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        seed
    }
}

fn next_random(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state & 0xFFFFFFFF) as f64 / 0xFFFFFFFF_u64 as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_scalars() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = random_point_scalars(&pd, "noise", 0.0, 1.0, 42);
        let arr = result.point_data().get_array("noise").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!(val[0] >= 0.0 && val[0] <= 1.0);
    }

    #[test]
    fn random_vectors() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = random_point_vectors(&pd, "vel", -1.0, 1.0, 99);
        let arr = result.point_data().get_array("vel").unwrap();
        assert_eq!(arr.num_components(), 3);
        assert_eq!(arr.num_tuples(), 2);
    }

    #[test]
    fn random_cell_data() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = random_cell_scalars(&pd, "cval", 0.0, 100.0, 7);
        let arr = result.cell_data().get_array("cval").unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }
}
