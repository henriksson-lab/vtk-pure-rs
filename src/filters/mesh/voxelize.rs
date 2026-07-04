use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};

/// Voxelize a triangle mesh into an ImageData volume.
///
/// Voxels inside the mesh are set to 1.0 and outside to 0.0 in the
/// "Voxels" scalar array. Uses ray-casting along the Z axis with parity
/// counting to determine inside/outside.
///
/// `dims` specifies the output grid resolution `[nx, ny, nz]`.
/// `bounds` specifies the spatial extent `[[xmin, xmax], [ymin, ymax], [zmin, zmax]]`.
pub fn voxelize(input: &PolyData, dims: [u32; 3], bounds: [[f64; 2]; 3]) -> ImageData {
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    let sx: f64 = if nx > 1 {
        (bounds[0][1] - bounds[0][0]) / (nx - 1) as f64
    } else {
        1.0
    };
    let sy: f64 = if ny > 1 {
        (bounds[1][1] - bounds[1][0]) / (ny - 1) as f64
    } else {
        1.0
    };
    let sz: f64 = if nz > 1 {
        (bounds[2][1] - bounds[2][0]) / (nz - 1) as f64
    } else {
        1.0
    };

    // Collect triangles
    let mut triangles: Vec<[[f64; 3]; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if !valid_cell(cell, input.points.len()) {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            triangles.push([v0, v1, v2]);
        }
    }

    let mut voxels: Vec<f64> = vec![0.0; n];

    // For each (i, j) column, cast a ray along Z and count crossings
    for j in 0..ny {
        let ry: f64 = bounds[1][0] + j as f64 * sy;
        for i in 0..nx {
            let rx: f64 = bounds[0][0] + i as f64 * sx;

            // Collect Z intersections of ray (rx, ry, z) with all triangles
            let mut z_hits: Vec<f64> = Vec::new();

            for tri in &triangles {
                if let Some(z) = ray_z_intersect(rx, ry, tri) {
                    z_hits.push(z);
                }
            }

            z_hits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            collapse_duplicate_hits(&mut z_hits, sz.abs().max(1.0) * 1e-9);

            // Walk voxels along Z, toggle inside/outside at each crossing
            let mut hit_idx: usize = 0;
            let mut inside: bool = false;
            for k in 0..nz {
                let rz: f64 = bounds[2][0] + k as f64 * sz;

                // Advance past all crossings below this Z
                while hit_idx < z_hits.len() && z_hits[hit_idx] < rz - sz * 0.5 {
                    inside = !inside;
                    hit_idx += 1;
                }

                if inside {
                    voxels[k * ny * nx + j * nx + i] = 1.0;
                }
            }
        }
    }

    let mut output = ImageData::with_dimensions(nx, ny, nz);
    output.set_spacing([sx, sy, sz]);
    output.set_origin([bounds[0][0], bounds[1][0], bounds[2][0]]);
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Voxels", voxels, 1)));
    output
}

fn valid_cell(cell: &[i64], num_points: usize) -> bool {
    cell.len() >= 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn collapse_duplicate_hits(z_hits: &mut Vec<f64>, tolerance: f64) {
    if z_hits.len() < 2 {
        return;
    }

    let mut write = 1;
    let mut last = z_hits[0];
    for read in 1..z_hits.len() {
        if (z_hits[read] - last).abs() > tolerance {
            z_hits[write] = z_hits[read];
            last = z_hits[read];
            write += 1;
        }
    }
    z_hits.truncate(write);
}

/// Test if a Z-axis ray at (rx, ry) intersects a triangle, returning the Z coordinate.
fn ray_z_intersect(rx: f64, ry: f64, tri: &[[f64; 3]; 3]) -> Option<f64> {
    let ax: f64 = tri[0][0];
    let ay: f64 = tri[0][1];
    let bx: f64 = tri[1][0];
    let by: f64 = tri[1][1];
    let cx: f64 = tri[2][0];
    let cy: f64 = tri[2][1];

    // Barycentric coordinates in XY
    let denom: f64 = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy);
    if denom.abs() < 1e-15 {
        return None;
    }
    let inv_denom: f64 = 1.0 / denom;

    let u: f64 = ((by - cy) * (rx - cx) + (cx - bx) * (ry - cy)) * inv_denom;
    let v: f64 = ((cy - ay) * (rx - cx) + (ax - cx) * (ry - cy)) * inv_denom;
    let w: f64 = 1.0 - u - v;

    let eps: f64 = -1e-8;
    if u < eps || v < eps || w < eps {
        return None;
    }

    let z: f64 = u * tri[0][2] + v * tri[1][2] + w * tri[2][2];
    Some(z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box_mesh() -> PolyData {
        // A simple axis-aligned box from (0,0,0) to (1,1,1)
        let pts = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        // 12 triangles forming the 6 faces
        let tris = vec![
            [0, 2, 1],
            [0, 3, 2], // -Z face
            [4, 5, 6],
            [4, 6, 7], // +Z face
            [0, 1, 5],
            [0, 5, 4], // -Y face
            [2, 3, 7],
            [2, 7, 6], // +Y face
            [0, 4, 7],
            [0, 7, 3], // -X face
            [1, 2, 6],
            [1, 6, 5], // +X face
        ];
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn voxelize_box_produces_correct_dimensions() {
        let mesh = make_box_mesh();
        let result = voxelize(&mesh, [5, 5, 5], [[-0.5, 1.5], [-0.5, 1.5], [-0.5, 1.5]]);
        assert_eq!(result.dimensions(), [5, 5, 5]);
        assert_eq!(result.num_points(), 125);
        let arr = result.point_data().get_array("Voxels").unwrap();
        assert_eq!(arr.num_tuples(), 125);
    }

    #[test]
    fn voxelize_box_has_inside_voxels() {
        let mesh = make_box_mesh();
        let result = voxelize(&mesh, [5, 5, 5], [[-0.5, 1.5], [-0.5, 1.5], [-0.5, 1.5]]);
        let arr = result.point_data().get_array("Voxels").unwrap();
        let mut buf = [0.0f64];

        // Count how many voxels are inside
        let mut inside_count: usize = 0;
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                inside_count += 1;
            }
        }
        // Should have some inside voxels but not all
        assert!(inside_count > 0);
        assert!(inside_count < 125);
    }

    #[test]
    fn empty_mesh_gives_all_outside() {
        let mesh = PolyData::new();
        let result = voxelize(&mesh, [3, 3, 3], [[0.0, 1.0], [0.0, 1.0], [0.0, 1.0]]);
        let arr = result.point_data().get_array("Voxels").unwrap();
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0]).abs() < 1e-10);
        }
    }

    #[test]
    fn duplicate_hits_count_as_single_crossings() {
        let mut hits = vec![0.0, 0.0, 1.0, 1.0];
        collapse_duplicate_hits(&mut hits, 1e-9);
        assert_eq!(hits, vec![0.0, 1.0]);
    }

    #[test]
    fn invalid_cells_are_ignored() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);

        let result = voxelize(&mesh, [2, 2, 2], [[0.0, 1.0], [0.0, 1.0], [0.0, 1.0]]);
        let arr = result.point_data().get_array("Voxels").unwrap();
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }
}
