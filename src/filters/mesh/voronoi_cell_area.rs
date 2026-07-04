//! Compute Voronoi cell area per vertex (dual area).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the Voronoi area (dual area) for each vertex.
///
/// Uses the standard mixed Voronoi area for triangle meshes. Obtuse triangles
/// contribute half the area to the obtuse vertex and a quarter to the others.
pub fn voronoi_vertex_area(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut areas = vec![0.0f64; n];

    for cell in mesh.polys.iter() {
        if !valid_cell(cell, n) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            let ids = [cell[0] as usize, cell[i] as usize, cell[i + 1] as usize];
            let contrib = mixed_triangle_areas(mesh, ids);
            for j in 0..3 {
                areas[ids[j]] += contrib[j];
            }
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiArea",
            areas,
            1,
        )));
    result.point_data_mut().set_active_scalars("VoronoiArea");
    result
}

/// Compute an area-weighted dual-region centroid approximation per vertex.
pub fn voronoi_centroids(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut weights = vec![0.0f64; n];
    let mut sums = vec![[0.0f64; 3]; n];

    for cell in mesh.polys.iter() {
        if !valid_cell(cell, n) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            let ids = [cell[0] as usize, cell[i] as usize, cell[i + 1] as usize];
            let contrib = mixed_triangle_areas(mesh, ids);
            let pa = mesh.points.get(ids[0]);
            let pb = mesh.points.get(ids[1]);
            let pc = mesh.points.get(ids[2]);
            let centroid = [
                (pa[0] + pb[0] + pc[0]) / 3.0,
                (pa[1] + pb[1] + pc[1]) / 3.0,
                (pa[2] + pb[2] + pc[2]) / 3.0,
            ];
            for j in 0..3 {
                let id = ids[j];
                weights[id] += contrib[j];
                sums[id][0] += contrib[j] * centroid[0];
                sums[id][1] += contrib[j] * centroid[1];
                sums[id][2] += contrib[j] * centroid[2];
            }
        }
    }

    let mut centroids = Vec::with_capacity(n * 3);

    for i in 0..n {
        if weights[i] <= 1e-30 {
            let p = mesh.points.get(i);
            centroids.extend_from_slice(&p);
            continue;
        }
        centroids.push(sums[i][0] / weights[i]);
        centroids.push(sums[i][1] / weights[i]);
        centroids.push(sums[i][2] / weights[i]);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiCentroid",
            centroids,
            3,
        )));
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
    let area = 0.5 * norm(cross(e01, e02));
    if area <= 1e-30 {
        return [0.0; 3];
    }

    let dot0 = dot(e01, e02);
    let dot1 = dot(scale(e01, -1.0), e12);
    let dot2 = dot(scale(e02, -1.0), scale(e12, -1.0));
    if dot0 < 0.0 {
        return [area * 0.5, area * 0.25, area * 0.25];
    }
    if dot1 < 0.0 {
        return [area * 0.25, area * 0.5, area * 0.25];
    }
    if dot2 < 0.0 {
        return [area * 0.25, area * 0.25, area * 0.5];
    }

    let cot0 = dot0 / (2.0 * area);
    let cot1 = dot1 / (2.0 * area);
    let cot2 = dot2 / (2.0 * area);
    let l01 = dot(e01, e01);
    let l02 = dot(e02, e02);
    let l12 = dot(e12, e12);
    [
        (l02 * cot1 + l01 * cot2) / 8.0,
        (l01 * cot2 + l12 * cot0) / 8.0,
        (l02 * cot1 + l12 * cot0) / 8.0,
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
    fn area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = voronoi_vertex_area(&mesh);
        let arr = result.point_data().get_array("VoronoiArea").unwrap();
        let mut buf = [0.0f64];
        let mut total = 0.0;
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            total += buf[0];
        }
        assert!((total - 0.5).abs() < 0.01); // total should equal triangle area
    }
    #[test]
    fn centroids() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = voronoi_centroids(&mesh);
        assert!(result.point_data().get_array("VoronoiCentroid").is_some());
    }
}
