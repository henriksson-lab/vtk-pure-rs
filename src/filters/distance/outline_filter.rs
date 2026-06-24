//! Outline (bounding box wireframe) for any dataset.

use crate::data::{CellArray, ImageData, Points, PolyData, RectilinearGrid, UnstructuredGrid};
use crate::types::BoundingBox;

/// Create a wireframe outline (12 edges) of a bounding box.
pub fn outline_from_bounds(bounds: &BoundingBox) -> PolyData {
    let min = [bounds.x_min, bounds.y_min, bounds.z_min];
    let max = [bounds.x_max, bounds.y_max, bounds.z_max];
    let pts = vec![
        [min[0], min[1], min[2]], // 0
        [max[0], min[1], min[2]], // 1
        [max[0], max[1], min[2]], // 2
        [min[0], max[1], min[2]], // 3
        [min[0], min[1], max[2]], // 4
        [max[0], min[1], max[2]], // 5
        [max[0], max[1], max[2]], // 6
        [min[0], max[1], max[2]], // 7
    ];

    let edges: Vec<[usize; 2]> = vec![
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0], // bottom
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4], // top
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7], // verticals
    ];

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }

    let mut lines = CellArray::new();
    for e in &edges {
        lines.push_cell(&[e[0] as i64, e[1] as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;
    mesh
}

/// Create a wireframe outline of a PolyData's bounding box.
pub fn outline_poly_data(mesh: &PolyData) -> PolyData {
    if mesh.points.len() == 0 {
        return PolyData::new();
    }
    let mut min = mesh.points.get(0);
    let mut max = min;
    for i in 1..mesh.points.len() {
        let p = mesh.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }
    outline_from_bounds(&BoundingBox::from_corners(min, max))
}

/// Create a wireframe outline of an ImageData's bounds.
pub fn outline_image_data(image: &ImageData) -> PolyData {
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();
    let max = [
        origin[0] + (dims[0] as f64 - 1.0) * spacing[0],
        origin[1] + (dims[1] as f64 - 1.0) * spacing[1],
        origin[2] + (dims[2] as f64 - 1.0) * spacing[2],
    ];
    outline_from_bounds(&BoundingBox::from_corners(origin, max))
}

/// Create a wireframe outline of a RectilinearGrid's bounds.
pub fn outline_rectilinear_grid(grid: &RectilinearGrid) -> PolyData {
    let x = grid.x_coords();
    let y = grid.y_coords();
    let z = grid.z_coords();
    let min = [x[0], y[0], z[0]];
    let max = [x[x.len() - 1], y[y.len() - 1], z[z.len() - 1]];
    outline_from_bounds(&BoundingBox::from_corners(min, max))
}

/// Create a wireframe outline of an UnstructuredGrid's bounding box.
pub fn outline_unstructured_grid(grid: &UnstructuredGrid) -> PolyData {
    let n = grid.points.len();
    if n == 0 {
        return PolyData::new();
    }
    let mut min = grid.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = grid.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }
    outline_from_bounds(&BoundingBox::from_corners(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_unit_cube() {
        let bounds = BoundingBox::from_corners([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let outline = outline_from_bounds(&bounds);
        assert_eq!(outline.points.len(), 8);
        assert_eq!(outline.lines.num_cells(), 12);
    }

    #[test]
    fn outline_from_mesh() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let outline = outline_poly_data(&mesh);
        assert_eq!(outline.points.len(), 8);
        assert_eq!(outline.lines.num_cells(), 12);
    }

    #[test]
    fn outline_image() {
        let img = ImageData::with_dimensions(10, 10, 10).with_spacing([0.5, 0.5, 0.5]);
        let outline = outline_image_data(&img);
        assert_eq!(outline.points.len(), 8);
    }

    #[test]
    fn outline_rectilinear() {
        let grid =
            RectilinearGrid::from_coords(vec![0.0, 1.0, 3.0], vec![0.0, 2.0], vec![0.0, 1.0]);
        let outline = outline_rectilinear_grid(&grid);
        assert_eq!(outline.lines.num_cells(), 12);
    }

    #[test]
    fn empty_mesh_outline() {
        let mesh = PolyData::new();
        let outline = outline_poly_data(&mesh);
        assert_eq!(outline.points.len(), 0);
    }
}
