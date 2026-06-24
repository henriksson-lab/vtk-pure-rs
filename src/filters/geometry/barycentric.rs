use crate::data::{AnyDataArray, CellLocator, DataArray, PolyData};

/// Compute barycentric coordinates for probe points relative to a triangle mesh.
///
/// For each probe point, finds the nearest triangle and computes the
/// barycentric coordinates (u, v, w) within that triangle.
/// Adds "Barycentric" (3-component) and "NearestCell" (1-component) arrays.
pub fn barycentric_coordinates(surface: &PolyData, probe: &PolyData) -> PolyData {
    let locator = CellLocator::build(surface);
    let n = probe.points.len();

    let mut bary = Vec::with_capacity(n * 3);
    let mut cell_ids = Vec::with_capacity(n);

    // Collect triangles for barycentric computation
    let tris: Vec<[[f64; 3]; 3]> = surface
        .polys
        .iter()
        .filter_map(|cell| {
            if cell.len() >= 3 {
                Some([
                    surface.points.get(cell[0] as usize),
                    surface.points.get(cell[1] as usize),
                    surface.points.get(cell[2] as usize),
                ])
            } else {
                None
            }
        })
        .collect();

    for i in 0..n {
        let p = probe.points.get(i);
        if let Some((cell_id, closest_pt, _)) = locator.find_closest_cell(p) {
            cell_ids.push(cell_id as f64);
            if cell_id < tris.len() {
                let (u, v, w) = compute_bary(closest_pt, &tris[cell_id]);
                bary.push(u);
                bary.push(v);
                bary.push(w);
            } else {
                bary.push(1.0 / 3.0);
                bary.push(1.0 / 3.0);
                bary.push(1.0 / 3.0);
            }
        } else {
            cell_ids.push(-1.0);
            bary.push(0.0);
            bary.push(0.0);
            bary.push(0.0);
        }
    }

    let mut pd = probe.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Barycentric",
            bary,
            3,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NearestCell",
            cell_ids,
            1,
        )));
    pd
}

fn compute_bary(p: [f64; 3], tri: &[[f64; 3]; 3]) -> (f64, f64, f64) {
    let v0 = [
        tri[1][0] - tri[0][0],
        tri[1][1] - tri[0][1],
        tri[1][2] - tri[0][2],
    ];
    let v1 = [
        tri[2][0] - tri[0][0],
        tri[2][1] - tri[0][1],
        tri[2][2] - tri[0][2],
    ];
    let v2 = [p[0] - tri[0][0], p[1] - tri[0][1], p[2] - tri[0][2]];

    let d00 = v0[0] * v0[0] + v0[1] * v0[1] + v0[2] * v0[2];
    let d01 = v0[0] * v1[0] + v0[1] * v1[1] + v0[2] * v1[2];
    let d11 = v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2];
    let d20 = v2[0] * v0[0] + v2[1] * v0[1] + v2[2] * v0[2];
    let d21 = v2[0] * v1[0] + v2[1] * v1[1] + v2[2] * v1[2];

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-15 {
        return (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
    }

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    (u, v, w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centroid_coords() {
        let mut surface = PolyData::new();
        surface.points.push([0.0, 0.0, 0.0]);
        surface.points.push([3.0, 0.0, 0.0]);
        surface.points.push([0.0, 3.0, 0.0]);
        surface.polys.push_cell(&[0, 1, 2]);

        let mut probe = PolyData::new();
        probe.points.push([1.0, 1.0, 0.0]); // near centroid

        let result = barycentric_coordinates(&surface, &probe);
        let arr = result.point_data().get_array("Barycentric").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        // Near centroid: u,v,w should all be ~1/3
        assert!((buf[0] - 1.0 / 3.0).abs() < 0.2);
        assert!((buf[1] - 1.0 / 3.0).abs() < 0.2);
    }

    #[test]
    fn has_cell_ids() {
        let mut surface = PolyData::new();
        surface.points.push([0.0, 0.0, 0.0]);
        surface.points.push([1.0, 0.0, 0.0]);
        surface.points.push([0.0, 1.0, 0.0]);
        surface.polys.push_cell(&[0, 1, 2]);

        let mut probe = PolyData::new();
        probe.points.push([0.3, 0.3, 0.0]);

        let result = barycentric_coordinates(&surface, &probe);
        assert!(result.point_data().get_array("NearestCell").is_some());
    }

    #[test]
    fn empty_surface() {
        let surface = PolyData::new();
        let mut probe = PolyData::new();
        probe.points.push([0.0, 0.0, 0.0]);

        let result = barycentric_coordinates(&surface, &probe);
        let arr = result.point_data().get_array("NearestCell").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], -1.0);
    }
}
