//! UV-mapped sphere with texture coordinates.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Create a sphere with proper UV texture coordinates.
pub fn sphere_uv(radius: f64, u_res: usize, v_res: usize) -> PolyData {
    let ures = u_res.max(3);
    let vres = v_res.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut uvs = Vec::new();

    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=ures {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ures as f64;
            pts.push([radius * sv * u.cos(), radius * sv * u.sin(), radius * cv]);
            uvs.push(iu as f64 / ures as f64);
            uvs.push(1.0 - iv as f64 / vres as f64);
        }
    }

    let w = ures + 1;
    for iv in 0..vres {
        for iu in 0..ures {
            let i00 = (iv * w + iu) as i64;
            let i10 = (iv * w + iu + 1) as i64;
            let i01 = ((iv + 1) * w + iu) as i64;
            let i11 = ((iv + 1) * w + iu + 1) as i64;
            if iv == 0 {
                polys.push_cell(&[i00, i11, i01]);
            } else if iv == vres - 1 {
                polys.push_cell(&[i00, i10, i01]);
            } else {
                polys.push_cell(&[i00, i10, i11, i01]);
            }
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uvs, 2)));
    result.point_data_mut().set_active_tcoords("UV");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sphere_uv() {
        let s = sphere_uv(1.0, 16, 8);
        assert!(s.points.len() > 50);
        assert!(s.polys.num_cells() > 50);
        assert!(s.point_data().get_array("UV").is_some());
        assert_eq!(s.point_data().get_array("UV").unwrap().num_components(), 2);
    }
}
