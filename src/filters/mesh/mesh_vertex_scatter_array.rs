//! Scatter/jitter vertex positions by array values.
use crate::data::PolyData;
pub fn scatter_by_array(mesh: &PolyData, array_name: &str, scale: f64, seed: u64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let nt = n.min(arr.num_tuples());
    let mut rng = seed;
    let mut buf = [0.0f64];
    let mut r = mesh.clone();
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        let amp = buf[0].abs() * scale;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let dx = (rand_unit(rng) * 2.0 - 1.0) * amp;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let dy = (rand_unit(rng) * 2.0 - 1.0) * amp;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let dz = (rand_unit(rng) * 2.0 - 1.0) * amp;
        let p = r.points.get(i);
        r.points.set(i, [p[0] + dx, p[1] + dy, p[2] + dz]);
    }
    r
}
pub fn scatter_along_normals_by_array(
    mesh: &PolyData,
    array_name: &str,
    scale: f64,
    seed: u64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let nt = n.min(arr.num_tuples());
    let mut rng = seed;
    let mut buf = [0.0f64];
    let mut r = mesh.clone();
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        let amp = buf[0] * scale;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let t = (rand_unit(rng) * 2.0 - 1.0) * amp;
        let p = r.points.get(i);
        r.points.set(
            i,
            [
                p[0] + t * nm[i][0],
                p[1] + t * nm[i][1],
                p[2] + t * nm[i][2],
            ],
        );
    }
    r
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        accumulate_cell_normal(mesh, cell, &mut nm);
    }
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                accumulate_cell_normal(mesh, &[tri[0], tri[1], tri[2]], &mut nm);
            } else {
                accumulate_cell_normal(mesh, &[tri[1], tri[0], tri[2]], &mut nm);
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
fn accumulate_cell_normal(mesh: &PolyData, cell: &[i64], nm: &mut [[f64; 3]]) {
    let n = mesh.points.len();
    let mut ids = Vec::with_capacity(cell.len());
    for &v in cell {
        let Some(id) = usize::try_from(v).ok().filter(|&id| id < n) else {
            return;
        };
        ids.push(id);
    }
    if ids.len() < 3 {
        return;
    }
    let a = mesh.points.get(ids[0]);
    let b = mesh.points.get(ids[1]);
    let c = mesh.points.get(ids[2]);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let fn_ = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    for &vi in &ids {
        nm[vi][0] += fn_[0];
        nm[vi][1] += fn_[1];
        nm[vi][2] += fn_[2];
    }
}
fn rand_unit(state: u64) -> f64 {
    ((state >> 33) as f64) / ((1u64 << 31) - 1) as f64
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_scatter() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.1, 0.1, 0.1],
                1,
            )));
        let r = scatter_by_array(&m, "s", 1.0, 42);
        assert_eq!(r.points.len(), 3);
    }
}
