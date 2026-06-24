use crate::data::{CellArray, Points, PolyData};

/// Parameters for streamline integration.
pub struct StreamTracerParams {
    /// Maximum number of integration steps. Default: 500
    pub max_steps: usize,
    /// Integration step size. Default: 0.1
    pub step_size: f64,
    /// Minimum velocity magnitude to continue. Default: 1e-8
    pub terminal_speed: f64,
}

impl Default for StreamTracerParams {
    fn default() -> Self {
        Self {
            max_steps: 500,
            step_size: 0.1,
            terminal_speed: 1e-8,
        }
    }
}

/// Integrate streamlines through a vector field defined at points.
///
/// For each seed point, traces a streamline by stepping through the
/// velocity field using 4th-order Runge-Kutta integration with
/// nearest-neighbor interpolation.
///
/// `source` contains the vector field (active vectors in point data).
/// `seeds` contains the starting points.
pub fn stream_tracer(source: &PolyData, seeds: &PolyData, params: &StreamTracerParams) -> PolyData {
    let n_source = source.points.len();
    if n_source == 0 {
        return PolyData::new();
    }

    // Get vector field
    let vectors = match source.point_data().vectors() {
        Some(v) if v.num_components() == 3 => v,
        _ => return PolyData::new(),
    };

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for si in 0..seeds.points.len() {
        let mut pos = seeds.points.get(si);
        let mut line_ids: Vec<i64> = Vec::new();

        for _ in 0..params.max_steps {
            let idx = out_points.len() as i64;
            out_points.push(pos);
            line_ids.push(idx);

            // RK4 integration
            let k1 = interpolate_vector(source, vectors, pos);
            let speed = (k1[0] * k1[0] + k1[1] * k1[1] + k1[2] * k1[2]).sqrt();
            if speed < params.terminal_speed {
                break;
            }

            let h = params.step_size;
            let p2 = [
                pos[0] + 0.5 * h * k1[0],
                pos[1] + 0.5 * h * k1[1],
                pos[2] + 0.5 * h * k1[2],
            ];
            let k2 = interpolate_vector(source, vectors, p2);

            let p3 = [
                pos[0] + 0.5 * h * k2[0],
                pos[1] + 0.5 * h * k2[1],
                pos[2] + 0.5 * h * k2[2],
            ];
            let k3 = interpolate_vector(source, vectors, p3);

            let p4 = [pos[0] + h * k3[0], pos[1] + h * k3[1], pos[2] + h * k3[2]];
            let k4 = interpolate_vector(source, vectors, p4);

            pos = [
                pos[0] + h / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
                pos[1] + h / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
                pos[2] + h / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
            ];
        }

        if line_ids.len() >= 2 {
            out_lines.push_cell(&line_ids);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

fn interpolate_vector(
    source: &PolyData,
    vectors: &crate::data::AnyDataArray,
    pos: [f64; 3],
) -> [f64; 3] {
    // Nearest-neighbor interpolation
    let mut best_dist = f64::MAX;
    let mut best_idx = 0;
    for i in 0..source.points.len() {
        let sp = source.points.get(i);
        let d = (pos[0] - sp[0]) * (pos[0] - sp[0])
            + (pos[1] - sp[1]) * (pos[1] - sp[1])
            + (pos[2] - sp[2]) * (pos[2] - sp[2]);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    let mut buf = [0.0f64; 3];
    vectors.tuple_as_f64(best_idx, &mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn streamline_constant_field() {
        // Uniform velocity field in +x direction
        let mut source = PolyData::new();
        for i in 0..10 {
            source.points.push([i as f64, 0.0, 0.0]);
        }
        let vecs: Vec<f64> = (0..10).flat_map(|_| vec![1.0, 0.0, 0.0]).collect();
        let arr = DataArray::from_vec("velocity", vecs, 3);
        source.point_data_mut().add_array(arr.into());
        source.point_data_mut().set_active_vectors("velocity");

        let mut seeds = PolyData::new();
        seeds.points.push([0.0, 0.0, 0.0]);

        let result = stream_tracer(
            &source,
            &seeds,
            &StreamTracerParams {
                max_steps: 20,
                step_size: 0.5,
                ..Default::default()
            },
        );

        assert_eq!(result.lines.num_cells(), 1);
        assert!(result.points.len() > 5);

        // Streamline should advance in +x
        let last = result.points.get(result.points.len() - 1);
        assert!(last[0] > 5.0, "last x = {}", last[0]);
    }

    #[test]
    fn no_vectors_returns_empty() {
        let source = PolyData::new();
        let seeds = PolyData::new();
        let result = stream_tracer(&source, &seeds, &Default::default());
        assert_eq!(result.lines.num_cells(), 0);
    }
}
