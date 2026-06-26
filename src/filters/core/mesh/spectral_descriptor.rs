use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute a spectral shape descriptor by running heat diffusion at multiple scales.
///
/// For each vertex, records the heat remaining after diffusion at
/// t=1,2,4,8,... time scales. Adds "SpectralDesc" multi-component array.
pub fn spectral_descriptor(input: &PolyData, num_scales: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let ns = num_scales.max(1).min(8);

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % cell.len()], n) else {
                continue;
            };
            if a == b {
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

    let dt = 0.3;
    let mut desc = vec![0.0f64; n * ns];

    for src in 0..n {
        let mut heat = vec![0.0f64; n];
        heat[src] = 1.0;
        let mut scale_idx = 0;
        let mut next_record = 1usize;

        for step in 1..=(1 << (ns - 1)) {
            let mut new = heat.clone();
            for i in 0..n {
                if neighbors[i].is_empty() {
                    continue;
                }
                let avg: f64 =
                    neighbors[i].iter().map(|&j| heat[j]).sum::<f64>() / neighbors[i].len() as f64;
                new[i] = heat[i] + dt * (avg - heat[i]);
            }
            heat = new;

            if step == next_record && scale_idx < ns {
                desc[src * ns + scale_idx] = heat[src];
                scale_idx += 1;
                next_record *= 2;
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SpectralDesc",
            desc,
            ns,
        )));
    pd
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&idx| idx < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = spectral_descriptor(&pd, 3);
        let arr = result.point_data().get_array("SpectralDesc").unwrap();
        assert_eq!(arr.num_components(), 3);
    }

    #[test]
    fn decreasing_with_scale() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = spectral_descriptor(&pd, 2);
        let arr = result.point_data().get_array("SpectralDesc").unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        // Heat at source should decrease with time
        assert!(buf[0] >= buf[1]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = spectral_descriptor(&pd, 3);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn skips_invalid_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 99]);
        pd.polys.push_cell(&[-1, 0]);

        let result = spectral_descriptor(&pd, 2);
        let arr = result.point_data().get_array("SpectralDesc").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        assert_eq!(arr.num_components(), 2);
    }
}
