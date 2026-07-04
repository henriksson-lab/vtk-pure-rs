use std::io::Write;
use std::path::Path;

use crate::data::{DataSetAttributes, ImageData, PolyData};
use crate::types::ScalarType;
use crate::types::VtkError;

/// Writer for XDMF (eXtensible Data Model and Format) files.
///
/// Produces a self-contained `.xdmf` file with inline ASCII data.
/// XDMF is an XML-based format widely supported by ParaView, VisIt, and
/// other visualization tools. It can reference external HDF5 files, but
/// this writer uses inline data for simplicity and zero dependencies.
pub struct XdmfWriter;

impl XdmfWriter {
    /// Write a PolyData mesh as an XDMF file with inline data.
    pub fn write_poly_data(path: &Path, pd: &PolyData) -> Result<(), VtkError> {
        let mut f = std::fs::File::create(path)?;
        Self::write_poly_data_to(&mut f, pd)
    }

    /// Write a PolyData mesh as XDMF to a writer.
    pub fn write_poly_data_to<W: Write>(w: &mut W, pd: &PolyData) -> Result<(), VtkError> {
        let n_pts = pd.points.len();

        let cells = validated_polygon_cells(pd)?;
        let n_cells = cells.len();
        let nodes_per_element = cells.first().map_or(3, Vec::len);
        let homogeneous = cells.iter().all(|cell| cell.len() == nodes_per_element);
        let topology_type = if !homogeneous {
            "Mixed"
        } else if nodes_per_element == 3 {
            "Triangle"
        } else if nodes_per_element == 4 {
            "Quadrilateral"
        } else {
            "Polygon"
        };

        writeln!(w, r#"<?xml version="1.0" ?>"#)?;
        writeln!(w, r#"<Xdmf Version="3.0">"#)?;
        writeln!(w, r#"  <Domain>"#)?;
        writeln!(w, r#"    <Grid Name="mesh" GridType="Uniform">"#)?;

        // Topology
        if topology_type == "Polygon" {
            writeln!(
                w,
                r#"      <Topology TopologyType="{topology_type}" NumberOfElements="{n_cells}" NodesPerElement="{nodes_per_element}">"#
            )?;
        } else {
            writeln!(
                w,
                r#"      <Topology TopologyType="{topology_type}" NumberOfElements="{n_cells}">"#
            )?;
        }
        if topology_type == "Mixed" {
            let mixed = mixed_topology_connectivity(&cells);
            writeln!(
                w,
                r#"        <DataItem Format="XML" DataType="Int" Dimensions="{}">"#,
                mixed.len()
            )?;
            let vals: Vec<String> = mixed.iter().map(|id| id.to_string()).collect();
            writeln!(w, "          {}", vals.join(" "))?;
        } else {
            writeln!(
                w,
                r#"        <DataItem Format="XML" DataType="Int" Dimensions="{n_cells} {nodes_per_element}">"#
            )?;
            for cell in &cells {
                let vals: Vec<String> = cell.iter().map(|id| id.to_string()).collect();
                writeln!(w, "          {}", vals.join(" "))?;
            }
        }
        writeln!(w, r#"        </DataItem>"#)?;
        writeln!(w, r#"      </Topology>"#)?;

        // Geometry
        writeln!(w, r#"      <Geometry GeometryType="XYZ">"#)?;
        writeln!(
            w,
            r#"        <DataItem Format="XML" DataType="Float" Precision="8" Dimensions="{n_pts} 3">"#
        )?;
        for i in 0..n_pts {
            let p = pd.points.get(i);
            writeln!(w, "          {} {} {}", p[0], p[1], p[2])?;
        }
        writeln!(w, r#"        </DataItem>"#)?;
        writeln!(w, r#"      </Geometry>"#)?;

        write_attributes(w, pd.cell_data(), "Cell")?;
        write_attributes(w, pd.point_data(), "Node")?;

        writeln!(w, r#"    </Grid>"#)?;
        writeln!(w, r#"  </Domain>"#)?;
        writeln!(w, r#"</Xdmf>"#)?;

        Ok(())
    }

    /// Write an ImageData as an XDMF file with inline data.
    pub fn write_image_data(path: &Path, img: &ImageData) -> Result<(), VtkError> {
        let mut f = std::fs::File::create(path)?;
        Self::write_image_data_to(&mut f, img)
    }

    /// Write an ImageData as XDMF to a writer.
    pub fn write_image_data_to<W: Write>(w: &mut W, img: &ImageData) -> Result<(), VtkError> {
        let dims = img.dimensions();
        let spacing = img.spacing();
        let origin = img.origin();

        writeln!(w, r#"<?xml version="1.0" ?>"#)?;
        writeln!(w, r#"<Xdmf Version="3.0">"#)?;
        writeln!(w, r#"  <Domain>"#)?;
        writeln!(w, r#"    <Grid Name="image" GridType="Uniform">"#)?;

        // 3DCoRectMesh topology
        writeln!(
            w,
            r#"      <Topology TopologyType="3DCoRectMesh" Dimensions="{} {} {}"/>"#,
            dims[2], dims[1], dims[0]
        )?;

        // Origin + spacing geometry
        writeln!(w, r#"      <Geometry GeometryType="ORIGIN_DXDYDZ">"#)?;
        writeln!(w, r#"        <DataItem Format="XML" Dimensions="3">"#)?;
        writeln!(w, "          {} {} {}", origin[2], origin[1], origin[0])?;
        writeln!(w, r#"        </DataItem>"#)?;
        writeln!(w, r#"        <DataItem Format="XML" Dimensions="3">"#)?;
        writeln!(w, "          {} {} {}", spacing[2], spacing[1], spacing[0])?;
        writeln!(w, r#"        </DataItem>"#)?;
        writeln!(w, r#"      </Geometry>"#)?;

        write_image_attributes(w, img.cell_data(), "Cell", dims)?;
        write_image_attributes(w, img.point_data(), "Node", dims)?;

        writeln!(w, r#"    </Grid>"#)?;
        writeln!(w, r#"  </Domain>"#)?;
        writeln!(w, r#"</Xdmf>"#)?;

        Ok(())
    }
}

fn validated_polygon_cells(pd: &PolyData) -> Result<Vec<Vec<usize>>, VtkError> {
    let mut cells = Vec::with_capacity(pd.polys.num_cells());
    let mut nodes_per_element = None;

    for cell in pd.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if nodes_per_element.is_none() {
            nodes_per_element = Some(cell.len());
        }

        let mut converted = Vec::with_capacity(cell.len());
        for &id in cell {
            let id = usize::try_from(id)
                .map_err(|_| VtkError::Parse("invalid polygon point index".into()))?;
            if id >= pd.points.len() {
                return Err(VtkError::Parse("invalid polygon point index".into()));
            }
            converted.push(id);
        }
        cells.push(converted);
    }

    Ok(cells)
}

fn mixed_topology_connectivity(cells: &[Vec<usize>]) -> Vec<usize> {
    const XDMF_POLYGON: usize = 0x3;
    const XDMF_TRI: usize = 0x4;
    const XDMF_QUAD: usize = 0x5;

    let mut mixed = Vec::new();
    for cell in cells {
        if cell.len() == 3 {
            mixed.push(XDMF_TRI);
        } else if cell.len() == 4 {
            mixed.push(XDMF_QUAD);
        } else {
            mixed.push(XDMF_POLYGON);
            mixed.push(cell.len());
        }
        mixed.extend(cell.iter().copied());
    }
    mixed
}

fn write_attributes<W: Write>(
    w: &mut W,
    attrs: &DataSetAttributes,
    center: &str,
) -> Result<(), VtkError> {
    let mut arrays = (0..attrs.num_arrays())
        .filter_map(|idx| attrs.get_array_by_index(idx))
        .collect::<Vec<_>>();
    arrays.sort_by(|a, b| a.name().cmp(b.name()));

    for arr in arrays {
        let name = escape_xml_attr(arr.name());
        let nc = arr.num_components();
        let nt = arr.num_tuples();
        let attr_type = attribute_type(nc);

        writeln!(
            w,
            r#"      <Attribute Name="{name}" AttributeType="{attr_type}" Center="{center}">"#
        )?;
        if nc == 1 {
            writeln!(
                w,
                r#"        <DataItem Format="XML" DataType="{}" Precision="{}" Dimensions="{nt}">"#,
                xdmf_data_type(arr.scalar_type()),
                xdmf_precision(arr.scalar_type())
            )?;
        } else {
            writeln!(
                w,
                r#"        <DataItem Format="XML" DataType="{}" Precision="{}" Dimensions="{nt} {nc}">"#,
                xdmf_data_type(arr.scalar_type()),
                xdmf_precision(arr.scalar_type())
            )?;
        }
        write_array_values(w, arr)?;
        writeln!(w, r#"        </DataItem>"#)?;
        writeln!(w, r#"      </Attribute>"#)?;
    }

    Ok(())
}

fn write_image_attributes<W: Write>(
    w: &mut W,
    attrs: &DataSetAttributes,
    center: &str,
    dims: [usize; 3],
) -> Result<(), VtkError> {
    let attr_dims = if center == "Cell" {
        [
            dims[0].saturating_sub(1),
            dims[1].saturating_sub(1),
            dims[2].saturating_sub(1),
        ]
    } else {
        dims
    };
    let mut arrays = (0..attrs.num_arrays())
        .filter_map(|idx| attrs.get_array_by_index(idx))
        .collect::<Vec<_>>();
    arrays.sort_by(|a, b| a.name().cmp(b.name()));

    for arr in arrays {
        let name = escape_xml_attr(arr.name());
        let nc = arr.num_components();
        let attr_type = attribute_type(nc);

        writeln!(
            w,
            r#"      <Attribute Name="{name}" AttributeType="{attr_type}" Center="{center}">"#
        )?;
        if nc == 1 {
            writeln!(
                w,
                r#"        <DataItem Format="XML" DataType="{}" Precision="{}" Dimensions="{} {} {}">"#,
                xdmf_data_type(arr.scalar_type()),
                xdmf_precision(arr.scalar_type()),
                attr_dims[2],
                attr_dims[1],
                attr_dims[0]
            )?;
        } else {
            writeln!(
                w,
                r#"        <DataItem Format="XML" DataType="{}" Precision="{}" Dimensions="{} {} {} {}">"#,
                xdmf_data_type(arr.scalar_type()),
                xdmf_precision(arr.scalar_type()),
                attr_dims[2],
                attr_dims[1],
                attr_dims[0],
                nc
            )?;
        }
        write_array_values(w, arr)?;
        writeln!(w, r#"        </DataItem>"#)?;
        writeln!(w, r#"      </Attribute>"#)?;
    }

    Ok(())
}

fn attribute_type(num_components: usize) -> &'static str {
    match num_components {
        1 => "Scalar",
        3 => "Vector",
        6 => "Tensor6",
        9 => "Tensor",
        _ => "Matrix",
    }
}

fn write_array_values<W: Write>(
    w: &mut W,
    arr: &crate::data::AnyDataArray,
) -> Result<(), VtkError> {
    macro_rules! write_typed {
        ($array:expr) => {{
            for i in 0..$array.num_tuples() {
                let vals: Vec<String> = $array.tuple(i).iter().map(|v| v.to_string()).collect();
                writeln!(w, "          {}", vals.join(" "))?;
            }
        }};
    }

    match arr {
        crate::data::AnyDataArray::F32(a) => write_typed!(a),
        crate::data::AnyDataArray::F64(a) => write_typed!(a),
        crate::data::AnyDataArray::I8(a) => write_typed!(a),
        crate::data::AnyDataArray::I16(a) => write_typed!(a),
        crate::data::AnyDataArray::I32(a) => write_typed!(a),
        crate::data::AnyDataArray::I64(a) => write_typed!(a),
        crate::data::AnyDataArray::U8(a) => write_typed!(a),
        crate::data::AnyDataArray::U16(a) => write_typed!(a),
        crate::data::AnyDataArray::U32(a) => write_typed!(a),
        crate::data::AnyDataArray::U64(a) => write_typed!(a),
    }
    Ok(())
}

fn xdmf_data_type(scalar_type: ScalarType) -> &'static str {
    match scalar_type {
        ScalarType::F32 | ScalarType::F64 => "Float",
        ScalarType::I8 | ScalarType::I16 | ScalarType::I32 | ScalarType::I64 => "Int",
        ScalarType::U8 | ScalarType::U16 | ScalarType::U32 | ScalarType::U64 => "UInt",
    }
}

fn xdmf_precision(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::I8 | ScalarType::U8 => 1,
        ScalarType::I16 | ScalarType::U16 => 2,
        ScalarType::F32 | ScalarType::I32 | ScalarType::U32 => 4,
        ScalarType::F64 | ScalarType::I64 | ScalarType::U64 => 8,
    }
}

fn escape_xml_attr(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn write_poly_data_xdmf() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        XdmfWriter::write_poly_data_to(&mut buf, &pd).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("Xdmf Version"));
        assert!(xml.contains("Triangle"));
        assert!(xml.contains("XYZ"));
    }

    #[test]
    fn write_poly_data_with_scalars() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = DataArray::from_vec("temperature", vec![10.0f64, 20.0, 30.0], 1);
        pd.point_data_mut().add_array(s.into());

        let mut buf = Vec::new();
        XdmfWriter::write_poly_data_to(&mut buf, &pd).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("temperature"));
        assert!(xml.contains("Scalar"));
    }

    #[test]
    fn write_poly_data_preserves_quads_and_cell_data() {
        let mut pd = PolyData::from_quads(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        pd.cell_data_mut()
            .add_array(DataArray::from_vec("cell&value", vec![42.0f64], 1).into());

        let mut buf = Vec::new();
        XdmfWriter::write_poly_data_to(&mut buf, &pd).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"TopologyType="Quadrilateral""#));
        assert!(xml.contains(r#"Name="cell&amp;value""#));
        assert!(xml.contains(r#"Center="Cell""#));
    }

    #[test]
    fn write_poly_data_rejects_invalid_connectivity() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, -1, 2]);

        let mut buf = Vec::new();
        let err = XdmfWriter::write_poly_data_to(&mut buf, &pd).unwrap_err();
        assert!(matches!(err, VtkError::Parse(_)));
    }

    #[test]
    fn write_poly_data_mixed_polygons() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let mut buf = Vec::new();
        XdmfWriter::write_poly_data_to(&mut buf, &pd).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"TopologyType="Mixed""#));
        assert!(xml.contains("4 0 1 2 5 0 1 2 3"));
    }

    #[test]
    fn write_image_data_xdmf() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let mut buf = Vec::new();
        XdmfWriter::write_image_data_to(&mut buf, &img).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("3DCoRectMesh"));
        assert!(xml.contains("ORIGIN_DXDYDZ"));
    }
}
