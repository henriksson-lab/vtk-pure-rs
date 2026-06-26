//! Evenly-spaced streamline seeding for 2D vector fields.
//!
//! Implements the Jobard-Lefer algorithm for generating streamlines with
//! uniform spacing across the domain. Produces aesthetically pleasing
//! flow visualizations.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

/// Parameters for evenly-spaced streamline generation.
pub struct EvenSpacedStreamParams {
    /// Separation distance between streamlines. Default: 1.0
    pub separation: f64,
    /// Integration step size. Default: 0.05
    pub step_size: f64,
    /// Maximum number of integration steps per streamline. Default: 500
    pub max_steps: usize,
    /// Minimum velocity magnitude to continue. Default: 1e-8
    pub terminal_speed: f64,
    /// Maximum number of streamlines. Default: 1000
    pub max_lines: usize,
}

impl Default for EvenSpacedStreamParams {
    fn default() -> Self {
        Self {
            separation: 1.0,
            step_size: 0.05,
            max_steps: 500,
            terminal_speed: 1e-8,
            max_lines: 1000,
        }
    }
}

/// Generate evenly-spaced streamlines in a 2D vector field on ImageData.
///
/// Uses a simplified Jobard-Lefer approach: integrates from a seed,
/// then seeds new streamlines at the separation distance perpendicular
/// to existing ones.
pub fn even_spaced_streamlines_2d(field: &ImageData, params: &EvenSpacedStreamParams) -> PolyData {
    let vectors = match field.point_data().vectors() {
        Some(v) if v.num_components() >= 2 => v,
        _ => return PolyData::new(),
    };

    let dims = field.dimensions();
    let spacing = field.spacing();
    let origin = field.origin();

    if dims[0] < 2 || dims[1] < 2 {
        return PolyData::new();
    }
    if params.separation <= 0.0 || params.step_size <= 0.0 {
        return PolyData::new();
    }

    let end_x = origin[0] + (dims[0] - 1) as f64 * spacing[0];
    let end_y = origin[1] + (dims[1] - 1) as f64 * spacing[1];
    let domain_min = [origin[0].min(end_x), origin[1].min(end_y), origin[2]];
    let domain_w = (end_x - origin[0]).abs();
    let domain_h = (end_y - origin[1]).abs();

    // Grid for checking separation distance
    let cell_size = params.separation * 0.5;
    let grid_nx = (domain_w / cell_size).ceil() as usize + 1;
    let grid_ny = (domain_h / cell_size).ceil() as usize + 1;
    let mut occupied: Vec<bool> = vec![false; grid_nx * grid_ny];

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut line_id_data: Vec<f64> = Vec::new();
    let mut seed_queue: Vec<[f64; 2]> = Vec::new();
    let mut num_lines = 0;

    // Start with center of domain
    let cx = (origin[0] + end_x) / 2.0;
    let cy = (origin[1] + end_y) / 2.0;
    seed_queue.push([cx, cy]);

    while let Some(seed) = seed_queue.pop() {
        if num_lines >= params.max_lines {
            break;
        }

        // Check if seed is too close to existing streamlines
        if is_too_close(
            seed,
            &occupied,
            domain_min,
            cell_size,
            grid_nx,
            grid_ny,
            params.separation,
        ) {
            continue;
        }

        // Integrate streamline in both directions
        let forward = integrate_2d(vectors, seed, params, origin, spacing, dims, 1.0);
        let backward = integrate_2d(vectors, seed, params, origin, spacing, dims, -1.0);

        // Combine: backward (reversed) + forward
        let mut line_points: Vec<[f64; 2]> = Vec::new();
        for i in (0..backward.len()).rev() {
            line_points.push(backward[i]);
        }
        if !forward.is_empty() {
            // Skip first forward point (same as seed)
            for i in 1..forward.len() {
                line_points.push(forward[i]);
            }
        }

        if line_points.len() < 2 {
            continue;
        }

        // Mark cells as occupied and generate new seeds
        let mut line_ids: Vec<i64> = Vec::new();
        for (pi, &pt) in line_points.iter().enumerate() {
            let idx = out_points.len() as i64;
            out_points.push([pt[0], pt[1], 0.0]);
            line_id_data.push(num_lines as f64);
            line_ids.push(idx);

            mark_occupied(&mut occupied, pt, domain_min, cell_size, grid_nx, grid_ny);

            // Generate perpendicular seeds at separation distance
            if pi > 0 && pi < line_points.len() - 1 {
                let prev = line_points[pi - 1];
                let next = line_points[pi + 1];
                let dx = next[0] - prev[0];
                let dy = next[1] - prev[1];
                let len = (dx * dx + dy * dy).sqrt();
                if len > 1e-15 {
                    let nx = -dy / len * params.separation;
                    let ny = dx / len * params.separation;

                    let s1 = [pt[0] + nx, pt[1] + ny];
                    let s2 = [pt[0] - nx, pt[1] - ny];

                    if in_domain(s1, domain_min, domain_w, domain_h)
                        && !is_too_close(
                            s1,
                            &occupied,
                            domain_min,
                            cell_size,
                            grid_nx,
                            grid_ny,
                            params.separation * 0.8,
                        )
                    {
                        seed_queue.push(s1);
                    }
                    if in_domain(s2, domain_min, domain_w, domain_h)
                        && !is_too_close(
                            s2,
                            &occupied,
                            domain_min,
                            cell_size,
                            grid_nx,
                            grid_ny,
                            params.separation * 0.8,
                        )
                    {
                        seed_queue.push(s2);
                    }
                }
            }
        }

        out_lines.push_cell(&line_ids);
        num_lines += 1;
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.lines = out_lines;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "StreamlineId",
            line_id_data,
            1,
        )));
    result
}

