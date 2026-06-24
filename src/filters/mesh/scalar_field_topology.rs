use crate::data::{AnyDataArray, DataArray, PolyData};

/// Classify vertices of a scalar field on a mesh as minima, maxima, or saddles.
///
/// A vertex is a local minimum if all neighbors have higher values,
/// maximum if all have lower, saddle if there are alternating higher/lower
/// sequences. Adds "CriticalType" scalar: -1=min, 1=max, 0=regular, 2=saddle.
pub fn scalar_field_critical_points(input: &PolyData, array_name: &str) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    if n == 0 {
        return input.clone();
    }

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
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut types = vec![0.0f64; n];
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let v = values[i];
        let mut n_higher = 0;
        let mut n_lower = 0;
        let mut sign_changes = 0;
        let mut prev_sign = 0i8; // -1=lower, 1=higher

        for (ni, &j) in neighbors[i].iter().enumerate() {
            let nv = values[j];
            let sign = if nv > v + 1e-15 {
                1i8
            } else if nv < v - 1e-15 {
                -1i8
            } else {
                0i8
            };
            if sign > 0 {
                n_higher += 1;
            }
            if sign < 0 {
                n_lower += 1;
            }
            if sign != 0 && prev_sign != 0 && sign != prev_sign {
                sign_changes += 1;
            }
            if sign != 0 {
                prev_sign = sign;
            }
        }

        types[i] = if n_lower == 0 && n_higher > 0 {
            -1.0
        }
        // minimum
        else if n_higher == 0 && n_lower > 0 {
            1.0
        }
        // maximum
        else if sign_changes >= 2 {
            2.0
        }
        // saddle (multiple sign changes)
        else {
            0.0
        }; // regular
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalType",
            types,
            1,
        )));
    pd
}

/// Count critical points by type.
pub fn count_critical_points(input: &PolyData, array_name: &str) -> (usize, usize, usize) {
    // (minima, maxima, saddles)
    let result = scalar_field_critical_points(input, array_name);
    let arr = match result.point_data().get_array("CriticalType") {
        Some(a) => a,
        None => return (0, 0, 0),
    };
    let mut buf = [0.0f64];
    let (mut mins, mut maxs, mut saddles) = (0, 0, 0);
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] < -0.5 {
            mins += 1;
        } else if buf[0] > 0.5 && buf[0] < 1.5 {
            maxs += 1;
        } else if buf[0] > 1.5 {
            saddles += 1;
        }
    }
    (mins, maxs, saddles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_detection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center: peak
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![10.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
                1,
            )));

        let (mins, maxs, _) = count_critical_points(&pd, "h");
        assert_eq!(maxs, 1);
    }

    #[test]
    fn valley_detection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
                1,
            )));

        let (mins, _, _) = count_critical_points(&pd, "h");
        assert_eq!(mins, 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (m, x, s) = count_critical_points(&pd, "h");
        assert_eq!(m + x + s, 0);
    }
}
