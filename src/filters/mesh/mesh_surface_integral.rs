//! Integrate a scalar field over the mesh surface.
use crate::data::PolyData;

pub fn surface_integral(mesh: &PolyData, scalar_name: &str) -> f64 {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return 0.0,
    };
    let mut total = 0.0f64;
    let mut buf = [0.0f64];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        // Average scalar over triangle vertices
        arr.tuple_as_f64(a, &mut buf);
        let sa = buf[0];
        arr.tuple_as_f64(b, &mut buf);
        let sb = buf[0];
        arr.tuple_as_f64(c, &mut buf);
        let sc = buf[0];
        total += area * (sa + sb + sc) / 3.0;
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
}
