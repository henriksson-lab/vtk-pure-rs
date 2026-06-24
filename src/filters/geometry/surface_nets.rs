use crate::data::{CellArray, ImageData, Points, PolyData};
use std::collections::HashMap;

/// Surface Nets isosurface extraction — produces smoother meshes than marching cubes.
///
/// For each cell in the ImageData that straddles the isovalue, places a vertex
/// at the average of the edge intersection points, then connects adjacent cells
/// with quad faces. The result is a quad-dominant mesh without the staircase
/// artifacts of marching cubes.
pub fn surface_nets(input: &ImageData, scalars: &str, isovalue: f64) -> PolyData {
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;

    if nx < 2 || ny < 2 || nz < 2 {
        return PolyData::new();
    }

    let scalar_arr = match input.point_data().get_array(scalars) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let origin = input.origin();
    let spacing = input.spacing();

    // Read scalar values
    let n_pts = nx * ny * nz;
    let mut values = vec![0.0f64; n_pts];
    let mut buf = [0.0f64];
    for (i, v) in values.iter_mut().enumerate() {
        scalar_arr.tuple_as_f64(i, &mut buf);
        *v = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let point_at = |i: usize, j: usize, k: usize| -> [f64; 3] {
        [
            origin[0] + i as f64 * spacing[0],
            origin[1] + j as f64 * spacing[1],
            origin[2] + k as f64 * spacing[2],
        ]
    };

    // For each cell, determine if it straddles the isosurface
    // Cell (i,j,k) has corners at (i,j,k), (i+1,j,k), ..., (i+1,j+1,k+1)
    let ncx = nx - 1;
    let ncy = ny - 1;
    let ncz = nz - 1;

    let cell_idx = |i: usize, j: usize, k: usize| -> usize { k * ncy * ncx + j * ncx + i };

    // Map from cell index to output vertex index
    let mut cell_vertex: HashMap<usize, usize> = HashMap::new();
    let mut out_points = Points::<f64>::new();

    for k in 0..ncz {
        for j in 0..ncy {
            for i in 0..ncx {
                // 8 corner values
                let corners = [
                    values[idx(i, j, k)],
                    values[idx(i + 1, j, k)],
                    values[idx(i + 1, j + 1, k)],
                    values[idx(i, j + 1, k)],
                    values[idx(i, j, k + 1)],
                    values[idx(i + 1, j, k + 1)],
                    values[idx(i + 1, j + 1, k + 1)],
                    values[idx(i, j + 1, k + 1)],
                ];

                // Check if cell straddles isovalue
                let above = corners.iter().filter(|&&v| v >= isovalue).count();
                if above == 0 || above == 8 {
                    continue;
                }

                // Corner positions
                let corner_pts = [
                    point_at(i, j, k),
                    point_at(i + 1, j, k),
                    point_at(i + 1, j + 1, k),
                    point_at(i, j + 1, k),
                    point_at(i, j, k + 1),
                    point_at(i + 1, j, k + 1),
                    point_at(i + 1, j + 1, k + 1),
                    point_at(i, j + 1, k + 1),
                ];

                // 12 edges of the cube
                let edges: [(usize, usize); 12] = [
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (3, 0),
                    (4, 5),
                    (5, 6),
                    (6, 7),
                    (7, 4),
                    (0, 4),
                    (1, 5),
                    (2, 6),
                    (3, 7),
                ];

                // Find average of edge intersection points
                let mut sum = [0.0f64; 3];
                let mut count = 0;
                for &(a, b) in &edges {
                    let va = corners[a];
                    let vb = corners[b];
                    if (va >= isovalue) != (vb >= isovalue) {
                        let t = (isovalue - va) / (vb - va);
                        sum[0] += corner_pts[a][0] + t * (corner_pts[b][0] - corner_pts[a][0]);
                        sum[1] += corner_pts[a][1] + t * (corner_pts[b][1] - corner_pts[a][1]);
                        sum[2] += corner_pts[a][2] + t * (corner_pts[b][2] - corner_pts[a][2]);
                        count += 1;
                    }
                }

                if count > 0 {
                    let vid = out_points.len();
                    out_points.push([
                        sum[0] / count as f64,
                        sum[1] / count as f64,
                        sum[2] / count as f64,
                    ]);
                    cell_vertex.insert(cell_idx(i, j, k), vid);
                }
            }
        }
    }

    // Connect adjacent cells with quads
    let mut out_polys = CellArray::new();

    for k in 0..ncz {
        for j in 0..ncy {
            for i in 0..ncx {
                let ci = cell_idx(i, j, k);
                if !cell_vertex.contains_key(&ci) {
                    continue;
                }

                // Check 3 face directions: +X, +Y, +Z
                // +X face: shared edge between (i,j,k) and (i+1,j,k)
                if i + 1 < ncx {
                    let edge_val_a = values[idx(i + 1, j, k)];
                    let edge_val_b = values[idx(i + 1, j + 1, k)];
                    let edge_val_c = values[idx(i + 1, j + 1, k + 1)];
                    let edge_val_d = values[idx(i + 1, j, k + 1)];
                    // The face between cells (i,j,k) and (i+1,j,k)
                    // straddles if the 4 face vertices have mixed signs
                    let face_above = [edge_val_a, edge_val_b, edge_val_c, edge_val_d]
                        .iter()
                        .filter(|&&v| v >= isovalue)
                        .count();
                    if face_above > 0 && face_above < 4 {
                        try_add_quad(
                            &cell_vertex,
                            &mut out_polys,
                            cell_idx(i, j, k),
                            cell_idx(i + 1, j, k),
                            if j + 1 < ncy {
                                Some(cell_idx(i + 1, j + 1, k))
                            } else {
                                None
                            },
                            if j + 1 < ncy {
                                Some(cell_idx(i, j + 1, k))
                            } else {
                                None
                            },
                        );
                    }
                }

                // +Y face
                if j + 1 < ncy {
                    let edge_val_a = values[idx(i, j + 1, k)];
                    let edge_val_b = values[idx(i + 1, j + 1, k)];
                    let edge_val_c = values[idx(i + 1, j + 1, k + 1)];
                    let edge_val_d = values[idx(i, j + 1, k + 1)];
                    let face_above = [edge_val_a, edge_val_b, edge_val_c, edge_val_d]
                        .iter()
                        .filter(|&&v| v >= isovalue)
                        .count();
                    if face_above > 0 && face_above < 4 {
                        try_add_quad(
                            &cell_vertex,
                            &mut out_polys,
                            cell_idx(i, j, k),
                            cell_idx(i, j + 1, k),
                            if k + 1 < ncz {
                                Some(cell_idx(i, j + 1, k + 1))
                            } else {
                                None
                            },
                            if k + 1 < ncz {
                                Some(cell_idx(i, j, k + 1))
                            } else {
                                None
                            },
                        );
                    }
                }

                // +Z face
                if k + 1 < ncz {
                    let edge_val_a = values[idx(i, j, k + 1)];
                    let edge_val_b = values[idx(i + 1, j, k + 1)];
                    let edge_val_c = values[idx(i + 1, j + 1, k + 1)];
                    let edge_val_d = values[idx(i, j + 1, k + 1)];
                    let face_above = [edge_val_a, edge_val_b, edge_val_c, edge_val_d]
                        .iter()
                        .filter(|&&v| v >= isovalue)
                        .count();
                    if face_above > 0 && face_above < 4 {
                        try_add_quad(
                            &cell_vertex,
                            &mut out_polys,
                            cell_idx(i, j, k),
                            cell_idx(i, j, k + 1),
                            if i + 1 < ncx {
                                Some(cell_idx(i + 1, j, k + 1))
                            } else {
                                None
                            },
                            if i + 1 < ncx {
                                Some(cell_idx(i + 1, j, k))
                            } else {
                                None
                            },
                        );
                    }
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn try_add_quad(
    cell_vertex: &HashMap<usize, usize>,
    polys: &mut CellArray,
    a: usize,
    b: usize,
    c: Option<usize>,
    d: Option<usize>,
) {
    let va = cell_vertex.get(&a);
    let vb = cell_vertex.get(&b);
    let vc = c.and_then(|ci| cell_vertex.get(&ci));
    let vd = d.and_then(|ci| cell_vertex.get(&ci));

    if let (Some(&a), Some(&b), Some(&c), Some(&d)) = (va, vb, vc, vd) {
        polys.push_cell(&[a as i64, b as i64, c as i64, d as i64]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    fn make_sphere_image(dims: usize) -> ImageData {
        let mut img = ImageData::with_dimensions(dims, dims, dims);
        img.set_origin([-1.0, -1.0, -1.0]);
        let sp = 2.0 / (dims as f64 - 1.0);
        img.set_spacing([sp, sp, sp]);

        let n = dims * dims * dims;
        let mut values = Vec::with_capacity(n);
        for k in 0..dims {
            for j in 0..dims {
                for i in 0..dims {
                    let x = -1.0 + i as f64 * sp;
                    let y = -1.0 + j as f64 * sp;
                    let z = -1.0 + k as f64 * sp;
                    values.push(x * x + y * y + z * z);
                }
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("field", values, 1)));
        img
    }

    #[test]
    fn sphere_isosurface() {
        let img = make_sphere_image(10);
        let result = surface_nets(&img, "field", 0.5);
        assert!(result.points.len() > 10);
        assert!(result.polys.num_cells() > 5);
    }

    #[test]
    fn no_isosurface() {
        let img = make_sphere_image(5);
        // Isovalue outside all values
        let result = surface_nets(&img, "field", 100.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn missing_scalars() {
        let img = make_sphere_image(5);
        let result = surface_nets(&img, "missing", 0.5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
