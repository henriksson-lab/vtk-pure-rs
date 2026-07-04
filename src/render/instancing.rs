use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};

/// An instance of a glyph placed at a position with optional scale and color.
#[derive(Debug, Clone)]
pub struct GlyphInstance {
    /// World-space position.
    pub position: [f32; 3],
    /// Uniform scale factor.
    pub scale: f32,
    /// Override color (RGB). If None, uses the glyph's original coloring.
    pub color: Option<[f32; 3]>,
}

/// A set of glyph instances sharing the same template mesh.
///
/// For efficient rendering of many copies of the same geometry at different
/// positions (e.g., arrow glyphs at vector field points, sphere glyphs at
/// point locations).
#[derive(Debug, Clone)]
pub struct InstancedGlyphs {
    /// Template mesh to instance.
    pub template: PolyData,
    /// Per-instance transforms.
    pub instances: Vec<GlyphInstance>,
}

impl InstancedGlyphs {
    /// Create a new instanced glyph set.
    pub fn new(template: PolyData) -> Self {
        Self {
            template,
            instances: Vec::new(),
        }
    }

    /// Add an instance at the given position with unit scale.
    pub fn add(&mut self, position: [f32; 3]) {
        self.instances.push(GlyphInstance {
            position,
            scale: 1.0,
            color: None,
        });
    }

    /// Add an instance with position, scale, and color.
    pub fn add_with(&mut self, position: [f32; 3], scale: f32, color: [f32; 3]) {
        self.instances.push(GlyphInstance {
            position,
            scale,
            color: Some(color),
        });
    }

    /// Flatten all instances into a single PolyData by copying and transforming
    /// the template mesh for each instance.
    ///
    /// This is a CPU-side approach suitable for moderate instance counts.
    /// For very large counts, GPU instancing should be used.
    pub fn flatten(&self) -> PolyData {
        self.to_flat_poly_data(&self.template)
    }

    /// Flatten a given glyph mesh into a single PolyData by copying and
    /// transforming it for each instance. Unlike `flatten()`, this uses an
    /// externally provided glyph rather than `self.template`.
    ///
    /// This enables CPU-side instancing for renderers without GPU instancing.
    pub fn to_flat_poly_data(&self, glyph: &PolyData) -> PolyData {
        let tpl_npts = glyph.points.len();
        if tpl_npts == 0 || self.instances.is_empty() {
            return PolyData::new();
        }

        let mut result = PolyData::new();

        for inst in &self.instances {
            let base = result.points.len() as i64;

            // Copy and transform points
            for i in 0..tpl_npts {
                let p = glyph.points.get(i);
                result.points.push([
                    p[0] * inst.scale as f64 + inst.position[0] as f64,
                    p[1] * inst.scale as f64 + inst.position[1] as f64,
                    p[2] * inst.scale as f64 + inst.position[2] as f64,
                ]);
            }

            copy_cells_with_offset(&glyph.verts, &mut result.verts, base);
            copy_cells_with_offset(&glyph.lines, &mut result.lines, base);
            copy_cells_with_offset(&glyph.polys, &mut result.polys, base);
            copy_cells_with_offset(&glyph.strips, &mut result.strips, base);
        }

        copy_repeated_attributes(
            glyph.point_data(),
            result.point_data_mut(),
            tpl_npts,
            self.instances.len(),
        );
        copy_repeated_attributes(
            glyph.cell_data(),
            result.cell_data_mut(),
            glyph.total_cells(),
            self.instances.len(),
        );

        result
    }

