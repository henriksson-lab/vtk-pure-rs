use crate::data::{CellArray, Points, PolyData};

/// Explode mesh cells outward from the mesh centroid.
///
/// Each cell's vertices are displaced away from the overall centroid
/// by `factor` (0 = no displacement, 1 = fully separated).
/// Creates disconnected cells for visualization.
pub fn explode(input: &PolyData, factor: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Global centroid
    let mut gcx = 0.0;
    let mut gcy = 0.0;
    let mut gcz = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        gcx += p[0];
        gcy += p[1];
        gcz += p[2];
    }
    let nf = n as f64;
    gcx /= nf;
    gcy /= nf;
    gcz /= nf;

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        // Cell centroid
        let mut ccx = 0.0;
        let mut ccy = 0.0;
        let mut ccz = 0.0;
        for &id in cell.iter() {
            let p = input.points.get(id as usize);
            ccx += p[0];
            ccy += p[1];
            ccz += p[2];
        }
        let cn = cell.len() as f64;
        ccx /= cn;
        ccy /= cn;
        ccz /= cn;

        // Displacement from global centroid to cell centroid
        let dx = (ccx - gcx) * factor;
        let dy = (ccy - gcy) * factor;
        let dz = (ccz - gcz) * factor;

        let base = out_points.len() as i64;
        let mut ids = Vec::with_capacity(cell.len());
        for &pid in cell.iter() {
            let p = input.points.get(pid as usize);
            out_points.push([p[0] + dx, p[1] + dy, p[2] + dz]);
            ids.push(base + ids.len() as i64);
        }
        out_polys.push_cell(&ids);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explode_separates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);

        let result = explode(&pd, 1.0);
        assert_eq!(result.points.len(), 6); // separated
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn zero_factor_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = explode(&pd, 0.0);
        // With factor=0, points don't move (but are still duplicated)
        let p = result.points.get(0);
        assert!((p[0]).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = explode(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
