//! Minimal CityGML reader that extracts polygon rings.
//!
//! Parses `<gml:posList>` / `<gml:pos>` inside `<gml:LinearRing>` elements using simple
//! string matching (no XML library dependency).

use std::fs;
use std::path::Path;

use crate::data::{CellArray, Points, PolyData};
use crate::types::VtkError;

/// Read CityGML linear rings from a file as polygon `PolyData`.
///
/// Extracts coordinate lists found inside `<gml:LinearRing>` elements.
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

    let mut search_from = 0;
    while let Some(tag_start) = xml[search_from..].find("<gml:LinearRing") {
        let abs_start = search_from + tag_start;
        let ring_start = match xml[abs_start..].find('>') {
            Some(i) => abs_start + i + 1,
            None => break,
        };
        let ring_end = match xml[ring_start..].find("</gml:LinearRing>") {
            Some(i) => ring_start + i,
            None => break,
        };

        let ring_xml = &xml[ring_start..ring_end];
        insert_ring(ring_xml, &mut points, &mut polys);

        search_from = ring_end + "</gml:LinearRing>".len();
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    Ok(pd)
}

fn insert_ring(ring_xml: &str, points: &mut Points<f64>, polys: &mut CellArray) {
    let coords = if let Some(pos_list) = first_tag_text(ring_xml, "gml:posList") {
        parse_pos_list(pos_list)
    } else {
        parse_pos_elements(ring_xml)
    };

    if coords.len() < 3 {
        return;
    }

    let mut unique_coords = coords.as_slice();
    if coords.len() > 1 && coords.first() == coords.last() {
        unique_coords = &coords[..coords.len() - 1];
    }
    if unique_coords.len() < 3 {
        return;
    }

    let mut cell_ids = Vec::with_capacity(unique_coords.len());
    for pt in unique_coords {
        cell_ids.push(points.len() as i64);
        points.push(*pt);
    }
    polys.push_cell(&cell_ids);
}

fn first_tag_text<'a>(xml: &'a str, tag_name: &str) -> Option<&'a str> {
    let open = format!("<{tag_name}");
    let close = format!("</{tag_name}>");
    let tag_start = xml.find(&open)?;
    let content_start = tag_start + xml[tag_start..].find('>')? + 1;
    let content_end = content_start + xml[content_start..].find(&close)?;
    Some(xml[content_start..content_end].trim())
}

fn parse_pos_elements(xml: &str) -> Vec<[f64; 3]> {
    let mut coords = Vec::new();
    let mut search_from = 0;
    while let Some(text) = first_tag_text(&xml[search_from..], "gml:pos") {
        if let Some(pt) = parse_pos(text) {
            coords.push(pt);
        }
        let rel_end = match xml[search_from..].find("</gml:pos>") {
            Some(i) => i + "</gml:pos>".len(),
            None => break,
        };
        search_from += rel_end;
    }
    coords
}

fn parse_pos(text: &str) -> Option<[f64; 3]> {
    let nums: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    match nums.as_slice() {
        [x, y] => Some([*x, *y, 0.0]),
        [x, y, z, ..] => Some([*x, *y, *z]),
        _ => None,
    }
}

/// Parse a space-separated coordinate list into `[x, y, z]` triples.
fn parse_pos_list(text: &str) -> Vec<[f64; 3]> {
    let nums: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    if nums.len().is_multiple_of(3) {
        nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
    } else if nums.len().is_multiple_of(2) {
        nums.chunks_exact(2).map(|c| [c[0], c[1], 0.0]).collect()
    } else {
        Vec::new()
    }
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
        assert_eq!(pd.polys.num_cells(), 2);
        assert_eq!(pd.points.len(), 8);
        assert_eq!(pd.polys.cell(0).len(), 4);
        assert_eq!(pd.polys.cell(1).len(), 4);
    }

    #[test]
    fn parse_pos_elements() {
        let xml = r#"<CityModel xmlns:gml="http://www.opengis.net/gml">
  <gml:LinearRing>
    <gml:pos>0 0 0</gml:pos><gml:pos>1 0 0</gml:pos>
    <gml:pos>1 1 0</gml:pos><gml:pos>0 0 0</gml:pos>
  </gml:LinearRing>
</CityModel>"#;
        let pd = parse_citygml_string(xml).unwrap();
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.polys.num_cells(), 1);
    }
}
