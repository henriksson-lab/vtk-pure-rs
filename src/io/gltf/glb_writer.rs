use std::io::Write;
use std::path::Path;

use crate::data::PolyData;
use crate::types::VtkError;

/// Writer for binary glTF (.glb) format.
///
/// Exports PolyData meshes as glTF 2.0 binary files.
/// Supports positions, normals (if available), points, lines, and triangles.
pub struct GlbWriter;

impl GlbWriter {
    /// Write a PolyData mesh to a .glb file.
    pub fn write(path: &Path, poly_data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        Self::write_to(&mut writer, poly_data)
    }

    /// Write a PolyData mesh to a writer in .glb format.
    pub fn write_to<W: Write>(writer: &mut W, poly_data: &PolyData) -> Result<(), VtkError> {
        let (json, bin) = build_glb_data(poly_data)?;
        write_glb(writer, &json, &bin)
    }
}

fn build_glb_data(pd: &PolyData) -> Result<(Vec<u8>, Vec<u8>), VtkError> {
    let n_points = pd.points.len();
    if n_points == 0 {
        return Err(VtkError::InvalidData("empty PolyData".into()));
    }

    // Build binary buffer
    let mut bin = Vec::new();
    let mut accessors = Vec::new();
    let mut buffer_views = Vec::new();
    let mut attributes = Vec::new();
    let mut primitives = Vec::new();

    // Positions
    let pos_offset = bin.len();
    let mut min_pos = [f32::INFINITY; 3];
    let mut max_pos = [f32::NEG_INFINITY; 3];
    for i in 0..n_points {
        let p = pd.points.get(i);
        let pos = [p[0] as f32, p[1] as f32, p[2] as f32];
        if !pos.iter().all(|value| value.is_finite()) {
            return Err(VtkError::InvalidData(
                "glTF positions must be finite Float32 values".into(),
            ));
        }
        for k in 0..3 {
            min_pos[k] = min_pos[k].min(pos[k]);
            max_pos[k] = max_pos[k].max(pos[k]);
        }
        bin.extend_from_slice(&pos[0].to_le_bytes());
        bin.extend_from_slice(&pos[1].to_le_bytes());
        bin.extend_from_slice(&pos[2].to_le_bytes());
    }
    let pos_len = bin.len() - pos_offset;
    pad_to_4(&mut bin);

    let pos_bv_idx = buffer_views.len();
    buffer_views.push(format!(
        r#"{{"buffer":0,"byteOffset":{},"byteLength":{},"byteStride":12,"target":34962}}"#,
        pos_offset, pos_len
    ));
    let pos_acc_idx = accessors.len();
    accessors.push(format!(
        r#"{{"bufferView":{},"componentType":5126,"count":{},"type":"VEC3","max":[{},{},{}],"min":[{},{},{}]}}"#,
        pos_bv_idx,
        n_points,
        max_pos[0], max_pos[1], max_pos[2],
        min_pos[0], min_pos[1], min_pos[2]
    ));
    attributes.push(format!(r#""POSITION":{}"#, pos_acc_idx));

    // Normals (if available)
    let has_normals = pd
        .point_data()
        .normals()
        .is_some_and(|normals| normals.num_tuples() >= n_points);
    if has_normals {
        let normals = pd.point_data().normals().unwrap();
        let norm_bv_idx = buffer_views.len();
        let norm_acc_idx = accessors.len();
        let norm_offset = bin.len();
        let mut buf = [0.0f64; 3];
        for i in 0..normals.num_tuples().min(n_points) {
            normals.tuple_as_f64(i, &mut buf);
            bin.extend_from_slice(&(buf[0] as f32).to_le_bytes());
            bin.extend_from_slice(&(buf[1] as f32).to_le_bytes());
            bin.extend_from_slice(&(buf[2] as f32).to_le_bytes());
        }
        let norm_len = bin.len() - norm_offset;
        pad_to_4(&mut bin);

        buffer_views.push(format!(
            r#"{{"buffer":0,"byteOffset":{},"byteLength":{},"byteStride":12,"target":34962}}"#,
            norm_offset, norm_len
        ));
        accessors.push(format!(
            r#"{{"bufferView":{},"componentType":5126,"count":{},"type":"VEC3"}}"#,
            norm_bv_idx,
            normals.num_tuples().min(n_points)
        ));
        attributes.push(format!(r#""NORMAL":{}"#, norm_acc_idx));
    }

    let attrs_str = attributes.join(",");

    let mut point_indices = Vec::new();
    for cell in pd.verts.iter() {
        for &pid in cell {
            push_index(&mut point_indices, pid, n_points)?;
        }
    }
    if !point_indices.is_empty() {
        let index_acc =
            write_indices_accessor(&mut bin, &mut buffer_views, &mut accessors, &point_indices);
        primitives.push(format!(
            r#"{{"attributes":{{{}}},"indices":{},"mode":0}}"#,
            attrs_str, index_acc
        ));
    }

    let mut line_indices = Vec::new();
    for cell in pd.lines.iter() {
        for segment in cell.windows(2) {
            push_index(&mut line_indices, segment[0], n_points)?;
            push_index(&mut line_indices, segment[1], n_points)?;
        }
    }
    if !line_indices.is_empty() {
        let index_acc =
            write_indices_accessor(&mut bin, &mut buffer_views, &mut accessors, &line_indices);
        primitives.push(format!(
            r#"{{"attributes":{{{}}},"indices":{},"mode":1}}"#,
            attrs_str, index_acc
        ));
    }

    let mut triangle_indices = Vec::new();
    for cell in pd.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 1..cell.len() - 1 {
            push_index(&mut triangle_indices, cell[0], n_points)?;
            push_index(&mut triangle_indices, cell[i], n_points)?;
            push_index(&mut triangle_indices, cell[i + 1], n_points)?;
        }
    }
    if !triangle_indices.is_empty() {
        let index_acc = write_indices_accessor(
            &mut bin,
            &mut buffer_views,
            &mut accessors,
            &triangle_indices,
        );
        primitives.push(format!(
            r#"{{"attributes":{{{}}},"indices":{},"mode":4}}"#,
            attrs_str, index_acc
        ));
    }

    if primitives.is_empty() {
        return Err(VtkError::InvalidData(
            "no vertices, lines, or triangles in PolyData".into(),
        ));
    }

    // Build JSON
    let primitives_str = primitives.join(",");
    let accessors_str = accessors.join(",");
    let buffer_views_str = buffer_views.join(",");

    let json_str = format!(
        concat!(
            r#"{{"asset":{{"version":"2.0","generator":"vtk-rs"}},"#,
            r#""scene":0,"scenes":[{{"nodes":[0]}}],"#,
            r#""nodes":[{{"mesh":0}}],"#,
            r#""meshes":[{{"primitives":[{}]}}],"#,
            r#""accessors":[{}],"bufferViews":[{}],"buffers":[{{"byteLength":{}}}]}}"#
        ),
        primitives_str,
        accessors_str,
        buffer_views_str,
        bin.len()
    );

    let json_bytes = json_str.into_bytes();
    Ok((json_bytes, bin))
}

fn push_index(indices: &mut Vec<u32>, pid: i64, point_count: usize) -> Result<(), VtkError> {
    if pid < 0 || pid as usize >= point_count {
        return Err(VtkError::InvalidData(format!(
            "cell point id {pid} is outside point range 0..{point_count}"
        )));
    }
    indices.push(pid as u32);
    Ok(())
}

fn write_indices_accessor(
    bin: &mut Vec<u8>,
    buffer_views: &mut Vec<String>,
    accessors: &mut Vec<String>,
    indices: &[u32],
) -> usize {
    let indices_offset = bin.len();
    for &idx in indices {
        bin.extend_from_slice(&idx.to_le_bytes());
    }
    let indices_len = bin.len() - indices_offset;
    pad_to_4(bin);
    let idx_max = indices.iter().copied().max().unwrap_or(0);

    let buffer_view_idx = buffer_views.len();
    buffer_views.push(format!(
        r#"{{"buffer":0,"byteOffset":{},"byteLength":{},"target":34963}}"#,
        indices_offset, indices_len
    ));
    let accessor_idx = accessors.len();
    accessors.push(format!(
        r#"{{"bufferView":{},"componentType":5125,"count":{},"type":"SCALAR","max":[{}],"min":[0]}}"#,
        buffer_view_idx,
        indices.len(),
        idx_max
    ));
    accessor_idx
}

fn write_glb<W: Write>(writer: &mut W, json: &[u8], bin: &[u8]) -> Result<(), VtkError> {
    let json_padded_len = (json.len() + 3) & !3;
    let bin_padded_len = (bin.len() + 3) & !3;
    let total_len = 12 + 8 + json_padded_len + 8 + bin_padded_len;

    // GLB header
    writer.write_all(b"glTF")?; // magic
    writer.write_all(&2u32.to_le_bytes())?; // version
    writer.write_all(&(total_len as u32).to_le_bytes())?; // length

    // JSON chunk
    writer.write_all(&(json_padded_len as u32).to_le_bytes())?;
    writer.write_all(&0x4E4F534Au32.to_le_bytes())?; // "JSON"
    writer.write_all(json)?;
    for _ in 0..(json_padded_len - json.len()) {
        writer.write_all(b" ")?;
    }

    // BIN chunk
    writer.write_all(&(bin_padded_len as u32).to_le_bytes())?;
    writer.write_all(&0x004E4942u32.to_le_bytes())?; // "BIN\0"
    writer.write_all(bin)?;
    for _ in 0..(bin_padded_len - bin.len()) {
        writer.write_all(&[0u8])?;
    }

    Ok(())
}

fn pad_to_4(buf: &mut Vec<u8>) {
    while buf.len() % 4 != 0 {
        buf.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_triangle_glb() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        GlbWriter::write_to(&mut buf, &pd).unwrap();

        // Check GLB magic
        assert_eq!(&buf[0..4], b"glTF");
        // Check version
        assert_eq!(u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]), 2);
        // Check total length matches
        let total = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]) as usize;
        assert_eq!(total, buf.len());
    }

    #[test]
    fn write_quad_glb() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let mut buf = Vec::new();
        GlbWriter::write_to(&mut buf, &pd).unwrap();
        assert_eq!(&buf[0..4], b"glTF");
    }

    #[test]
    fn empty_poly_data_error() {
        let pd = PolyData::new();
        let mut buf = Vec::new();
        assert!(GlbWriter::write_to(&mut buf, &pd).is_err());
    }

    #[test]
    fn json_chunk_contains_asset() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        GlbWriter::write_to(&mut buf, &pd).unwrap();

        // Extract JSON chunk
        let json_len = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as usize;
        let json_str = std::str::from_utf8(&buf[20..20 + json_len]).unwrap();
        assert!(json_str.contains("\"version\":\"2.0\""));
        assert!(json_str.contains("\"generator\":\"vtk-rs\""));
        assert!(json_str.contains("POSITION"));
    }

    #[test]
    fn rejects_non_finite_positions() {
        let pd = PolyData::from_triangles(
            vec![[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        assert!(GlbWriter::write_to(&mut buf, &pd).is_err());
    }

    #[test]
    fn rejects_invalid_cell_indices() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 99]],
        );
        let mut buf = Vec::new();
        assert!(GlbWriter::write_to(&mut buf, &pd).is_err());
    }
}
