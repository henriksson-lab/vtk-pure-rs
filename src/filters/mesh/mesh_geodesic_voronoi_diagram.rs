//! Geodesic Voronoi diagram on mesh surface: region labels plus the polylines
//! where two regions meet.
//!
//! The partition itself is computed by
//! [`crate::filters::mesh::mesh_geodesic_voronoi`] (re-exported here); this
//! module only adds the boundary extraction on top of it.
use crate::data::PolyData;

pub use crate::filters::mesh::mesh_geodesic_voronoi::geodesic_voronoi;

/// Extract the edges separating two Voronoi regions as a line mesh.
pub fn geodesic_voronoi_boundaries(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    if seeds.is_empty() {
        return PolyData::new();
    }
    let labeled = geodesic_voronoi(mesh, seeds);
    let Some(arr) = labeled.point_data().get_array("VoronoiRegion") else {
        return PolyData::new();
    };
    let mut buf = [0.0f64];
    let labels: Vec<usize> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] < 0.0 {
                usize::MAX
            } else {
                buf[0] as usize
            }
        })
        .collect();
    let mut pts = crate::data::Points::<f64>::new();
    let mut lines = crate::data::CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], labels.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], labels.len()) else {
                continue;
            };
            if labels[a] != labels[b] {
                let ia = *pm.entry(a).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(a));
                    i
                });
                let ib = *pm.entry(b).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(b));
                    i
                });
                lines.push_cell(&[ia as i64, ib as i64]);
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boundaries() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [2.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = geodesic_voronoi_boundaries(&m, &[0, 3]);
        assert!(r.lines.num_cells() >= 1);
    }
}
