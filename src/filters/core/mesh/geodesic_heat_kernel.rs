use crate::data::{AnyDataArray, DataArray, PolyData};

/// Multi-scale heat trace descriptor.
///
/// For each vertex, records the trace of heat kernel at multiple time scales.
/// The trace K(x,x,t) = sum of exp(-lambda_i * t) * phi_i(x)^2
/// approximated by diffusion. Adds "HeatTrace" multi-component array.
pub fn heat_trace_descriptor(input: &PolyData, num_scales: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let ns = num_scales.max(1).min(6);

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }
    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            let a = edge[0] as usize;
            let b = edge[1] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let dt = 0.25;
    let mut desc = vec![0.0f64; n * ns];

    // For efficiency, compute heat diffusion for ALL sources simultaneously
    // using the "single step at a time" approach
    let mut heat_all: Vec<Vec<f64>> = (0..n)
        .map(|src| {
            let mut v = vec![0.0; n];
            v[src] = 1.0;
            v
        })
        .collect();

    let mut scale_idx = 0;
    let mut step = 0;
    let mut next_record = 1usize;

    let max_steps = 1 << (ns - 1);
    for s in 1..=max_steps {
        // One diffusion step for all sources
        for src in 0..n {
            let prev = heat_all[src].clone();
            for i in 0..n {
                if neighbors[i].is_empty() {
                    continue;
                }
                let avg: f64 =
                    neighbors[i].iter().map(|&j| prev[j]).sum::<f64>() / neighbors[i].len() as f64;
                heat_all[src][i] = prev[i] + dt * (avg - prev[i]);
            }
        }

        if s == next_record && scale_idx < ns {
            // Record self-heat for each vertex
            for i in 0..n {
                desc[i * ns + scale_idx] = heat_all[i][i];
            }
            scale_idx += 1;
            next_record *= 2;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HeatTrace",
            desc,
            ns,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_computed() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = heat_trace_descriptor(&pd, 2);
        let arr = result.point_data().get_array("HeatTrace").unwrap();
        assert_eq!(arr.num_components(), 2);
    }

    #[test]
    fn decreasing_trace() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = heat_trace_descriptor(&pd, 2);
        let arr = result.point_data().get_array("HeatTrace").unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] >= buf[1]); // heat trace decreases with time
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = heat_trace_descriptor(&pd, 3);
        assert_eq!(result.points.len(), 0);
    }
}
