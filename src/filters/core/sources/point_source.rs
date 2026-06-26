use crate::data::{CellArray, Points, PolyData};

/// Point placement distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointDistribution {
    /// Points are placed on the sphere surface.
    Shell,
    /// Points are placed uniformly through the sphere volume.
    Uniform,
    /// Points are placed with VTK's exponential radial distribution.
    Exponential,
}

/// Parameters for generating a random point cloud.
pub struct PointSourceParams {
    /// Number of points to generate. Default: 10
    pub number_of_points: usize,
    /// Center of the point cloud. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Radius of the bounding sphere. Default: 0.5
    pub radius: f64,
    /// Random seed. Default: 42
    pub seed: u64,
    /// Point distribution. Default: Uniform
    pub distribution: PointDistribution,
    /// Exponential distribution lambda. Default: 1.0
    pub lambda: f64,
}

impl Default for PointSourceParams {
    fn default() -> Self {
        Self {
            number_of_points: 10,
            center: [0.0, 0.0, 0.0],
            radius: 0.5,
            seed: 42,
            distribution: PointDistribution::Uniform,
            lambda: 1.0,
        }
    }
}

/// Generate a random point cloud within a sphere.
pub fn point_source(params: &PointSourceParams) -> PolyData {
    let mut points = Points::new();
    let mut verts = CellArray::new();
    let mut state = params.seed;
    let number_of_points = params.number_of_points.max(1);

    let mut vertex_ids = Vec::with_capacity(number_of_points);
    for i in 0..number_of_points {
        let cosphi = 1.0 - 2.0 * next_random(&mut state);
        let sinphi = (1.0 - cosphi * cosphi).sqrt();
        let vtk_radius = params.radius.max(0.0);
        let rho = match params.distribution {
            PointDistribution::Shell => vtk_radius,
            PointDistribution::Exponential if params.lambda != 0.0 => {
                let u = next_random(&mut state);
                (1.0 - u * (1.0 - (-params.lambda * vtk_radius).exp())).ln() / params.lambda
            }
            PointDistribution::Exponential | PointDistribution::Uniform => {
                vtk_radius * next_random(&mut state).powf(0.33333333)
            }
        };
        let radius = rho * sinphi;
        let theta = 2.0 * std::f64::consts::PI * next_random(&mut state);

        points.push([
            params.center[0] + radius * theta.cos(),
            params.center[1] + radius * theta.sin(),
            params.center[2] + rho * cosphi,
        ]);
        vertex_ids.push(i as i64);
    }
    verts.push_cell(&vertex_ids);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.verts = verts;
    pd
}

fn next_random(state: &mut u64) -> f64 {
    // xorshift64
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state & 0xFFFFFFFF) as f64 / 0xFFFFFFFF_u64 as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_point_source() {
        let pd = point_source(&PointSourceParams::default());
        assert_eq!(pd.points.len(), 10);
        assert_eq!(pd.verts.num_cells(), 1);
        assert_eq!(pd.verts.cell(0).len(), 10);
    }

    #[test]
    fn points_within_radius() {
        let pd = point_source(&PointSourceParams {
            number_of_points: 50,
            radius: 1.0,
            center: [0.0, 0.0, 0.0],
            seed: 123,
            distribution: PointDistribution::Uniform,
            lambda: 1.0,
        });
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let dist = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(dist <= 1.0 + 1e-10, "point {} at distance {}", i, dist);
        }
    }

    #[test]
    fn centered_point_source() {
        let pd = point_source(&PointSourceParams {
            number_of_points: 10,
            center: [5.0, 5.0, 5.0],
            radius: 0.1,
            seed: 99,
            distribution: PointDistribution::Uniform,
            lambda: 1.0,
        });
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            assert!((p[0] - 5.0).abs() <= 0.1 + 1e-10);
            assert!((p[1] - 5.0).abs() <= 0.1 + 1e-10);
            assert!((p[2] - 5.0).abs() <= 0.1 + 1e-10);
        }
    }

    #[test]
    fn shell_distribution_places_points_on_radius() {
        let pd = point_source(&PointSourceParams {
            number_of_points: 20,
            radius: 2.0,
            center: [1.0, -1.0, 0.5],
            seed: 7,
            distribution: PointDistribution::Shell,
            lambda: 1.0,
        });
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let dx = p[0] - 1.0;
            let dy = p[1] + 1.0;
            let dz = p[2] - 0.5;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            assert!(
                (dist - 2.0).abs() <= 1e-10,
                "point {} at distance {}",
                i,
                dist
            );
        }
    }

    #[test]
    fn exponential_zero_lambda_falls_back_to_uniform() {
        let pd = point_source(&PointSourceParams {
            number_of_points: 12,
            radius: 1.0,
            center: [0.0, 0.0, 0.0],
            seed: 11,
            distribution: PointDistribution::Exponential,
            lambda: 0.0,
        });
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.verts.num_cells(), 1);
    }
}
