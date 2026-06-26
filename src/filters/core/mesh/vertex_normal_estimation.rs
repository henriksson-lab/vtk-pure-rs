//! Vertex normal estimation methods: area-weighted, angle-weighted, PCA.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Neighborhood selection mode for PCA normal estimation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PcaSearchMode {
    /// Select the closest `sample_size` points.
    Knn,
    /// Select points within `radius`, falling back to KNN if too few are found.
    Radius,
}

/// Orientation mode for PCA normal estimation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PcaNormalOrientation {
    /// Leave the eigenvector sign as produced by the eigensolver.
    None,
    /// Orient normals toward `orientation_point`, matching vtkPCANormalEstimation::POINT.
    Point([f64; 3]),
}

/// Compute vertex normals using area-weighted face normal averaging.
pub fn normals_area_weighted(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut normals = vec![[0.0; 3]; n];

    for cell in mesh.polys.iter() {
        let face_normal = polygon_normal(mesh, cell);
        if norm_squared(face_normal) == 0.0 {
            continue;
        }

        for &point_id in cell {
            let point_id = point_id as usize;
            if point_id < n {
                normals[point_id][0] += face_normal[0];
                normals[point_id][1] += face_normal[1];
                normals[point_id][2] += face_normal[2];
            }
        }
    }

    normals_to_poly_data(mesh, normals, "Normals")
}

/// Compute vertex normals using angle-weighted face normal averaging.
pub fn normals_angle_weighted(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut normals = vec![[0.0; 3]; n];

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let mut face_normal = polygon_normal(mesh, cell);
        if !normalize(&mut face_normal) {
            continue;
        }

        for point_index in 0..cell.len() {
            let point_id = cell[point_index] as usize;
            if point_id >= n {
                continue;
            }

            let previous_id = cell[(point_index + cell.len() - 1) % cell.len()] as usize;
            let next_id = cell[(point_index + 1) % cell.len()] as usize;
            let current = mesh.points.get(point_id);
            let previous = mesh.points.get(previous_id);
            let next = mesh.points.get(next_id);
            let to_previous = [
                previous[0] - current[0],
                previous[1] - current[1],
                previous[2] - current[2],
            ];
            let to_next = [
                next[0] - current[0],
                next[1] - current[1],
                next[2] - current[2],
            ];
            let angle = angle_between(to_previous, to_next);

            normals[point_id][0] += angle * face_normal[0];
            normals[point_id][1] += angle * face_normal[1];
            normals[point_id][2] += angle * face_normal[2];
        }
    }

    normals_to_poly_data(mesh, normals, "Normals")
}

/// Estimate point normals from local PCA neighborhoods.
///
/// This follows `vtkPCANormalEstimation`: each point gathers a KNN or radius
/// neighborhood, computes the covariance matrix, and uses the smallest
/// eigenvector as the point normal.
pub fn normals_pca(
    mesh: &PolyData,
    sample_size: usize,
    radius: f64,
    search_mode: PcaSearchMode,
    orientation: PcaNormalOrientation,
    flip_normals: bool,
) -> PolyData {
    let num_points = mesh.points.len();
    if num_points == 0 {
        return mesh.clone();
    }

    let sample_size = sample_size.max(1).min(num_points);
    let mut normals = vec![[0.0; 3]; num_points];

    for (point_id, normal) in normals.iter_mut().enumerate() {
        let point = mesh.points.get(point_id);
        let neighbor_ids = find_points(mesh, point, search_mode, sample_size, radius);
        if neighbor_ids.is_empty() {
            continue;
        }

        let mut mean = [0.0; 3];
        for &neighbor_id in &neighbor_ids {
            let neighbor = mesh.points.get(neighbor_id);
            mean[0] += neighbor[0];
            mean[1] += neighbor[1];
            mean[2] += neighbor[2];
        }
        let count = neighbor_ids.len() as f64;
        mean[0] /= count;
        mean[1] /= count;
        mean[2] /= count;

        let mut covariance = [[0.0; 3]; 3];
        for &neighbor_id in &neighbor_ids {
            let neighbor = mesh.points.get(neighbor_id);
            let x = [
                neighbor[0] - mean[0],
                neighbor[1] - mean[1],
                neighbor[2] - mean[2],
            ];
            for row in 0..3 {
                for col in 0..3 {
                    covariance[row][col] += x[row] * x[col];
                }
            }
        }
        for row in &mut covariance {
            for value in row {
                *value /= count;
            }
        }

        *normal = smallest_eigenvector(covariance);
        if let PcaNormalOrientation::Point(orientation_point) = orientation {
            let orientation_vector = [
                orientation_point[0] - point[0],
                orientation_point[1] - point[1],
                orientation_point[2] - point[2],
            ];
            if dot(orientation_vector, *normal) < 0.0 {
                negate(normal);
            }
        }
        if flip_normals {
            negate(normal);
        }
    }

    normals_to_poly_data(mesh, normals, "PCANormals")
}

