use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Parameters for generating a line source, following `vtkLineSource`.
pub struct LineParams {
    /// Start point. Default: [-0.5, 0, 0]
    pub point1: [f64; 3],
    /// End point. Default: [0.5, 0, 0]
    pub point2: [f64; 3],
    /// Divide each segment into this many pieces. Default: 1.
    pub resolution: usize,
}

impl Default for LineParams {
    fn default() -> Self {
        Self {
            point1: [-0.5, 0.0, 0.0],
            point2: [0.5, 0.0, 0.0],
            resolution: 1,
        }
    }
}

/// Full `vtkLineSource`-style parameters.
pub struct LineSourceParams {
    /// Start point. Default: [-0.5, 0, 0]
    pub point1: [f64; 3],
    /// End point. Default: [0.5, 0, 0]
    pub point2: [f64; 3],
    /// Divide each segment into this many pieces when regular refinement is used. Default: 1.
    pub resolution: usize,
    /// Optional corner points defining a broken line.
    pub points: Option<Vec<[f64; 3]>>,
    /// Use regular refinement based on resolution. Default: true.
    pub use_regular_refinement: bool,
    /// Explicit per-segment refinement ratios used when regular refinement is disabled.
    pub refinement_ratios: Vec<f64>,
}

impl Default for LineSourceParams {
    fn default() -> Self {
        Self {
            point1: [-0.5, 0.0, 0.0],
            point2: [0.5, 0.0, 0.0],
            resolution: 1,
            points: None,
            use_regular_refinement: true,
            refinement_ratios: Vec::new(),
        }
    }
}

impl From<&LineParams> for LineSourceParams {
    fn from(params: &LineParams) -> Self {
        Self {
            point1: params.point1,
            point2: params.point2,
            resolution: params.resolution,
            ..Default::default()
        }
    }
}

/// Generate a line as PolyData with a single polyline cell.
pub fn line(params: &LineParams) -> PolyData {
    line_source(&LineSourceParams::from(params))
}

/// Generate a line as PolyData with a single polyline cell, following `vtkLineSource`.
pub fn line_source(params: &LineSourceParams) -> PolyData {
    let input_points = params
        .points
        .clone()
        .unwrap_or_else(|| vec![params.point1, params.point2]);
    let n_segments = input_points.len().saturating_sub(1);
    if n_segments < 1 {
        return PolyData::new();
    }

    let resolution = params.resolution.max(1);
    let refinements = if params.use_regular_refinement {
        let mut refinements = Vec::with_capacity(resolution + 1);
        for cc in 0..resolution {
            refinements.push(cc as f64 / resolution as f64);
        }
        refinements.push(1.0);
        refinements
    } else {
        params.refinement_ratios.clone()
    };

    let mut points = Points::new();
    let mut lines = CellArray::new();
    let mut ids = Vec::with_capacity(n_segments * refinements.len());
    let skip_shared_endpoints =
        refinements.first() == Some(&0.0) && refinements.last() == Some(&1.0);

    for seg in 0..n_segments {
        let point1 = input_points[seg];
        let point2 = input_points[seg + 1];
        let v = [
            point2[0] - point1[0],
            point2[1] - point1[1],
            point2[2] - point1[2],
        ];
        for (i, refinement) in refinements.iter().enumerate() {
            if seg > 0 && i == 0 && skip_shared_endpoints {
                continue;
            }
            let idx = points.len();
            points.push([
                point1[0] + refinement * v[0],
                point1[1] + refinement * v[1],
                point1[2] + refinement * v[2],
            ]);
            ids.push(idx as i64);
        }
    }
    lines.push_cell(&ids);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;

    let mut tcoords = DataArray::<f32>::new("Texture Coordinates", 2);
    let mut lengths = vec![0.0f32; pd.points.len()];
    let mut length_sum = 0.0f32;
    for cc in 1..pd.points.len() {
        let p1 = pd.points.get(cc - 1);
        let p2 = pd.points.get(cc);
        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        let dz = p2[2] - p1[2];
        length_sum += (dx * dx + dy * dy + dz * dz).sqrt() as f32;
        lengths[cc] = length_sum;
    }
    for length in lengths {
        let s = if length_sum != 0.0 {
            length / length_sum
        } else {
            0.0
        };
        tcoords.push_tuple(&[s, 0.0]);
    }
    pd.point_data_mut().add_array(AnyDataArray::F32(tcoords));
    pd.point_data_mut()
        .set_active_tcoords("Texture Coordinates");

    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_line() {
        let pd = line(&LineParams::default());
        assert_eq!(pd.points.len(), 2);
        assert_eq!(pd.lines.num_cells(), 1);
        let p0 = pd.points.get(0);
        let p1 = pd.points.get(1);
        assert!((p0[0] + 0.5).abs() < 1e-10);
        assert!((p1[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn subdivided_line() {
        let pd = line(&LineParams {
            point1: [0.0, 0.0, 0.0],
            point2: [10.0, 0.0, 0.0],
            resolution: 10,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 11);
        assert_eq!(pd.lines.num_cells(), 1);
        assert_eq!(pd.lines.cell(0).len(), 11);
        // Midpoint should be at x=5
        let mid = pd.points.get(5);
        assert!((mid[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn generates_texture_coordinates() {
        let pd = line(&LineParams::default());
        assert!(pd.point_data().tcoords().is_some());
    }

    #[test]
    fn broken_line_skips_duplicate_segment_join() {
        let pd = line_source(&LineSourceParams {
            points: Some(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]),
            resolution: 2,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 5);
        assert_eq!(pd.lines.cell(0), &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn explicit_refinement_ratios() {
        let pd = line_source(&LineSourceParams {
            point1: [0.0, 0.0, 0.0],
            point2: [10.0, 0.0, 0.0],
            use_regular_refinement: false,
            refinement_ratios: vec![0.0, 0.25, 1.0],
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 3);
        assert!((pd.points.get(1)[0] - 2.5).abs() < 1e-10);
    }
}
