//! Compute Voronoi area (mixed area) at each vertex for accurate curvature estimation.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn voronoi_area(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut area = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell(cell, n) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            let ids = [cell[0] as usize, cell[i] as usize, cell[i + 1] as usize];
            let contrib = mixed_triangle_areas(mesh, ids);
            for j in 0..3 {
                area[ids[j]] += contrib[j];
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiArea",
            area,
            1,
        )));
    result.point_data_mut().set_active_scalars("VoronoiArea");
    result
}

fn mixed_triangle_areas(mesh: &PolyData, ids: [usize; 3]) -> [f64; 3] {
    let p = [
        mesh.points.get(ids[0]),
        mesh.points.get(ids[1]),
        mesh.points.get(ids[2]),
    ];
    let e01 = sub(p[1], p[0]);
    let e02 = sub(p[2], p[0]);
    let e12 = sub(p[2], p[1]);
    let tri_area = 0.5 * norm(cross(e01, e02));
    if tri_area <= 1e-30 {
        return [0.0; 3];
    }

    let dot_a = dot(e01, e02);
    let dot_b = dot(scale(e01, -1.0), e12);
    let dot_c = dot(scale(e02, -1.0), scale(e12, -1.0));
    if dot_a < 0.0 {
        return [tri_area * 0.5, tri_area * 0.25, tri_area * 0.25];
    }
    if dot_b < 0.0 {
        return [tri_area * 0.25, tri_area * 0.5, tri_area * 0.25];
    }
    if dot_c < 0.0 {
        return [tri_area * 0.25, tri_area * 0.25, tri_area * 0.5];
    }

    let cot_a = dot_a / (2.0 * tri_area);
    let cot_b = dot_b / (2.0 * tri_area);
    let cot_c = dot_c / (2.0 * tri_area);
    let l_ab = dot(e01, e01);
    let l_ac = dot(e02, e02);
    let l_bc = dot(e12, e12);
    [
        (l_ac * cot_b + l_ab * cot_c) / 8.0,
        (l_ab * cot_c + l_bc * cot_a) / 8.0,
        (l_ac * cot_b + l_bc * cot_a) / 8.0,
    ]
}

fn valid_cell(cell: &[i64], num_points: usize) -> bool {
    cell.len() >= 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_voronoi_area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = voronoi_area(&mesh);
        let arr = r.point_data().get_array("VoronoiArea").unwrap();
        let total: f64 = (0..3)
            .map(|i| {
                let mut b = [0.0f64];
                arr.tuple_as_f64(i, &mut b);
                b[0]
            })
            .sum();
        assert!((total - 0.5).abs() < 1e-9); // total area = 0.5
    }
}
