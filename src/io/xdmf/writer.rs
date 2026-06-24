use std::io::Write;
use std::path::Path;

use crate::data::{ImageData, PolyData};
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

        // Triangulate polys
        let mut tris: Vec<[usize; 3]> = Vec::new();
        for cell in pd.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            for i in 1..cell.len() - 1 {
                tris.push([cell[0] as usize, cell[i] as usize, cell[i + 1] as usize]);
            }
        }
        let n_tris = tris.len();

        writeln!(w, r#"<?xml version="1.0" ?>"#)?;
        writeln!(w, r#"<Xdmf Version="3.0">"#)?;
        writeln!(w, r#"  <Domain>"#)?;
        writeln!(w, r#"    <Grid Name="mesh" GridType="Uniform">"#)?;

        // Topology
        writeln!(
            w,
            r#"      <Topology TopologyType="Triangle" NumberOfElements="{n_tris}">"#
        )?;
        writeln!(
            w,
            r#"        <DataItem Format="XML" DataType="Int" Dimensions="{n_tris} 3">"#
        )?;
        for tri in &tris {
            writeln!(w, "          {} {} {}", tri[0], tri[1], tri[2])?;
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

        // Point data attributes
        for idx in 0..pd.point_data().num_arrays() {
            let arr = pd.point_data().get_array_by_index(idx).unwrap();
            let name = arr.name();
            let nc = arr.num_components();
            let nt = arr.num_tuples();
            let attr_type = if nc == 1 {
                "Scalar"
            } else if nc == 3 {
                "Vector"
            } else {
                "Matrix"
            };

            writeln!(
                w,
                r#"      <Attribute Name="{name}" AttributeType="{attr_type}" Center="Node">"#
            )?;
            if nc == 1 {
                writeln!(
                    w,
                    r#"        <DataItem Format="XML" DataType="Float" Precision="8" Dimensions="{nt}">"#
                )?;
            } else {
                writeln!(
                    w,
                    r#"        <DataItem Format="XML" DataType="Float" Precision="8" Dimensions="{nt} {nc}">"#
                )?;
            }
            let mut buf = vec![0.0f64; nc];
            for i in 0..nt {
                arr.tuple_as_f64(i, &mut buf);
                let vals: Vec<String> = buf.iter().map(|v| format!("{v}")).collect();
                writeln!(w, "          {}", vals.join(" "))?;
            }
            writeln!(w, r#"        </DataItem>"#)?;
            writeln!(w, r#"      </Attribute>"#)?;
        }

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

        // Point data
        for idx in 0..img.point_data().num_arrays() {
            let arr = img.point_data().get_array_by_index(idx).unwrap();
            let name = arr.name();
            let nc = arr.num_components();
            let nt = arr.num_tuples();
            let attr_type = if nc == 1 { "Scalar" } else { "Vector" };

            writeln!(
                w,
                r#"      <Attribute Name="{name}" AttributeType="{attr_type}" Center="Node">"#
            )?;
            if nc == 1 {
                writeln!(
                    w,
                    r#"        <DataItem Format="XML" DataType="Float" Precision="8" Dimensions="{} {} {}">"#,
                    dims[2], dims[1], dims[0]
                )?;
            } else {
                writeln!(
                    w,
                    r#"        <DataItem Format="XML" DataType="Float" Precision="8" Dimensions="{} {} {} {}">"#,
                    dims[2], dims[1], dims[0], nc
                )?;
            }
            let mut buf = vec![0.0f64; nc];
            for i in 0..nt {
                arr.tuple_as_f64(i, &mut buf);
                let vals: Vec<String> = buf.iter().map(|v| format!("{v}")).collect();
                writeln!(w, "          {}", vals.join(" "))?;
            }
            writeln!(w, r#"        </DataItem>"#)?;
            writeln!(w, r#"      </Attribute>"#)?;
        }

        writeln!(w, r#"    </Grid>"#)?;
        writeln!(w, r#"  </Domain>"#)?;
        writeln!(w, r#"</Xdmf>"#)?;

        Ok(())
    }
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
    fn write_image_data_xdmf() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let mut buf = Vec::new();
        XdmfWriter::write_image_data_to(&mut buf, &img).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("3DCoRectMesh"));
        assert!(xml.contains("ORIGIN_DXDYDZ"));
    }
}
