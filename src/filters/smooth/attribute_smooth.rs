use crate::data::{AnyDataArray, DataArray, PolyData};

const VTK_DEFAULT_RELAXATION_FACTOR: f64 = 0.10;

/// Smooth point data attributes over the mesh connectivity.
///
/// For each point, updates the named array with VTK's default
/// `vtkAttributeSmoothingFilter` stencil:
/// `(1 - R) * a(i) + R * sum(w(j) * a(j))`, where `R = 0.10` and
/// the normalized weights use VTK's default inverse squared edge distance.
pub fn attribute_smooth(input: &PolyData, array_name: &str, iterations: usize) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    if iterations == 0 || n == 0 || arr.num_tuples() != n {
        return input.clone();
    }

    let num_comp = arr.num_components();
    let mut values = vec![0.0f64; n * num_comp];
    let mut buf = vec![0.0f64; num_comp];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        for c in 0..num_comp {
            values[i * num_comp + c] = buf[c];
        }
    }

    let stencils = build_weighted_stencils(input, n);

    for _ in 0..iterations {
        let mut new_values = values.clone();
        for i in 0..n {
            if stencils[i].is_empty() {
                continue;
            }
            for c in 0..num_comp {
                let mut weighted_sum = 0.0;
                for &(j, weight) in &stencils[i] {
                    weighted_sum += weight * values[j * num_comp + c];
                }
                let old = values[i * num_comp + c];
                new_values[i * num_comp + c] =
                    (1.0 - VTK_DEFAULT_RELAXATION_FACTOR) * old + weighted_sum;
            }
        }
        values = new_values;
    }

    let mut pd = input.clone();
    let smoothed = AnyDataArray::F64(DataArray::from_vec(array_name, values, num_comp));

    pd.point_data_mut().add_array(smoothed);
    pd
}

fn build_weighted_stencils(input: &PolyData, n: usize) -> Vec<Vec<(usize, f64)>> {
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        add_closed_cell_edges(cell, n, &mut neighbors);
    }
    for cell in input.lines.iter() {
        add_open_cell_edges(cell, n, &mut neighbors);
    }

    neighbors
        .into_iter()
        .enumerate()
        .map(|(i, mut nbrs)| {
            nbrs.sort_unstable();
            nbrs.dedup();
            let p = input.points.get(i);
            let mut force_weight = None;
            let mut weights = Vec::with_capacity(nbrs.len());
            let mut weight_sum = 0.0;
            for (k, j) in nbrs.iter().copied().enumerate() {
                let q = input.points.get(j);
                let d2 = distance2(p, q);
                if d2 == 0.0 {
                    force_weight = Some(k);
                    weights.push((j, 0.0));
                } else {
                    let w = 1.0 / d2;
                    weight_sum += w;
                    weights.push((j, w));
                }
            }
            if let Some(k) = force_weight {
                for (_, w) in &mut weights {
                    *w = 0.0;
                }
                weights[k].1 = VTK_DEFAULT_RELAXATION_FACTOR;
            } else if weight_sum > 0.0 {
                let f = VTK_DEFAULT_RELAXATION_FACTOR / weight_sum;
                for (_, w) in &mut weights {
                    *w *= f;
                }
            }
            weights
        })
        .collect()
}

fn add_closed_cell_edges(cell: &[i64], n: usize, neighbors: &mut [Vec<usize>]) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        add_edge(cell[i], cell[(i + 1) % cell.len()], n, neighbors);
    }
}

fn add_open_cell_edges(cell: &[i64], n: usize, neighbors: &mut [Vec<usize>]) {
    for edge in cell.windows(2) {
        add_edge(edge[0], edge[1], n, neighbors);
    }
}

fn add_edge(a: i64, b: i64, n: usize, neighbors: &mut [Vec<usize>]) {
    let (Some(a), Some(b)) = (valid_point_id(a, n), valid_point_id(b, n)) else {
        return;
    };
    if a != b {
        neighbors[a].push(b);
        neighbors[b].push(a);
    }
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_spike() {
        let mut pd = PolyData::new();
        // Triangle with center spike value
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![0.0, 0.0, 10.0],
                1,
            )));

        let result = attribute_smooth(&pd, "temp", 1);
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];

        // VTK default R=0.10: new value = 0.9*10 + 0.1*0.
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn zero_iterations() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let result = attribute_smooth(&pd, "val", 0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 3.0);
    }

    #[test]
    fn missing_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = attribute_smooth(&pd, "nonexistent", 5);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn convergence() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 9.0],
                1,
            )));

        // Many iterations should converge to a constant value. With VTK's
        // default distance-squared weights, this is not necessarily the
        // unweighted arithmetic mean.
        let result = attribute_smooth(&pd, "v", 100);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let expected = buf[0];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - expected).abs() < 0.01, "val[{}]={}", i, buf[0]);
        }
    }
}
