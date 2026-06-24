//! Young's material interface reconstruction.
//!
//! Reconstructs the interface between two materials in a mixed cell
//! using volume fractions and interface normals (Young's method).
//! Each cell with a volume fraction between 0 and 1 produces an
//! interface plane that divides the cell.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Reconstruct material interfaces from volume fraction and normal data.
///
/// Input: PolyData with cell data arrays:
/// - `volume_fraction_name`: volume fraction per cell (0 to 1)
/// - `normal_name`: interface normal per cell (3-component)
///
/// Output: PolyData with interface line/face segments for mixed cells.
pub fn youngs_material_interface(
    input: &PolyData,
    volume_fraction_name: &str,
    normal_name: &str,
) -> PolyData {
    let vf = match input.cell_data().get_array(volume_fraction_name) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };
    let normals = match input.cell_data().get_array(normal_name) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let nc = input.polys.num_cells();
    let mut interface_points = crate::data::Points::<f64>::new();
    let mut interface_lines = crate::data::CellArray::new();
    let mut material_id = Vec::new();

    for ci in 0..nc {
        let mut frac = [0.0f64];
        vf.tuple_as_f64(ci, &mut frac);
        let f = frac[0];

        // Only process mixed cells (not fully one material)
        if f <= 0.01 || f >= 0.99 {
            continue;
        }

        let mut n = [0.0f64; 3];
        normals.tuple_as_f64(ci, &mut n);
        let nlen = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nlen < 1e-15 {
            continue;
        }
        n[0] /= nlen;
        n[1] /= nlen;
        n[2] /= nlen;

        let cell = input.polys.cell(ci);
        if cell.len() < 3 {
            continue;
        }

        // Compute cell centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &vid in cell {
            let p = input.points.get(vid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let nc_pts = cell.len() as f64;
        cx /= nc_pts;
        cy /= nc_pts;
        cz /= nc_pts;

        // Place interface plane through centroid, offset by volume fraction
        // The offset shifts the plane along the normal to match the volume fraction
        let pts: Vec<[f64; 3]> = cell.iter().map(|&v| input.points.get(v as usize)).collect();
        let projections: Vec<f64> = pts
            .iter()
            .map(|p| (p[0] - cx) * n[0] + (p[1] - cy) * n[1] + (p[2] - cz) * n[2])
            .collect();
        let min_proj = projections.iter().cloned().fold(f64::MAX, f64::min);
        let max_proj = projections.iter().cloned().fold(f64::MIN, f64::max);
        let plane_offset = min_proj + f * (max_proj - min_proj);

        // Find intersection of plane with cell edges
        let mut isect_pts = Vec::new();
        for i in 0..cell.len() {
            let j = (i + 1) % cell.len();
            let di = projections[i] - plane_offset;
            let dj = projections[j] - plane_offset;
            if (di > 0.0) != (dj > 0.0) {
                let t = di / (di - dj);
                let pi = &pts[i];
                let pj = &pts[j];
                isect_pts.push([
                    pi[0] + t * (pj[0] - pi[0]),
                    pi[1] + t * (pj[1] - pi[1]),
                    pi[2] + t * (pj[2] - pi[2]),
                ]);
            }
        }

        if isect_pts.len() >= 2 {
            let base = interface_points.len() as i64;
            let mut cell_ids = Vec::new();
            for p in &isect_pts {
                cell_ids.push(base + cell_ids.len() as i64);
                interface_points.push(*p);
            }
            interface_lines.push_cell(&cell_ids);
            material_id.push(ci as f64);
        }
    }

    let mut result = PolyData::new();
    result.points = interface_points;
    result.lines = interface_lines;
    if !material_id.is_empty() {
        result
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "SourceCellId",
                material_id,
                1,
            )));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_cell_interface() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("VF", vec![0.5], 1)));
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normal",
                vec![1.0, 0.0, 0.0],
                3,
            )));

        let result = youngs_material_interface(&pd, "VF", "Normal");
        assert!(result.points.len() >= 2, "should produce interface points");
        assert_eq!(
            result.lines.num_cells(),
            1,
            "should produce one interface line"
        );
    }

    #[test]
    fn pure_cells_no_interface() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("VF", vec![1.0], 1)));
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normal",
                vec![0.0, 0.0, 1.0],
                3,
            )));

        let result = youngs_material_interface(&pd, "VF", "Normal");
        assert_eq!(
            result.points.len(),
            0,
            "pure cell should produce no interface"
        );
    }
}