fn integrate_2d(
    vectors: &AnyDataArray,
    seed: [f64; 2],
    params: &EvenSpacedStreamParams,
    origin: [f64; 3],
    spacing: [f64; 3],
    dims: [usize; 3],
    direction: f64,
) -> Vec<[f64; 2]> {
    let mut points = Vec::new();
    let mut pos = seed;
    let dt = params.step_size * direction;

    for _ in 0..params.max_steps {
        let end_x = origin[0] + (dims[0] - 1) as f64 * spacing[0];
        let end_y = origin[1] + (dims[1] - 1) as f64 * spacing[1];
        let min_x = origin[0].min(end_x);
        let max_x = origin[0].max(end_x);
        let min_y = origin[1].min(end_y);
        let max_y = origin[1].max(end_y);
        if pos[0] < min_x || pos[0] > max_x || pos[1] < min_y || pos[1] > max_y {
            break;
        }

        points.push(pos);

        let Some(v) = interp_vec2(vectors, pos, origin, spacing, dims) else {
            break;
        };
        let speed = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if speed < params.terminal_speed {
            break;
        }

        // RK2 midpoint method
        let mid = [pos[0] + 0.5 * dt * v[0], pos[1] + 0.5 * dt * v[1]];
        let Some(vm) = interp_vec2(vectors, mid, origin, spacing, dims) else {
            break;
        };

        pos = [pos[0] + dt * vm[0], pos[1] + dt * vm[1]];
    }

    points
}

fn interp_vec2(
    vectors: &AnyDataArray,
    pos: [f64; 2],
    origin: [f64; 3],
    spacing: [f64; 3],
    dims: [usize; 3],
) -> Option<[f64; 2]> {
    if dims[0] < 2 || dims[1] < 2 || spacing[0] == 0.0 || spacing[1] == 0.0 {
        return None;
    }
    let end_x = origin[0] + (dims[0] - 1) as f64 * spacing[0];
    let end_y = origin[1] + (dims[1] - 1) as f64 * spacing[1];
    let min_x = origin[0].min(end_x);
    let max_x = origin[0].max(end_x);
    let min_y = origin[1].min(end_y);
    let max_y = origin[1].max(end_y);
    if pos[0] < min_x || pos[0] > max_x || pos[1] < min_y || pos[1] > max_y {
        return None;
    }

    let fx = (pos[0] - origin[0]) / spacing[0];
    let fy = (pos[1] - origin[1]) / spacing[1];
    let ix = fx.floor() as usize;
    let iy = fy.floor() as usize;
    let ix = ix.min(dims[0].saturating_sub(2));
    let iy = iy.min(dims[1].saturating_sub(2));
    let tx = (fx - ix as f64).clamp(0.0, 1.0);
    let ty = (fy - iy as f64).clamp(0.0, 1.0);

    let mut result = [0.0; 2];
    let mut buf = [0.0f64; 3];

    for dy in 0..2usize {
        for dx in 0..2usize {
            let idx = (ix + dx) + (iy + dy) * dims[0];
            if idx < vectors.num_tuples() {
                vectors.tuple_as_f64(idx, &mut buf);
                let wx = if dx == 0 { 1.0 - tx } else { tx };
                let wy = if dy == 0 { 1.0 - ty } else { ty };
                let w = wx * wy;
                result[0] += w * buf[0];
                result[1] += w * buf[1];
            }
        }
    }
    Some(result)
}

