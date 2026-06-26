use crate::data::PolyData;

/// Compute the surface integral of a scalar field over the mesh.
///
/// For each triangle, computes area × average_scalar_value.
/// Returns the total integral.
pub fn surface_integral(input: &PolyData, array_name: &str) -> f64 {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0.0,
    };

    let mut total = 0.0;
    let mut buf = [0.0f64];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell.iter().all(|&id| {
            id >= 0 && (id as usize) < input.points.len() && (id as usize) < arr.num_tuples()
        }) {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let area = tri_area(v0, v1, v2);

            // Average scalar over the triangle
            arr.tuple_as_f64(cell[0] as usize, &mut buf);
            let s0 = buf[0];
            arr.tuple_as_f64(cell[i] as usize, &mut buf);
            let s1 = buf[0];
            arr.tuple_as_f64(cell[i + 1] as usize, &mut buf);
            let s2 = buf[0];
            total += area * (s0 + s1 + s2) / 3.0;
        }
    }
    total
}

/// Compute the flux of a vector field through the surface.
///
/// For each triangle, computes area × (vector · normal).
pub fn surface_flux(input: &PolyData, vector_name: &str) -> f64 {
    let arr = match input.point_data().get_array(vector_name) {
        Some(a) if a.num_components() == 3 => a,
        _ => return 0.0,
    };

    let mut total = 0.0;
    let mut buf = [0.0f64; 3];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell.iter().all(|&id| {
            id >= 0 && (id as usize) < input.points.len() && (id as usize) < arr.num_tuples()
        }) {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);

            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            // Area-weighted normal (not normalized)
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];

            // Average vector over triangle
            let mut avg = [0.0; 3];
            for &id in &[cell[0], cell[i], cell[i + 1]] {
                arr.tuple_as_f64(id as usize, &mut buf);
                avg[0] += buf[0];
                avg[1] += buf[1];
                avg[2] += buf[2];
            }
            avg[0] /= 3.0;
            avg[1] /= 3.0;
            avg[2] /= 3.0;

            // Flux = (avg_vector · area_normal) * 0.5
            total += 0.5 * (avg[0] * n[0] + avg[1] * n[1] + avg[2] * n[2]);
        }
    }
    total
}

fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn constant_scalar_integral() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let integral = surface_integral(&pd, "s");
        assert!((integral - 0.5).abs() < 1e-10); // area=0.5, scalar=1
    }

    #[test]
    fn flux_through_z_plane() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        // Uniform +Z vector field
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));

        let flux = surface_flux(&pd, "v");
        assert!((flux - 0.5).abs() < 1e-10); // area=0.5, normal=+Z, v·n=1
    }

    #[test]
    fn flux_uses_full_polygon_fan() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));

        let flux = surface_flux(&pd, "v");
        assert!((flux - 1.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert_eq!(surface_integral(&pd, "nope"), 0.0);
        assert_eq!(surface_flux(&pd, "nope"), 0.0);
    }
}
