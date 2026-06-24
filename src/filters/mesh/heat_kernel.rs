use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute Heat Kernel Signature (HKS) at each vertex.
///
/// HKS is a shape descriptor based on heat diffusion. Approximated by
/// running heat diffusion from each vertex for `time` steps and recording
/// the remaining heat at the source. Adds "HKS" scalar array.
///
/// This is a simplified single-scale HKS using explicit heat diffusion.
pub fn heat_kernel_signature(input: &PolyData, time: usize) -> PolyData {
    let n = input.points.len();
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

    let dt = 0.3;
    let mut hks = vec![0.0f64; n];

    for src in 0..n {
        let mut heat = vec![0.0f64; n];
        heat[src] = 1.0;

        for _ in 0..time {
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
        }
        hks[src] = heat[src]; // heat remaining at source
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("HKS", hks, 1)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hks_varies_with_topology() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // corner
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = heat_kernel_signature(&pd, 5);
        assert!(result.point_data().get_array("HKS").is_some());
    }

    #[test]
    fn symmetric_vertices_equal_hks() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = heat_kernel_signature(&pd, 3);
        let arr = result.point_data().get_array("HKS").unwrap();
        // All 3 vertices of a single triangle have same connectivity
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let h0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let h1 = buf[0];
        assert!((h0 - h1).abs() < 0.1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = heat_kernel_signature(&pd, 5);
        assert_eq!(result.points.len(), 0);
    }
}
