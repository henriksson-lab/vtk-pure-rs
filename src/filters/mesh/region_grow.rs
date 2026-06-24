use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashSet, VecDeque};

/// Region growing on a mesh: expand from seed vertices to neighbors
/// that satisfy a scalar predicate.
///
/// Starting from `seeds`, adds neighbors where `array_name` value
/// is within [min_val, max_val]. Adds "RegionMask" scalar (1=selected, 0=not).
pub fn region_grow(
    input: &PolyData,
    seeds: &[usize],
    array_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

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

    let mut buf = [0.0f64];
    let mut selected = HashSet::new();
    let mut queue = VecDeque::new();

    for &s in seeds {
        if s < n {
            arr.tuple_as_f64(s, &mut buf);
            if buf[0] >= min_val && buf[0] <= max_val {
                selected.insert(s);
                queue.push_back(s);
            }
        }
    }

    while let Some(v) = queue.pop_front() {
        for &nb in &neighbors[v] {
            if selected.contains(&nb) {
                continue;
            }
            arr.tuple_as_f64(nb, &mut buf);
            if buf[0] >= min_val && buf[0] <= max_val {
                selected.insert(nb);
                queue.push_back(nb);
            }
        }
    }

    let mask: Vec<f64> = (0..n)
        .map(|i| if selected.contains(&i) { 1.0 } else { 0.0 })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionMask",
            mask,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grow_from_seed() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 1.0, 1.0, 1.0, 5.0],
                1,
            )));

        let result = region_grow(&pd, &[0], "val", 0.5, 1.5);
        let arr = result.point_data().get_array("RegionMask").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0); // value=5 outside range
    }

    #[test]
    fn no_valid_seed() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![10.0], 1)));

        let result = region_grow(&pd, &[0], "v", 0.0, 1.0);
        let arr = result.point_data().get_array("RegionMask").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // seed outside range
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = region_grow(&pd, &[0], "v", 0.0, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
