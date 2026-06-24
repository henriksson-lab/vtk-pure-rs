//! Compute face area and assign as cell data.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn face_area(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut areas = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            areas.push(0.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            areas.push(0.0);
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
        areas.push(0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt());
        // For quads or higher, approximate as sum of triangles from first vertex
        for k in 3..cell.len() {
            let bk = cell[k - 1] as usize;
            let ck = cell[k] as usize;
            if bk < n && ck < n {
                let pb2 = mesh.points.get(bk);
                let pc2 = mesh.points.get(ck);
                let u2 = [pb2[0] - pa[0], pb2[1] - pa[1], pb2[2] - pa[2]];
                let v2 = [pc2[0] - pa[0], pc2[1] - pa[1], pc2[2] - pa[2]];
                let cr2 = [
                    u2[1] * v2[2] - u2[2] * v2[1],
                    u2[2] * v2[0] - u2[0] * v2[2],
                    u2[0] * v2[1] - u2[1] * v2[0],
                ];
                let last = areas.last_mut().unwrap();
                *last += 0.5 * (cr2[0] * cr2[0] + cr2[1] * cr2[1] + cr2[2] * cr2[2]).sqrt();
            }
        }
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("FaceArea", areas, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_face_area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_area(&mesh);
        let arr = r.cell_data().get_array("FaceArea").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 0.5).abs() < 1e-9);
    }
}