fn grid_idx(
    pt: [f64; 2],
    origin: [f64; 3],
    cell_size: f64,
    grid_nx: usize,
    grid_ny: usize,
) -> Option<usize> {
    let gx = ((pt[0] - origin[0]) / cell_size) as usize;
    let gy = ((pt[1] - origin[1]) / cell_size) as usize;
    if gx < grid_nx && gy < grid_ny {
        Some(gx + gy * grid_nx)
    } else {
        None
    }
}

fn is_too_close(
    pt: [f64; 2],
    occupied: &[bool],
    origin: [f64; 3],
    cell_size: f64,
    grid_nx: usize,
    grid_ny: usize,
    min_dist: f64,
) -> bool {
    let gx = ((pt[0] - origin[0]) / cell_size) as i64;
    let gy = ((pt[1] - origin[1]) / cell_size) as i64;
    let r = (min_dist / cell_size).ceil() as i64 + 1;

    for dy in -r..=r {
        for dx in -r..=r {
            let nx = gx + dx;
            let ny = gy + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < grid_nx && (ny as usize) < grid_ny {
                if occupied[nx as usize + ny as usize * grid_nx] {
                    return true;
                }
            }
        }
    }
    false
}

fn mark_occupied(
    occupied: &mut [bool],
    pt: [f64; 2],
    origin: [f64; 3],
    cell_size: f64,
    grid_nx: usize,
    grid_ny: usize,
) {
    if let Some(idx) = grid_idx(pt, origin, cell_size, grid_nx, grid_ny) {
        occupied[idx] = true;
    }
}

fn in_domain(pt: [f64; 2], origin: [f64; 3], w: f64, h: f64) -> bool {
    pt[0] >= origin[0] && pt[0] <= origin[0] + w && pt[1] >= origin[1] && pt[1] <= origin[1] + h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_uniform_2d_field() -> ImageData {
        let dims = [20, 20, 1];
        let spacing = [0.5, 0.5, 1.0];
        let origin = [0.0, 0.0, 0.0];
        let n = dims[0] * dims[1];

        let mut vdata = Vec::with_capacity(n * 3);
        for _ in 0..n {
            vdata.push(1.0); // uniform X flow
            vdata.push(0.0);
            vdata.push(0.0);
        }

        let mut field = ImageData::with_dimensions(dims[0], dims[1], dims[2]);
        field.set_spacing(spacing);
        field.set_origin(origin);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vdata, 3)));
        field.point_data_mut().set_active_vectors("velocity");
        field
    }

    #[test]
    fn uniform_flow_streamlines() {
        let field = make_uniform_2d_field();
        let params = EvenSpacedStreamParams {
            separation: 2.0,
            step_size: 0.1,
            max_steps: 100,
            max_lines: 50,
            ..Default::default()
        };
        let result = even_spaced_streamlines_2d(&field, &params);
        assert!(
            result.lines.num_cells() >= 1,
            "should produce at least one streamline"
        );
        assert!(result.points.len() > 2);
    }

    #[test]
    fn vortex_field_streamlines() {
        let dims = [30, 30, 1];
        let spacing = [0.1, 0.1, 1.0];
        let origin = [0.0, 0.0, 0.0];
        let cx = 1.5;
        let cy = 1.5;

        let n = dims[0] * dims[1];
        let mut vdata = Vec::with_capacity(n * 3);
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let x = origin[0] + ix as f64 * spacing[0];
                let y = origin[1] + iy as f64 * spacing[1];
                vdata.push(-(y - cy)); // circular flow
                vdata.push(x - cx);
                vdata.push(0.0);
            }
        }

        let mut field = ImageData::with_dimensions(dims[0], dims[1], dims[2]);
        field.set_spacing(spacing);
        field.set_origin(origin);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vdata, 3)));
        field.point_data_mut().set_active_vectors("velocity");

        let params = EvenSpacedStreamParams {
            separation: 0.5,
            step_size: 0.02,
            max_steps: 200,
            max_lines: 20,
            ..Default::default()
        };
        let result = even_spaced_streamlines_2d(&field, &params);
        assert!(result.lines.num_cells() >= 1);
    }

    #[test]
    fn empty_field() {
        let field = ImageData::new();
        let result = even_spaced_streamlines_2d(&field, &EvenSpacedStreamParams::default());
        assert_eq!(result.points.len(), 0);
    }
}
