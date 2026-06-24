//! Minimal CityGML reader that extracts building footprint polygons.
//!
//! Parses `<gml:posList>` and `<gml:LinearRing>` elements using simple
//! string matching (no XML library dependency).

use std::fs;
use std::path::Path;

use crate::data::{CellArray, Points, PolyData};
use crate::types::VtkError;

/// Read building footprints from a CityGML file as polygon `PolyData`.
///
/// Extracts `<gml:posList>` coordinate lists found inside
/// `<gml:LinearRing>` elements (typically inside `<bldg:Building>` blocks).
///
/// Coordinates are expected as space-separated `x y z` triples.
pub fn read_citygml(path: &Path) -> Result<PolyData, VtkError> {
    let content = fs::read_to_string(path)?;
    parse_citygml_string(&content)
}

/// Parse a CityGML XML string and return polygon `PolyData`.
pub fn parse_citygml_string(xml: &str) -> Result<PolyData, VtkError> {
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Find all <gml:posList>...</gml:posList> blocks
    let pos_list_start = "gml:posList";
    let pos_list_end_tag = "</gml:posList>";

    let mut search_from = 0;
    while let Some(tag_start) = xml[search_from..].find(&format!("<{pos_list_start}")) {
        let abs_start = search_from + tag_start;
        // Find the end of the opening tag (handle attributes)
        let content_start = match xml[abs_start..].find('>') {
            Some(i) => abs_start + i + 1,
            None => break,
        };
        let content_end = match xml[content_start..].find(pos_list_end_tag) {
            Some(i) => content_start + i,
            None => break,
        };

        let text = xml[content_start..content_end].trim();
        let coords = parse_pos_list(text);

        if coords.len() >= 3 {
            let _base = points.len() as i64;
            let mut cell_ids = Vec::new();
            for pt in &coords {
                cell_ids.push(points.len() as i64);
                points.push(*pt);
            }
            // Remove duplicate closing vertex if present (GML convention)
            if coords.len() > 1 && coords.first() == coords.last() {
                cell_ids.pop();
                // The duplicate point was already pushed; that's fine, just
                // don't include it in the cell.
            }
            if cell_ids.len() >= 3 {
                polys.push_cell(&cell_ids);
            }
        }

        search_from = content_end + pos_list_end_tag.len();
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    Ok(pd)
}

/// Parse a space-separated coordinate list into `[x, y, z]` triples.
fn parse_pos_list(text: &str) -> Vec<[f64; 3]> {
    let nums: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_inline_citygml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CityModel xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
           xmlns:gml="http://www.opengis.net/gml">
  <cityObjectMember>
    <bldg:Building gml:id="b1">
      <bldg:lod1Solid>
        <gml:Solid>
          <gml:exterior>
            <gml:CompositeSurface>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>
                        0.0 0.0 0.0  10.0 0.0 0.0  10.0 10.0 0.0  0.0 10.0 0.0  0.0 0.0 0.0
                      </gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>
                        0.0 0.0 5.0  10.0 0.0 5.0  10.0 10.0 5.0  0.0 10.0 5.0  0.0 0.0 5.0
                      </gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:CompositeSurface>
          </gml:exterior>
        </gml:Solid>
      </bldg:lod1Solid>
    </bldg:Building>
  </cityObjectMember>
</CityModel>"#;

        let pd = parse_citygml_string(xml).unwrap();
        // Two polygons (floor and roof), each with 4 unique vertices
        // (5th closing vertex is stripped from cell but point still exists)
        assert_eq!(pd.polys.num_cells(), 2);
        assert!(pd.points.len() >= 8);
        // Each polygon cell should have 4 vertices
        assert_eq!(pd.polys.cell(0).len(), 4);
        assert_eq!(pd.polys.cell(1).len(), 4);
    }
}