/// Flip normals to be consistently outward-facing using a reference point.
pub fn orient_normals_outward(mesh: &PolyData, reference_point: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let normals = match mesh.point_data().normals() {
        Some(normals) => normals,
        None => return normals_area_weighted(mesh),
    };
    let mut data = Vec::with_capacity(n * 3);
    let mut normal = [0.0f64; 3];

    for point_id in 0..n {
        normals.tuple_as_f64(point_id, &mut normal);
        let point = mesh.points.get(point_id);
        let to_reference = [
            reference_point[0] - point[0],
            reference_point[1] - point[1],
            reference_point[2] - point[2],
        ];
        if dot(normal, to_reference) >= 0.0 {
            normal = [-normal[0], -normal[1], -normal[2]];
        }
        data.extend_from_slice(&normal);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    result.point_data_mut().set_active_normals("Normals");
    result
}

fn normals_to_poly_data(mesh: &PolyData, mut normals: Vec<[f64; 3]>, name: &str) -> PolyData {
    for normal in &mut normals {
        normalize(normal);
    }

    let data: Vec<f64> = normals.into_iter().flatten().collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 3)));
    result.point_data_mut().set_active_normals(name);
    result
}

fn find_points(
    mesh: &PolyData,
    point: [f64; 3],
    search_mode: PcaSearchMode,
    sample_size: usize,
    radius: f64,
) -> Vec<usize> {
    let mut distances: Vec<(f64, usize)> = (0..mesh.points.len())
        .map(|point_id| (distance_squared(point, mesh.points.get(point_id)), point_id))
        .collect();
    distances.sort_by(|a, b| a.0.total_cmp(&b.0));

    match search_mode {
        PcaSearchMode::Radius => {
            let radius_squared = radius * radius;
            let mut ids: Vec<usize> = distances
                .iter()
                .filter_map(|&(distance, point_id)| {
                    if distance <= radius_squared {
                        Some(point_id)
                    } else {
                        None
                    }
                })
                .collect();
            if ids.len() < sample_size {
                ids = distances
                    .iter()
                    .take(sample_size)
                    .map(|&(_, point_id)| point_id)
                    .collect();
            }
            ids
        }
        PcaSearchMode::Knn => {
            let mut ids: Vec<usize> = distances
                .iter()
                .take(sample_size)
                .map(|&(_, point_id)| point_id)
                .collect();
            if radius > 0.0 {
                if let Some(&(farthest_distance, _)) = distances.get(sample_size.saturating_sub(1))
                {
                    if farthest_distance < radius * radius {
                        ids = distances
                            .iter()
                            .filter_map(|&(distance, point_id)| {
                                if distance <= radius * radius {
                                    Some(point_id)
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                }
            }
            ids
        }
    }
}

fn polygon_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0; 3];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let p0 = mesh.points.get(cell[point_id] as usize);
        let p1 = mesh.points.get(cell[point_id + 1] as usize);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if norm_squared(v1) > 0.0 {
            common = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common else {
        return [0.0; 3];
    };
    if point_id >= cell.len() {
        return [0.0; 3];
    }

    let p0 = mesh.points.get(cell[common_id] as usize);
    let mut normal = [0.0; 3];
    while point_id < cell.len() {
        let p = mesh.points.get(cell[point_id] as usize);
        let v2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        normal[0] += cross[0];
        normal[1] += cross[1];
        normal[2] += cross[2];
        v1 = v2;
        point_id += 1;
    }

    normal
}

fn smallest_eigenvector(mut a: [[f64; 3]; 3]) -> [f64; 3] {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..50 {
        let mut p = 0;
        let mut q = 1;
        let mut max_value = 0.0;
        for row in 0..3 {
            for col in (row + 1)..3 {
                let value = a[row][col].abs();
                if value > max_value {
                    max_value = value;
                    p = row;
                    q = col;
                }
            }
        }
        if max_value < 1e-15 {
            break;
        }

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        let theta = if (app - aqq).abs() < 1e-20 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };
        let c = theta.cos();
        let s = theta.sin();

        let mut next = a;
        next[p][p] = c * c * app + 2.0 * c * s * apq + s * s * aqq;
        next[q][q] = s * s * app - 2.0 * c * s * apq + c * c * aqq;
        next[p][q] = 0.0;
        next[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                next[r][p] = c * arp + s * arq;
                next[p][r] = next[r][p];
                next[r][q] = -s * arp + c * arq;
                next[q][r] = next[r][q];
            }
        }
        a = next;

        for row in &mut v {
            let vp = row[p];
            let vq = row[q];
            row[p] = c * vp + s * vq;
            row[q] = -s * vp + c * vq;
        }
    }

    let mut min_index = 0;
    for i in 1..3 {
        if a[i][i] < a[min_index][min_index] {
            min_index = i;
        }
    }

    let mut normal = [v[0][min_index], v[1][min_index], v[2][min_index]];
    normalize(&mut normal);
    normal
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let denominator = norm(a) * norm(b);
    if denominator == 0.0 {
        0.0
    } else {
        (dot(a, b) / denominator).clamp(-1.0, 1.0).acos()
    }
}

fn distance_squared(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn negate(vector: &mut [f64; 3]) {
    vector[0] = -vector[0];
    vector[1] = -vector[1];
    vector[2] = -vector[2];
}

fn normalize(vector: &mut [f64; 3]) -> bool {
    let length = norm(*vector);
    if length > 0.0 {
        vector[0] /= length;
        vector[1] /= length;
        vector[2] /= length;
        true
    } else {
        false
    }
}

fn norm(vector: [f64; 3]) -> f64 {
    norm_squared(vector).sqrt()
}

fn norm_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = normals_area_weighted(&mesh);
        assert!(result.point_data().normals().is_some());
        let arr = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.99);
    }

    #[test]
    fn angle_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = normals_angle_weighted(&mesh);
        assert!(result.point_data().normals().is_some());
    }

    #[test]
    fn pca_normals_on_plane() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ]);

        let result = normals_pca(
            &mesh,
            5,
            0.0,
            PcaSearchMode::Knn,
            PcaNormalOrientation::None,
            false,
        );
        let normals = result.point_data().normals().unwrap();
        assert_eq!(normals.num_tuples(), 5);
        let mut normal = [0.0; 3];
        normals.tuple_as_f64(0, &mut normal);
        assert!(normal[2].abs() > 0.99, "normal = {:?}", normal);
    }

    #[test]
    fn pca_orients_toward_point_then_flips() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ]);

        let result = normals_pca(
            &mesh,
            5,
            0.0,
            PcaSearchMode::Knn,
            PcaNormalOrientation::Point([0.5, 0.5, 1.0]),
            true,
        );
        let normals = result.point_data().normals().unwrap();
        let mut normal = [0.0; 3];
        normals.tuple_as_f64(0, &mut normal);
        assert!(normal[2] < -0.99, "normal = {:?}", normal);
    }

    #[test]
    fn orient() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh = normals_area_weighted(&mesh);
        let result = orient_normals_outward(&mesh, [0.5, 0.3, -1.0]);
        let arr = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.0);
    }

    #[test]
    fn polygon_normals_skip_initial_collinear_vertices() {
        let mesh = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3, 4]],
        );
        let result = normals_area_weighted(&mesh);
        let normals = result.point_data().normals().unwrap();
        let mut normal = [0.0; 3];
        normals.tuple_as_f64(0, &mut normal);
        assert!(normal[2] > 0.99, "normal = {:?}", normal);
    }
}
