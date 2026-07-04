//! Integrate a scalar field over the mesh surface.
use crate::data::PolyData;

pub fn surface_integral(mesh: &PolyData, scalar_name: &str) -> f64 {
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0.0,
    };
    let mut total = 0.0f64;
    let mut buf = [0.0f64];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell.iter().all(|&id| {
            id >= 0 && (id as usize) < mesh.points.len() && (id as usize) < arr.num_tuples()
        }) {
            continue;
        }
        let a = cell[0] as usize;
        let pa = mesh.points.get(a);
        for i in 1..cell.len() - 1 {
            let b = cell[i] as usize;
            let c = cell[i + 1] as usize;
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            // Average scalar over triangle vertices
            arr.tuple_as_f64(a, &mut buf);
            let sa = buf[0];
            arr.tuple_as_f64(b, &mut buf);
            let sb = buf[0];
            arr.tuple_as_f64(c, &mut buf);
            let sc = buf[0];
            total += area * (sa + sb + sc) / 3.0;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_integral() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Constant field = 2.0, area = 0.5 => integral = 1.0
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![2.0, 2.0, 2.0],
                1,
            )));
        let result = surface_integral(&mesh, "f");
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_integral_quad_uses_full_polygon() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![2.0, 2.0, 2.0, 2.0],
                1,
            )));
        let result = surface_integral(&mesh, "f");
        assert!((result - 2.0).abs() < 1e-9);
    }
}
