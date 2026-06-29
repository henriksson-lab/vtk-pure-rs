use crate::data::{
    AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData, RectilinearGrid,
    StructuredGrid, UnstructuredGrid,
};
use crate::types::CellType;

/// Convert an ImageData surface to PolyData quads (outer surface only).
pub fn image_data_surface_to_poly_data(img: &ImageData) -> PolyData {
    let dims = img.dimensions();
    let mut points = Points::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut surface_cell_ids = Vec::new();

    // Generate points
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                points.push(img.point_from_ijk(i, j, k));
            }
        }
    }

    let idx =
        |i: usize, j: usize, k: usize| -> i64 { (k * dims[1] * dims[0] + j * dims[0] + i) as i64 };
    let cell_idx = |i: usize, j: usize, k: usize| -> usize { structured_cell_idx(dims, i, j, k) };

    if let Some(cell_ids) = push_lower_dim_grid_cells(dims, idx, &mut verts, &mut lines, &mut polys)
    {
        let mut pd = PolyData::new();
        pd.points = points;
        pd.verts = verts;
        pd.lines = lines;
        pd.polys = polys;
        *pd.point_data_mut() = img.point_data().clone();
        copy_subset_cell_data(img.cell_data(), &cell_ids, pd.cell_data_mut());
        return pd;
    }

    // Z-min face (k=0)
    if dims[0] > 1 && dims[1] > 1 && dims[2] > 0 {
        for j in 0..dims[1] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, j, 0),
                    idx(i + 1, j, 0),
                    idx(i + 1, j + 1, 0),
                    idx(i, j + 1, 0),
                ]);
                surface_cell_ids.push(cell_idx(i, j, 0));
            }
        }
    }
    // Z-max face
    if dims[0] > 1 && dims[1] > 1 && dims[2] > 1 {
        let k = dims[2] - 1;
        for j in 0..dims[1] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, j, k),
                    idx(i, j + 1, k),
                    idx(i + 1, j + 1, k),
                    idx(i + 1, j, k),
                ]);
                surface_cell_ids.push(cell_idx(i, j, k - 1));
            }
        }
    }
    // Y-min face
    if dims[0] > 1 && dims[1] > 0 && dims[2] > 1 {
        for k in 0..dims[2] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, 0, k),
                    idx(i + 1, 0, k),
                    idx(i + 1, 0, k + 1),
                    idx(i, 0, k + 1),
                ]);
                surface_cell_ids.push(cell_idx(i, 0, k));
            }
        }
    }
    // Y-max face
    if dims[0] > 1 && dims[1] > 1 && dims[2] > 1 {
        let j = dims[1] - 1;
        for k in 0..dims[2] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, j, k),
                    idx(i, j, k + 1),
                    idx(i + 1, j, k + 1),
                    idx(i + 1, j, k),
                ]);
                surface_cell_ids.push(cell_idx(i, j - 1, k));
            }
        }
    }
    // X-min face
    if dims[0] > 0 && dims[1] > 1 && dims[2] > 1 {
        for k in 0..dims[2] - 1 {
            for j in 0..dims[1] - 1 {
                polys.push_cell(&[
                    idx(0, j, k),
                    idx(0, j, k + 1),
                    idx(0, j + 1, k + 1),
                    idx(0, j + 1, k),
                ]);
                surface_cell_ids.push(cell_idx(0, j, k));
            }
        }
    }
    // X-max face
    if dims[0] > 1 && dims[1] > 1 && dims[2] > 1 {
        let i = dims[0] - 1;
        for k in 0..dims[2] - 1 {
            for j in 0..dims[1] - 1 {
                polys.push_cell(&[
                    idx(i, j, k),
                    idx(i, j + 1, k),
                    idx(i, j + 1, k + 1),
                    idx(i, j, k + 1),
                ]);
                surface_cell_ids.push(cell_idx(i - 1, j, k));
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    *pd.point_data_mut() = img.point_data().clone();
    copy_subset_cell_data(img.cell_data(), &surface_cell_ids, pd.cell_data_mut());
    pd
}

/// Convert a PolyData triangle mesh to an UnstructuredGrid.
pub fn poly_data_to_unstructured_grid(pd: &PolyData) -> UnstructuredGrid {
    let mut ug = UnstructuredGrid::new();
    ug.points = pd.points.clone();

    for cell in pd.verts.iter() {
        let ct = if cell.len() == 1 {
            CellType::Vertex
        } else {
            CellType::PolyVertex
        };
        ug.push_cell(ct, cell);
    }
    for cell in pd.lines.iter() {
        let ct = if cell.len() == 2 {
            CellType::Line
        } else {
            CellType::PolyLine
        };
        ug.push_cell(ct, cell);
    }
    for cell in pd.polys.iter() {
        let ct = match cell.len() {
            3 => CellType::Triangle,
            4 => CellType::Quad,
            _ => CellType::Polygon,
        };
        ug.push_cell(ct, cell);
    }
    for cell in pd.strips.iter() {
        ug.push_cell(CellType::TriangleStrip, cell);
    }

    // Copy point and cell data in VTK's vtkPolyData cell order:
    // verts, lines, polys, strips.
    *ug.point_data_mut() = pd.point_data().clone();
    *ug.cell_data_mut() = pd.cell_data().clone();

    ug
}

/// Convert an UnstructuredGrid (triangles/quads only) to PolyData.
pub fn unstructured_grid_to_poly_data(ug: &UnstructuredGrid) -> PolyData {
    let mut pd = PolyData::new();
    pd.points = ug.points.clone();
    let mut included_cell_ids = Vec::new();

    for i in 0..ug.cells().num_cells() {
        let ct = ug.cell_type(i);
        let pts = ug.cell_points(i);
        match ct {
            CellType::Vertex | CellType::PolyVertex => {
                pd.verts.push_cell(pts);
                included_cell_ids.push(i);
            }
            CellType::Line | CellType::PolyLine => {
                pd.lines.push_cell(pts);
                included_cell_ids.push(i);
            }
            CellType::TriangleStrip => {
                pd.strips.push_cell(pts);
                included_cell_ids.push(i);
            }
            _ if ct.dimension() == 2 => {
                pd.polys.push_cell(pts);
                included_cell_ids.push(i);
            }
            _ => {}
        }
    }

    *pd.point_data_mut() = ug.point_data().clone();
    if included_cell_ids.len() == ug.cells().num_cells() {
        *pd.cell_data_mut() = ug.cell_data().clone();
    } else {
        for i in 0..ug.cell_data().num_arrays() {
            if let Some(arr) = ug.cell_data().get_array_by_index(i) {
                if let Some(subset) = subset_array(arr, &included_cell_ids) {
                    pd.cell_data_mut().add_array(subset);
                }
            }
        }
    }

    pd
}

/// Convert a RectilinearGrid surface to PolyData.
pub fn rectilinear_grid_to_poly_data(rg: &RectilinearGrid) -> PolyData {
    let dims = rg.dimensions();
    let mut points = Points::new();
    let x = rg.x_coords();
    let y = rg.y_coords();
    let z = rg.z_coords();

    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                points.push([x[i], y[j], z[k]]);
            }
        }
    }

    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let idx =
        |i: usize, j: usize, k: usize| -> i64 { (k * dims[1] * dims[0] + j * dims[0] + i) as i64 };

    if let Some(cell_ids) = push_lower_dim_grid_cells(dims, idx, &mut verts, &mut lines, &mut polys)
    {
        let mut pd = PolyData::new();
        pd.points = points;
        pd.verts = verts;
        pd.lines = lines;
        pd.polys = polys;
        *pd.point_data_mut() = rg.point_data().clone();
        copy_subset_cell_data(rg.cell_data(), &cell_ids, pd.cell_data_mut());
        return pd;
    }

    // Just do all faces of all cells (no shared face elimination)
    let mut surface_cell_ids = Vec::new();
    for k in 0..dims[2].saturating_sub(1) {
        for j in 0..dims[1].saturating_sub(1) {
            for i in 0..dims[0].saturating_sub(1) {
                let cell_id = structured_cell_idx(dims, i, j, k);
                // Top and bottom
                if k == 0 {
                    polys.push_cell(&[
                        idx(i, j, 0),
                        idx(i + 1, j, 0),
                        idx(i + 1, j + 1, 0),
                        idx(i, j + 1, 0),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if k == dims[2] - 2 {
                    polys.push_cell(&[
                        idx(i, j, k + 1),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                // Front and back
                if j == 0 {
                    polys.push_cell(&[
                        idx(i, 0, k),
                        idx(i + 1, 0, k),
                        idx(i + 1, 0, k + 1),
                        idx(i, 0, k + 1),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if j == dims[1] - 2 {
                    polys.push_cell(&[
                        idx(i, j + 1, k),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j + 1, k),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                // Left and right
                if i == 0 {
                    polys.push_cell(&[
                        idx(0, j, k),
                        idx(0, j, k + 1),
                        idx(0, j + 1, k + 1),
                        idx(0, j + 1, k),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if i == dims[0] - 2 {
                    polys.push_cell(&[
                        idx(i + 1, j, k),
                        idx(i + 1, j + 1, k),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    *pd.point_data_mut() = rg.point_data().clone();
    copy_subset_cell_data(rg.cell_data(), &surface_cell_ids, pd.cell_data_mut());
    pd
}

/// Convert a StructuredGrid surface to PolyData quads (outer faces).
pub fn structured_grid_to_poly_data(sg: &StructuredGrid) -> PolyData {
    let dims = sg.dimensions();
    let mut pd = PolyData::new();

    // Copy all points
    for i in 0..sg.points.len() {
        pd.points.push(sg.points.get(i));
    }

    let idx =
        |i: usize, j: usize, k: usize| -> i64 { (k * dims[1] * dims[0] + j * dims[0] + i) as i64 };

    if let Some(cell_ids) =
        push_lower_dim_grid_cells(dims, idx, &mut pd.verts, &mut pd.lines, &mut pd.polys)
    {
        *pd.point_data_mut() = sg.point_data().clone();
        copy_subset_cell_data(sg.cell_data(), &cell_ids, pd.cell_data_mut());
        return pd;
    }

    // Outer faces only
    let mut surface_cell_ids = Vec::new();
    for k in 0..dims[2].saturating_sub(1) {
        for j in 0..dims[1].saturating_sub(1) {
            for i in 0..dims[0].saturating_sub(1) {
                let cell_id = structured_cell_idx(dims, i, j, k);
                if k == 0 {
                    pd.polys.push_cell(&[
                        idx(i, j, 0),
                        idx(i + 1, j, 0),
                        idx(i + 1, j + 1, 0),
                        idx(i, j + 1, 0),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if k == dims[2].saturating_sub(2) {
                    pd.polys.push_cell(&[
                        idx(i, j, k + 1),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if j == 0 {
                    pd.polys.push_cell(&[
                        idx(i, 0, k),
                        idx(i + 1, 0, k),
                        idx(i + 1, 0, k + 1),
                        idx(i, 0, k + 1),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if j == dims[1].saturating_sub(2) {
                    pd.polys.push_cell(&[
                        idx(i, j + 1, k),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j + 1, k),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if i == 0 {
                    pd.polys.push_cell(&[
                        idx(0, j, k),
                        idx(0, j, k + 1),
                        idx(0, j + 1, k + 1),
                        idx(0, j + 1, k),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
                if i == dims[0].saturating_sub(2) {
                    pd.polys.push_cell(&[
                        idx(i + 1, j, k),
                        idx(i + 1, j + 1, k),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                    surface_cell_ids.push(cell_id);
                }
            }
        }
    }

    *pd.point_data_mut() = sg.point_data().clone();
    copy_subset_cell_data(sg.cell_data(), &surface_cell_ids, pd.cell_data_mut());
    pd
}

/// Convert an ImageData to a StructuredGrid (explicit point coordinates).
pub fn image_data_to_structured_grid(img: &ImageData) -> StructuredGrid {
    let dims = img.dimensions();
    let mut pts = Points::new();
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                pts.push(img.point_from_ijk(i, j, k));
            }
        }
    }
    let mut sg = StructuredGrid::from_dimensions_and_points(dims, pts);
    *sg.point_data_mut() = img.point_data().clone();
    *sg.cell_data_mut() = img.cell_data().clone();
    sg
}

/// Convert a StructuredGrid to an ImageData (only if the grid is regular).
///
/// Returns None if the grid is not axis-aligned with uniform spacing.
pub fn structured_grid_to_image_data(sg: &StructuredGrid) -> Option<ImageData> {
    let dims = sg.dimensions();
    if dims[0] < 2 || dims[1] < 2 || dims[2] < 2 {
        return None;
    }

    let p000 = sg.points.get(sg.index_from_ijk(0, 0, 0));
    let p100 = sg.points.get(sg.index_from_ijk(1, 0, 0));
    let p010 = sg.points.get(sg.index_from_ijk(0, 1, 0));
    let p001 = sg.points.get(sg.index_from_ijk(0, 0, 1));

    let spacing = [p100[0] - p000[0], p010[1] - p000[1], p001[2] - p000[2]];

    // Verify it's actually regular
    if spacing[0].abs() < 1e-15 || spacing[1].abs() < 1e-15 || spacing[2].abs() < 1e-15 {
        return None;
    }

    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                let p = sg.points.get(sg.index_from_ijk(i, j, k));
                let expected = [
                    p000[0] + i as f64 * spacing[0],
                    p000[1] + j as f64 * spacing[1],
                    p000[2] + k as f64 * spacing[2],
                ];
                if (p[0] - expected[0]).abs() > 1e-12
                    || (p[1] - expected[1]).abs() > 1e-12
                    || (p[2] - expected[2]).abs() > 1e-12
                {
                    return None;
                }
            }
        }
    }

    let mut img = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(p000);
    *img.point_data_mut() = sg.point_data().clone();
    *img.cell_data_mut() = sg.cell_data().clone();
    Some(img)
}

fn push_lower_dim_grid_cells<F>(
    dims: [usize; 3],
    idx: F,
    verts: &mut CellArray,
    lines: &mut CellArray,
    polys: &mut CellArray,
) -> Option<Vec<usize>>
where
    F: Fn(usize, usize, usize) -> i64,
{
    if dims.contains(&0) {
        return Some(Vec::new());
    }

    let active_axes: Vec<usize> = (0..3).filter(|&axis| dims[axis] > 1).collect();
    let mut cell_ids = Vec::new();

    match active_axes.as_slice() {
        [] => {
            verts.push_cell(&[idx(0, 0, 0)]);
            cell_ids.push(0);
            Some(cell_ids)
        }
        [axis] => {
            for n in 0..dims[*axis] - 1 {
                let mut a = [0usize; 3];
                let mut b = [0usize; 3];
                a[*axis] = n;
                b[*axis] = n + 1;
                lines.push_cell(&[idx(a[0], a[1], a[2]), idx(b[0], b[1], b[2])]);
                cell_ids.push(structured_cell_idx(dims, a[0], a[1], a[2]));
            }
            Some(cell_ids)
        }
        [0, 1] => {
            for j in 0..dims[1] - 1 {
                for i in 0..dims[0] - 1 {
                    polys.push_cell(&[
                        idx(i, j, 0),
                        idx(i + 1, j, 0),
                        idx(i + 1, j + 1, 0),
                        idx(i, j + 1, 0),
                    ]);
                    cell_ids.push(structured_cell_idx(dims, i, j, 0));
                }
            }
            Some(cell_ids)
        }
        [0, 2] => {
            for k in 0..dims[2] - 1 {
                for i in 0..dims[0] - 1 {
                    polys.push_cell(&[
                        idx(i, 0, k),
                        idx(i + 1, 0, k),
                        idx(i + 1, 0, k + 1),
                        idx(i, 0, k + 1),
                    ]);
                    cell_ids.push(structured_cell_idx(dims, i, 0, k));
                }
            }
            Some(cell_ids)
        }
        [1, 2] => {
            for k in 0..dims[2] - 1 {
                for j in 0..dims[1] - 1 {
                    polys.push_cell(&[
                        idx(0, j, k),
                        idx(0, j, k + 1),
                        idx(0, j + 1, k + 1),
                        idx(0, j + 1, k),
                    ]);
                    cell_ids.push(structured_cell_idx(dims, 0, j, k));
                }
            }
            Some(cell_ids)
        }
        _ => None,
    }
}

fn structured_cell_idx(dims: [usize; 3], i: usize, j: usize, k: usize) -> usize {
    let cx = dims[0].saturating_sub(1).max(1);
    let cy = dims[1].saturating_sub(1).max(1);
    k * cx * cy + j * cx + i
}

fn copy_subset_cell_data(
    input: &crate::data::DataSetAttributes,
    tuple_ids: &[usize],
    output: &mut crate::data::DataSetAttributes,
) {
    for i in 0..input.num_arrays() {
        if let Some(arr) = input.get_array_by_index(i) {
            if let Some(subset) = subset_array(arr, tuple_ids) {
                output.add_array(subset);
            }
        }
    }
    copy_active_attributes(input, output);
}

fn copy_active_attributes(
    input: &crate::data::DataSetAttributes,
    output: &mut crate::data::DataSetAttributes,
) {
    if let Some(array) = input.scalars() {
        if output.has_array(array.name()) {
            output.set_active_scalars(array.name());
        }
    }
    if let Some(array) = input.vectors() {
        if output.has_array(array.name()) {
            output.set_active_vectors(array.name());
        }
    }
    if let Some(array) = input.normals() {
        if output.has_array(array.name()) {
            output.set_active_normals(array.name());
        }
    }
    if let Some(array) = input.tcoords() {
        if output.has_array(array.name()) {
            output.set_active_tcoords(array.name());
        }
    }
    if let Some(array) = input.tensors() {
        if output.has_array(array.name()) {
            output.set_active_tensors(array.name());
        }
    }
    if let Some(array) = input.global_ids() {
        if output.has_array(array.name()) {
            output.set_active_global_ids(array.name());
        }
    }
    if let Some(array) = input.pedigree_ids() {
        if output.has_array(array.name()) {
            output.set_active_pedigree_ids(array.name());
        }
    }
    if let Some(array) = input.edge_flags() {
        if output.has_array(array.name()) {
            output.set_active_edge_flags(array.name());
        }
    }
    if let Some(array) = input.tangents() {
        if output.has_array(array.name()) {
            output.set_active_tangents(array.name());
        }
    }
    if let Some(array) = input.rational_weights() {
        if output.has_array(array.name()) {
            output.set_active_rational_weights(array.name());
        }
    }
    if let Some(array) = input.higher_order_degrees() {
        if output.has_array(array.name()) {
            output.set_active_higher_order_degrees(array.name());
        }
    }
    if let Some(array) = input.process_ids() {
        if output.has_array(array.name()) {
            output.set_active_process_ids(array.name());
        }
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            let nc = a.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                if tuple_id >= a.num_tuples() {
                    return None;
                }
                data.extend_from_slice(a.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                a.name(),
                data,
                nc,
            )))
        }};
    }

    match array {
        AnyDataArray::F32(_) => subset_variant!(F32),
        AnyDataArray::F64(_) => subset_variant!(F64),
        AnyDataArray::I8(_) => subset_variant!(I8),
        AnyDataArray::I16(_) => subset_variant!(I16),
        AnyDataArray::I32(_) => subset_variant!(I32),
        AnyDataArray::I64(_) => subset_variant!(I64),
        AnyDataArray::U8(_) => subset_variant!(U8),
        AnyDataArray::U16(_) => subset_variant!(U16),
        AnyDataArray::U32(_) => subset_variant!(U32),
        AnyDataArray::U64(_) => subset_variant!(U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_to_poly_surface() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let pd = image_data_surface_to_poly_data(&img);
        assert_eq!(pd.points.len(), 27);
        assert!(pd.polys.num_cells() > 0); // should have surface quads
    }

    #[test]
    fn poly_to_unstructured() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let ug = poly_data_to_unstructured_grid(&pd);
        assert_eq!(ug.points.len(), 3);
        assert_eq!(ug.cells().num_cells(), 1);
        assert_eq!(ug.cell_type(0), CellType::Triangle);
    }

    #[test]
    fn unstructured_to_poly() {
        let mut ug = UnstructuredGrid::new();
        ug.points.push([0.0, 0.0, 0.0]);
        ug.points.push([1.0, 0.0, 0.0]);
        ug.points.push([0.0, 1.0, 0.0]);
        ug.push_cell(CellType::Triangle, &[0, 1, 2]);
        let pd = unstructured_grid_to_poly_data(&ug);
        assert_eq!(pd.polys.num_cells(), 1);
    }

    #[test]
    fn rectilinear_to_poly() {
        let rg = RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![0.0, 1.0]);
        let pd = rectilinear_grid_to_poly_data(&rg);
        assert_eq!(pd.points.len(), 12); // 3*2*2
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn structured_to_poly() {
        let sg = StructuredGrid::uniform([3, 2, 2], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let pd = structured_grid_to_poly_data(&sg);
        assert_eq!(pd.points.len(), 12);
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn image_to_structured() {
        let mut img = ImageData::with_dimensions(3, 3, 3)
            .with_spacing([0.5, 0.5, 0.5])
            .with_origin([1.0, 2.0, 3.0]);
        img.point_data_mut()
            .add_array(crate::data::AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0; 27],
                1,
            )));
        let sg = image_data_to_structured_grid(&img);
        assert_eq!(sg.dimensions(), [3, 3, 3]);
        assert_eq!(sg.points.len(), 27);
        let p = sg.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!(sg.point_data().get_array("s").is_some());
    }

    #[test]
    fn roundtrip_image_structured() {
        let img = ImageData::with_dimensions(4, 3, 2)
            .with_spacing([0.5, 1.0, 2.0])
            .with_origin([1.0, 2.0, 3.0]);
        let sg = image_data_to_structured_grid(&img);
        let img2 = structured_grid_to_image_data(&sg).unwrap();
        assert_eq!(img2.dimensions(), img.dimensions());
        assert!((img2.spacing()[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn structured_to_image_rejects_sheared_grid() {
        let mut sg = StructuredGrid::uniform([3, 3, 3], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let idx = sg.index_from_ijk(1, 1, 1);
        let mut p = sg.points.get(idx);
        p[0] += 0.25;
        sg.points.set(idx, p);

        assert!(structured_grid_to_image_data(&sg).is_none());
    }

    #[test]
    fn roundtrip_poly_unstructured() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let ug = poly_data_to_unstructured_grid(&pd);
        let pd2 = unstructured_grid_to_poly_data(&ug);
        assert_eq!(pd2.polys.num_cells(), 2);
    }

    #[test]
    fn poly_to_unstructured_preserves_vtk_cell_order_and_strips() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.verts.push_cell(&[0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);
        pd.lines.push_cell(&[1, 2]);

        let ug = poly_data_to_unstructured_grid(&pd);

        assert_eq!(
            ug.cell_types(),
            &[
                CellType::Vertex,
                CellType::Line,
                CellType::Triangle,
                CellType::TriangleStrip,
            ]
        );
        assert_eq!(ug.cell_points(3), &[0, 1, 2, 3]);
    }

    #[test]
    fn poly_to_unstructured_copies_cell_data() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.cell_data_mut().add_array(crate::data::AnyDataArray::I32(
            crate::data::DataArray::from_vec("cell_ids", vec![7], 1),
        ));

        let ug = poly_data_to_unstructured_grid(&pd);

        let ids = ug.cell_data().get_array("cell_ids").unwrap();
        let mut value = [0.0];
        ids.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 7.0);
    }

    #[test]
    fn unstructured_to_poly_preserves_triangle_strips() {
        let mut ug = UnstructuredGrid::new();
        ug.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        ug.push_cell(CellType::TriangleStrip, &[0, 1, 2, 3]);

        let pd = unstructured_grid_to_poly_data(&ug);

        assert_eq!(pd.strips.num_cells(), 1);
        assert_eq!(pd.polys.num_cells(), 0);
        assert_eq!(pd.strips.cell(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn unstructured_to_poly_copies_cell_data_for_supported_cells() {
        let mut ug = UnstructuredGrid::new();
        ug.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]);
        ug.push_cell(CellType::Triangle, &[0, 1, 2]);
        ug.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        ug.cell_data_mut().add_array(crate::data::AnyDataArray::I32(
            crate::data::DataArray::from_vec("ids", vec![11, 22], 1),
        ));

        let pd = unstructured_grid_to_poly_data(&ug);

        assert_eq!(pd.polys.num_cells(), 1);
        let ids = pd.cell_data().get_array("ids").unwrap();
        let mut value = [0.0];
        ids.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 11.0);
    }

    #[test]
    fn surface_subset_cell_data_preserves_active_scalars() {
        let mut rg = RectilinearGrid::from_coords(vec![0.0, 1.0], vec![0.0, 1.0], vec![0.0, 1.0]);
        rg.cell_data_mut().add_array(crate::data::AnyDataArray::F64(
            crate::data::DataArray::from_vec("cell_ids", vec![5.0], 1),
        ));
        rg.cell_data_mut().set_active_scalars("cell_ids");

        let pd = rectilinear_grid_to_poly_data(&rg);

        let scalars = pd.cell_data().scalars().unwrap();
        assert_eq!(scalars.name(), "cell_ids");
        assert_eq!(scalars.num_tuples(), pd.polys.num_cells());
    }

    #[test]
    fn unstructured_to_poly_does_not_emit_volume_cells_as_polygons() {
        let ug = UnstructuredGrid::from_tetrahedra(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2, 3]],
        );

        let pd = unstructured_grid_to_poly_data(&ug);

        assert_eq!(pd.polys.num_cells(), 0);
    }

    #[test]
    fn image_surface_handles_flat_dimensions() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let pd = image_data_surface_to_poly_data(&img);
        assert_eq!(pd.points.len(), 9);
        assert_eq!(pd.polys.num_cells(), 4);
    }

    #[test]
    fn image_surface_handles_line_dimensions() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let pd = image_data_surface_to_poly_data(&img);
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.lines.num_cells(), 2);
        assert_eq!(pd.polys.num_cells(), 0);
    }

    #[test]
    fn image_surface_handles_empty_dimensions() {
        let img = ImageData::with_dimensions(0, 3, 3);
        let pd = image_data_surface_to_poly_data(&img);
        assert_eq!(pd.points.len(), 0);
        assert_eq!(pd.polys.num_cells(), 0);
    }

    #[test]
    fn rectilinear_surface_handles_flat_dimensions() {
        let rg = RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0], vec![0.0]);
        let pd = rectilinear_grid_to_poly_data(&rg);
        assert_eq!(pd.points.len(), 9);
        assert_eq!(pd.polys.num_cells(), 4);
    }

    #[test]
    fn rectilinear_surface_handles_line_dimensions() {
        let rg = RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0], vec![0.0]);
        let pd = rectilinear_grid_to_poly_data(&rg);
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.lines.num_cells(), 2);
        assert_eq!(pd.polys.num_cells(), 0);
    }

    #[test]
    fn structured_surface_handles_flat_dimensions() {
        let sg = StructuredGrid::uniform([3, 3, 1], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let pd = structured_grid_to_poly_data(&sg);
        assert_eq!(pd.points.len(), 9);
        assert_eq!(pd.polys.num_cells(), 4);
    }

    #[test]
    fn structured_surface_handles_line_dimensions() {
        let sg = StructuredGrid::uniform([3, 1, 1], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let pd = structured_grid_to_poly_data(&sg);
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.lines.num_cells(), 2);
        assert_eq!(pd.polys.num_cells(), 0);
    }
}
