//! Earth continent geometry source translated from `vtkEarthSource`.

use std::sync::OnceLock;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

const EARTH_SOURCE_DATA: &str =
    include_str!("../../../../VTK/Filters/Hybrid/vtkEarthSourceData.inl");

/// Parameters for earth continent generation.
pub struct EarthParams {
    /// Radius of earth. Default: 1.0
    pub radius: f64,
    /// Turn on every nth entity. VTK clamps this to 1..=16. Default: 10
    pub on_ratio: usize,
    /// Draw continents as closed outline loops instead of filled polygons.
    /// Default: true
    pub outline: bool,
}

impl Default for EarthParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            on_ratio: 10,
            outline: true,
        }
    }
}

fn vtk_earth_data() -> &'static [i16] {
    static DATA: OnceLock<Vec<i16>> = OnceLock::new();
    DATA.get_or_init(parse_vtk_earth_data)
}

fn parse_vtk_earth_data() -> Vec<i16> {
    let array_body = EARTH_SOURCE_DATA
        .split_once("static const short vtkEarthData[] = {")
        .and_then(|(_, rest)| rest.split_once("};").map(|(body, _)| body))
        .expect("vtkEarthSourceData.inl must contain vtkEarthData initializer");

    let mut values = Vec::new();
    let mut token = String::new();
    let mut chars = array_body.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' {
            match chars.peek().copied() {
                Some('*') => {
                    chars.next();
                    let mut previous = '\0';
                    for comment_char in chars.by_ref() {
                        if previous == '*' && comment_char == '/' {
                            break;
                        }
                        previous = comment_char;
                    }
                    continue;
                }
                Some('/') => {
                    chars.next();
                    for comment_char in chars.by_ref() {
                        if comment_char == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }

        if c == '-' || c.is_ascii_digit() {
            token.push(c);
        } else if !token.is_empty() {
            values.push(
                token
                    .parse::<i16>()
                    .expect("vtkEarthData values must fit in i16"),
            );
            token.clear();
        }
    }

    if !token.is_empty() {
        values.push(
            token
                .parse::<i16>()
                .expect("vtkEarthData values must fit in i16"),
        );
    }

    values
}

fn normalize(x: [f64; 3]) -> [f64; 3] {
    let length = (x[0] * x[0] + x[1] * x[1] + x[2] * x[2]).sqrt();
    if length > 0.0 {
        [x[0] / length, x[1] / length, x[2] / length]
    } else {
        x
    }
}

/// Generate earth continent outlines or filled polygons.
///
/// This follows VTK's `vtkEarthSource::RequestData`: the embedded
/// `vtkEarthData` table is a sequence of delta-encoded closed curves. Land
/// curves with enough points are sampled at `on_ratio`, mapped from VTK's
/// `[x, y, z]` data axes to output `[z, x, y]`, scaled by `radius`, and emitted
/// either as closed line cells (`outline`) or polygon cells.
pub fn earth(params: &EarthParams) -> PolyData {
    let radius = params.radius.clamp(0.0, f64::MAX);
    let on_ratio = params.on_ratio.clamp(1, 16);
    let outline = params.outline;
    let scale = 1.0 / 30000.0;

    let mut points = Points::<f64>::with_capacity(12000 / on_ratio);
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut cells = CellArray::new();

    let mut offset = 0usize;
    let mut actual_pts = 0usize;
    let mut actual_polys = 0usize;
    let data = vtk_earth_data();

    loop {
        if offset >= data.len() {
            break;
        }

        let npts = data[offset] as i32;
        offset += 1;
        if npts == 0 || actual_polys > 16 {
            break;
        }

        let land = data[offset] as i32;
        offset += 1;

        let mut base = [0.0f64; 3];
        for i in 1..=npts {
            base[0] += data[offset] as f64 * scale;
            base[1] += data[offset + 1] as f64 * scale;
            base[2] += data[offset + 2] as f64 * scale;
            offset += 3;

            let x = [base[2] * radius, base[0] * radius, base[1] * radius];

            if land == 1 && npts > (on_ratio as i32) * 3 && i % on_ratio as i32 == 0 {
                points.push(x);
                normals.push_tuple(&normalize(x));
                actual_pts += 1;
            }
        }

        if land == 1 && npts > (on_ratio as i32) * 3 {
            let sampled_pts = npts as usize / on_ratio;
            let first_pt = actual_pts - sampled_pts;
            let mut point_ids: Vec<i64> = (0..sampled_pts).map(|i| (first_pt + i) as i64).collect();

            if outline {
                point_ids.push(first_pt as i64);
            }

            cells.push_cell(&point_ids);
            actual_polys += 1;
        }
    }

    let mut output = PolyData::new();
    output.points = points;
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(normals));
    output.point_data_mut().set_active_normals("Normals");
    if outline {
        output.lines = cells;
    } else {
        output.polys = cells;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_earth_matches_vtk_outline_defaults() {
        let globe = earth(&EarthParams::default());
        assert!(!globe.points.is_empty());
        assert_eq!(globe.lines.num_cells(), 17);
        assert_eq!(globe.polys.num_cells(), 0);
        assert!(globe.point_data().normals().is_some());
    }

    #[test]
    fn filled_earth_uses_polygons() {
        let globe = earth(&EarthParams {
            outline: false,
            ..Default::default()
        });
        assert!(!globe.points.is_empty());
        assert_eq!(globe.lines.num_cells(), 0);
        assert_eq!(globe.polys.num_cells(), 17);
    }

    #[test]
    fn radius_scales_points() {
        let globe = earth(&EarthParams {
            radius: 2.0,
            on_ratio: 4,
            ..Default::default()
        });
        let p = globe.points.get(0);
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 2.0).abs() < 1e-3);
    }

    #[test]
    fn on_ratio_is_clamped_to_vtk_range() {
        let detailed = earth(&EarthParams {
            on_ratio: 0,
            ..Default::default()
        });
        let coarse = earth(&EarthParams {
            on_ratio: 99,
            ..Default::default()
        });
        assert!(detailed.points.len() > coarse.points.len());
    }
}
