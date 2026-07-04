use std::io::Write;
use std::path::Path;

use crate::data::PolyData;
use crate::types::VtkError;

/// File type for STL format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlFormat {
    Ascii,
    Binary,
}

/// Writer for STL (stereolithography) format.
///
/// STL only supports triangle meshes. Non-triangle polygons are triangulated on
/// export. Only polygon and triangle strip cells are exported.
pub struct StlWriter {
    pub format: StlFormat,
    pub solid_name: String,
}

impl Default for StlWriter {
    fn default() -> Self {
        Self {
            format: StlFormat::Ascii,
            solid_name: "Visualization Toolkit generated SLA File".to_string(),
        }
    }
}

impl StlWriter {
    pub fn ascii() -> Self {
        Self {
            format: StlFormat::Ascii,
            ..Default::default()
        }
    }

    pub fn binary() -> Self {
        Self {
            format: StlFormat::Binary,
            ..Default::default()
        }
    }

    pub fn write(&self, path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        self.write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(&self, w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        match self.format {
            StlFormat::Ascii => self.write_ascii(w, data),
            StlFormat::Binary => self.write_binary(w, data),
        }
    }

    fn write_ascii<W: Write>(&self, w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        writeln!(w, "solid {}", self.solid_name)?;

        for tri in stl_triangles(data) {
            let n = triangle_normal(tri[0], tri[1], tri[2]);
            writeln!(w, " facet normal {} {} {}", n[0], n[1], n[2])?;
            writeln!(w, "  outer loop")?;
            writeln!(w, "   vertex {} {} {}", tri[0][0], tri[0][1], tri[0][2])?;
            writeln!(w, "   vertex {} {} {}", tri[1][0], tri[1][1], tri[1][2])?;
            writeln!(w, "   vertex {} {} {}", tri[2][0], tri[2][1], tri[2][2])?;
            writeln!(w, "  endloop")?;
            writeln!(w, " endfacet")?;
        }

        writeln!(w, "endsolid")?;
        Ok(())
    }

    fn write_binary<W: Write>(&self, w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        // 80-byte header
        let mut header = [0u8; 80];
        let name = if self.solid_name.as_bytes().starts_with(b"solid") {
            "Visualization Toolkit generated SLA File"
        } else {
            &self.solid_name
        };
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(80);
        header[..len].copy_from_slice(&name_bytes[..len]);
        w.write_all(&header)?;

        let triangles = stl_triangles(data);
        let num_triangles = triangles.len() as u32;
        w.write_all(&num_triangles.to_le_bytes())?;

        // Write triangles
        for tri in triangles {
            let n = triangle_normal(tri[0], tri[1], tri[2]);

            // Normal (3 x f32 LE)
            for &v in &n {
                w.write_all(&(v as f32).to_le_bytes())?;
            }
            // Vertices (3 x 3 x f32 LE)
            for p in &tri {
                for &v in p {
                    w.write_all(&(v as f32).to_le_bytes())?;
                }
            }
            // Attribute byte count
            w.write_all(&0u16.to_le_bytes())?;
        }

        Ok(())
    }
}

fn stl_triangles(data: &PolyData) -> Vec<[[f64; 3]; 3]> {
    let mut triangles = Vec::new();

    for strip in data.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let ids = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            triangles.push([
                data.points.get(ids[0] as usize),
                data.points.get(ids[1] as usize),
                data.points.get(ids[2] as usize),
            ]);
        }
    }

    for cell in data.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if cell.len() == 3 {
            triangles.push([
                data.points.get(cell[0] as usize),
                data.points.get(cell[1] as usize),
                data.points.get(cell[2] as usize),
            ]);
        } else {
            for [i0, i1, i2] in triangulate_polygon_cell(data, cell) {
                triangles.push([
                    data.points.get(i0 as usize),
                    data.points.get(i1 as usize),
                    data.points.get(i2 as usize),
                ]);
            }
        }
    }

    triangles
}

fn triangulate_polygon_cell(data: &PolyData, cell: &[i64]) -> Vec<[i64; 3]> {
    let points: Vec<[f64; 3]> = cell
        .iter()
        .map(|&id| data.points.get(id as usize))
        .collect();
    let axis = dominant_normal_axis(&points);
    let projected: Vec<[f64; 2]> = points.iter().map(|&p| project_point(p, axis)).collect();
    let ccw = signed_area_2d(&projected) >= 0.0;
    let mut remaining: Vec<usize> = (0..cell.len()).collect();
    let mut triangles = Vec::new();

    while remaining.len() > 3 {
        let mut clipped = false;
        let n = remaining.len();
        for i in 0..n {
            let prev = remaining[(i + n - 1) % n];
            let curr = remaining[i];
            let next = remaining[(i + 1) % n];

            if !is_convex(projected[prev], projected[curr], projected[next], ccw) {
                continue;
            }
            if remaining.iter().any(|&idx| {
                idx != prev
                    && idx != curr
                    && idx != next
                    && point_in_triangle(
                        projected[idx],
                        projected[prev],
                        projected[curr],
                        projected[next],
                    )
            }) {
                continue;
            }

            triangles.push([cell[prev], cell[curr], cell[next]]);
            remaining.remove(i);
            clipped = true;
            break;
        }

        if !clipped {
            return fan_triangulate(cell);
        }
    }

    triangles.push([cell[remaining[0]], cell[remaining[1]], cell[remaining[2]]]);
    triangles
}

