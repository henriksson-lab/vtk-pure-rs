use crate::data::{AnyDataArray, DataArray, DataSet, PolyData};

/// Dice a PolyData into spatial regions.
///
/// Divides the bounding box into a grid of `nx × ny × nz` bins and assigns
/// each cell to a region based on its centroid location. Adds a "RegionId"
/// cell data array indicating which region each cell belongs to.
pub fn dicer(input: &PolyData, nx: usize, ny: usize, nz: usize) -> PolyData {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let nz = nz.max(1);

    let bb = input.bounds();
    let x_min = bb.x_min;
    let y_min = bb.y_min;
    let z_min = bb.z_min;
    let dx = (bb.x_max - x_min).max(1e-15) / nx as f64;
    let dy = (bb.y_max - y_min).max(1e-15) / ny as f64;
    let dz = (bb.z_max - z_min).max(1e-15) / nz as f64;

    let mut region_ids: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            region_ids.push(0.0);
            continue;
        }

        // Compute centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in cell.iter() {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;

        let ix = ((cx - x_min) / dx).floor() as usize;
        let iy = ((cy - y_min) / dy).floor() as usize;
        let iz = ((cz - z_min) / dz).floor() as usize;
        let ix = ix.min(nx - 1);
        let iy = iy.min(ny - 1);
        let iz = iz.min(nz - 1);

        let region = iz * ny * nx + iy * nx + ix;
        region_ids.push(region as f64);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionId", region_ids, 1,
        )));
    pd
}

/// Dice into approximately `num_pieces` regions (auto-computed grid).
pub fn dicer_auto(input: &PolyData, num_pieces: usize) -> PolyData {
    let num_pieces = num_pieces.max(1);
    let n = (num_pieces as f64).cbrt().ceil() as usize;
    dicer(input, n, n, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_into_quadrants() {
        let mut pd = PolyData::new();
        // 4 triangles in different quadrants
        // Bottom-left
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.4, 0.0, 0.0]);
        pd.points.push([0.2, 0.4, 0.0]);
        // Bottom-right
        pd.points.push([0.6, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.8, 0.4, 0.0]);
        // Top-left
        pd.points.push([0.0, 0.6, 0.0]);
        pd.points.push([0.4, 0.6, 0.0]);
        pd.points.push([0.2, 1.0, 0.0]);
        // Top-right
        pd.points.push([0.6, 0.6, 0.0]);
        pd.points.push([1.0, 0.6, 0.0]);
        pd.points.push([0.8, 1.0, 0.0]);

        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[6, 7, 8]);
        pd.polys.push_cell(&[9, 10, 11]);

        let result = dicer(&pd, 2, 2, 1);
        let arr = result.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];

        // Each triangle should be in a different region
        let mut ids = Vec::new();
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            ids.push(buf[0] as usize);
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn dice_auto() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = dicer_auto(&pd, 8);
        assert!(result.cell_data().get_array("RegionId").is_some());
    }

    #[test]
    fn single_region() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = dicer(&pd, 1, 1, 1);
        let arr = result.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