    /// Number of instances.
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Whether the instance set is empty.
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

fn copy_cells_with_offset(src: &CellArray, dst: &mut CellArray, base: i64) {
    for cell in src.iter() {
        let offset_cell: Vec<i64> = cell.iter().map(|&id| id + base).collect();
        dst.push_cell(&offset_cell);
    }
}

fn copy_repeated_attributes(
    src: &DataSetAttributes,
    dst: &mut DataSetAttributes,
    tuple_count: usize,
    repetitions: usize,
) {
    for array in src.iter() {
        if let Some(repeated) = repeat_array_tuples(array, tuple_count, repetitions) {
            dst.add_array(repeated);
        }
    }

    if let Some(name) = src.scalars().map(AnyDataArray::name) {
        dst.set_active_scalars(name);
    }
    if let Some(name) = src.vectors().map(AnyDataArray::name) {
        dst.set_active_vectors(name);
    }
    if let Some(name) = src.normals().map(AnyDataArray::name) {
        dst.set_active_normals(name);
    }
    if let Some(name) = src.tcoords().map(AnyDataArray::name) {
        dst.set_active_tcoords(name);
    }
    if let Some(name) = src.tensors().map(AnyDataArray::name) {
        dst.set_active_tensors(name);
    }
    if let Some(name) = src.global_ids().map(AnyDataArray::name) {
        dst.set_active_global_ids(name);
    }
    if let Some(name) = src.pedigree_ids().map(AnyDataArray::name) {
        dst.set_active_pedigree_ids(name);
    }
    if let Some(name) = src.edge_flags().map(AnyDataArray::name) {
        dst.set_active_edge_flags(name);
    }
    if let Some(name) = src.tangents().map(AnyDataArray::name) {
        dst.set_active_tangents(name);
    }
    if let Some(name) = src.rational_weights().map(AnyDataArray::name) {
        dst.set_active_rational_weights(name);
    }
    if let Some(name) = src.higher_order_degrees().map(AnyDataArray::name) {
        dst.set_active_higher_order_degrees(name);
    }
    if let Some(name) = src.process_ids().map(AnyDataArray::name) {
        dst.set_active_process_ids(name);
    }
}

fn repeat_array_tuples(
    array: &AnyDataArray,
    tuple_count: usize,
    repetitions: usize,
) -> Option<AnyDataArray> {
    macro_rules! repeat {
        ($variant:ident, $array:expr) => {{
            if $array.num_tuples() != tuple_count {
                return None;
            }
            let mut data = Vec::with_capacity($array.len() * repetitions);
            for _ in 0..repetitions {
                data.extend_from_slice($array.as_slice());
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $array.name(),
                data,
                $array.num_components(),
            )))
        }};
    }

    match array {
        AnyDataArray::F32(a) => repeat!(F32, a),
        AnyDataArray::F64(a) => repeat!(F64, a),
        AnyDataArray::I8(a) => repeat!(I8, a),
        AnyDataArray::I16(a) => repeat!(I16, a),
        AnyDataArray::I32(a) => repeat!(I32, a),
        AnyDataArray::I64(a) => repeat!(I64, a),
        AnyDataArray::U8(a) => repeat!(U8, a),
        AnyDataArray::U16(a) => repeat!(U16, a),
        AnyDataArray::U32(a) => repeat!(U32, a),
        AnyDataArray::U64(a) => repeat!(U64, a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_instances() {
        let template = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut glyphs = InstancedGlyphs::new(template);
        glyphs.add([0.0, 0.0, 0.0]);
        glyphs.add([5.0, 0.0, 0.0]);
        glyphs.add([0.0, 5.0, 0.0]);

        let result = glyphs.flatten();
        assert_eq!(result.points.len(), 9); // 3 instances * 3 points
        assert_eq!(result.polys.num_cells(), 3);

        // Second instance should be offset by (5,0,0)
        let p = result.points.get(3);
        assert!((p[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn scaled_instances() {
        let template = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut glyphs = InstancedGlyphs::new(template);
        glyphs.add_with([0.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0]);

        let result = glyphs.flatten();
        let p = result.points.get(1);
        assert!((p[0] - 2.0).abs() < 1e-10); // scaled by 2
    }

    #[test]
    fn to_flat_poly_data_external_glyph() {
        let template = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // Use a different glyph for flattening
        let glyph = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut glyphs = InstancedGlyphs::new(template);
        glyphs.add([0.0, 0.0, 0.0]);
        glyphs.add([10.0, 0.0, 0.0]);

        let result = glyphs.to_flat_poly_data(&glyph);
        assert_eq!(result.points.len(), 6); // 2 instances * 3 points
        assert_eq!(result.polys.num_cells(), 2);

        // Second instance offset check
        let p = result.points.get(4); // second instance, point index 1 (2.0, 0, 0) + (10, 0, 0)
        assert!((p[0] - 12.0).abs() < 1e-10);
    }

    #[test]
    fn empty_instances() {
        let template = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let glyphs = InstancedGlyphs::new(template);
        assert!(glyphs.is_empty());
        let result = glyphs.flatten();
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn flatten_preserves_all_poly_data_cell_arrays() {
        let mut template = PolyData::new();
        template.points.push([0.0, 0.0, 0.0]);
        template.points.push([1.0, 0.0, 0.0]);
        template.points.push([1.0, 1.0, 0.0]);
        template.points.push([0.0, 1.0, 0.0]);
        template.verts.push_cell(&[0]);
        template.lines.push_cell(&[0, 1]);
        template.polys.push_cell(&[0, 1, 2]);
        template.strips.push_cell(&[0, 1, 2, 3]);

        let mut glyphs = InstancedGlyphs::new(template);
        glyphs.add([0.0, 0.0, 0.0]);
        glyphs.add([10.0, 0.0, 0.0]);

        let result = glyphs.flatten();
        assert_eq!(result.verts.num_cells(), 2);
        assert_eq!(result.lines.num_cells(), 2);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.strips.num_cells(), 2);
        assert_eq!(result.lines.cell(1), &[4, 5]);
    }
}