fn fan_triangulate(cell: &[i64]) -> Vec<[i64; 3]> {
    let mut triangles = Vec::new();
    for i in 1..cell.len() - 1 {
        triangles.push([cell[0], cell[i], cell[i + 1]]);
    }
    triangles
}

fn dominant_normal_axis(points: &[[f64; 3]]) -> usize {
    let mut normal = [0.0; 3];
    for i in 0..points.len() {
        let p = points[i];
        let q = points[(i + 1) % points.len()];
        normal[0] += (p[1] - q[1]) * (p[2] + q[2]);
        normal[1] += (p[2] - q[2]) * (p[0] + q[0]);
        normal[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    if normal[0].abs() >= normal[1].abs() && normal[0].abs() >= normal[2].abs() {
        0
    } else if normal[1].abs() >= normal[2].abs() {
        1
    } else {
        2
    }
}

fn project_point(point: [f64; 3], axis: usize) -> [f64; 2] {
    match axis {
        0 => [point[1], point[2]],
        1 => [point[0], point[2]],
        _ => [point[0], point[1]],
    }
}

fn signed_area_2d(points: &[[f64; 2]]) -> f64 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let p = points[i];
        let q = points[(i + 1) % points.len()];
        area += p[0] * q[1] - q[0] * p[1];
    }
    area * 0.5
}

fn is_convex(prev: [f64; 2], curr: [f64; 2], next: [f64; 2], ccw: bool) -> bool {
    let cross = orient2d(prev, curr, next);
    if ccw {
        cross > 1e-12
    } else {
        cross < -1e-12
    }
}

fn point_in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let ab = orient2d(a, b, p);
    let bc = orient2d(b, c, p);
    let ca = orient2d(c, a, p);
    let has_neg = ab < -1e-12 || bc < -1e-12 || ca < -1e-12;
    let has_pos = ab > 1e-12 || bc > 1e-12 || ca > 1e-12;
    !(has_neg && has_pos)
}

fn orient2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn triangle_normal(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> [f64; 3] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-10 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_ascii_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let writer = StlWriter::ascii();
        let mut buf = Vec::new();
        writer.write_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("solid Visualization Toolkit generated SLA File"));
        assert!(output.contains("facet normal"));
        assert!(output.contains("vertex"));
        assert!(output.contains("endsolid"));
    }

    #[test]
    fn write_binary_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let writer = StlWriter::binary();
        let mut buf = Vec::new();
        writer.write_to(&mut buf, &pd).unwrap();
        // 80 header + 4 count + 50 per triangle = 134
        assert_eq!(buf.len(), 134);
    }

    #[test]
    fn binary_header_does_not_start_with_solid() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let writer = StlWriter {
            solid_name: "solid bad header".to_string(),
            ..StlWriter::binary()
        };
        let mut buf = Vec::new();
        writer.write_to(&mut buf, &pd).unwrap();
        assert_ne!(&buf[..5], b"solid");
    }

    #[test]
    fn writes_triangle_strips() {
        let mut pd = PolyData::new();
        pd.points = crate::data::Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        pd.strips.push_cell(&[0, 1, 2, 3]);
        let mut buf = Vec::new();
        StlWriter::binary().write_to(&mut buf, &pd).unwrap();
        assert_eq!(u32::from_le_bytes(buf[80..84].try_into().unwrap()), 2);
    }

    #[test]
    fn triangulates_concave_polygon_without_fan_overlap() {
        let mut pd = PolyData::new();
        pd.points = crate::data::Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ]);
        pd.polys.push_cell(&[0, 1, 2, 3, 4, 5]);

        let triangles = stl_triangles(&pd);
        assert_eq!(triangles.len(), 4);
        for tri in triangles {
            let centroid = [
                (tri[0][0] + tri[1][0] + tri[2][0]) / 3.0,
                (tri[0][1] + tri[1][1] + tri[2][1]) / 3.0,
            ];
            assert!(point_inside_concave_l(centroid));
        }
    }

    fn point_inside_concave_l(point: [f64; 2]) -> bool {
        (0.0..=2.0).contains(&point[0])
            && (0.0..=2.0).contains(&point[1])
            && !(point[0] > 1.0 && point[1] > 1.0)
    }
}
