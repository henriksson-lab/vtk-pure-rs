use crate::data::ImageData;
use std::collections::HashMap;

fn image_point_count(dims: [usize; 3]) -> Option<usize> {
    dims[0].checked_mul(dims[1])?.checked_mul(dims[2])
}

/// Properties of a labeled region in an ImageData.
#[derive(Debug, Clone, Default)]
pub struct RegionProps {
    pub label: i64,
    pub area: usize,
    pub centroid: [f64; 3],
    pub bounding_box: [usize; 6], // imin,imax,jmin,jmax,kmin,kmax
    pub mean_value: f64,
}

/// Compute properties for each labeled region.
pub fn region_properties(
    input: &ImageData,
    labels_name: &str,
    values_name: Option<&str>,
) -> Vec<RegionProps> {
    let la = match input.point_data().get_array(labels_name) {
        Some(a) => a,
        None => return vec![],
    };
    let va = values_name.and_then(|n| input.point_data().get_array(n));

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = match image_point_count(dims) {
        Some(n) => n,
        None => return vec![],
    };
    if la.num_tuples() < n || matches!(va, Some(a) if a.num_tuples() < n) {
        return vec![];
    }
    let sp = input.spacing();
    let origin = input.origin();

    let mut bl = [0.0f64];
    let mut bv = [0.0f64];

    struct Acc {
        area: usize,
        sx: f64,
        sy: f64,
        sz: f64,
        sv: f64,
        imin: usize,
        imax: usize,
        jmin: usize,
        jmax: usize,
        kmin: usize,
        kmax: usize,
    }

    let mut map: HashMap<i64, Acc> = HashMap::new();

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                la.tuple_as_f64(k * ny * nx + j * nx + i, &mut bl);
                let label = bl[0] as i64;
                if label == 0 {
                    continue;
                }

                let x = origin[0] + i as f64 * sp[0];
                let y = origin[1] + j as f64 * sp[1];
                let z = origin[2] + k as f64 * sp[2];
                let v = if let Some(a) = &va {
                    a.tuple_as_f64(k * ny * nx + j * nx + i, &mut bv);
                    bv[0]
                } else {
                    0.0
                };

                let e = map.entry(label).or_insert(Acc {
                    area: 0,
                    sx: 0.0,
                    sy: 0.0,
                    sz: 0.0,
                    sv: 0.0,
                    imin: usize::MAX,
                    imax: 0,
                    jmin: usize::MAX,
                    jmax: 0,
                    kmin: usize::MAX,
                    kmax: 0,
                });
                e.area += 1;
                e.sx += x;
                e.sy += y;
                e.sz += z;
                e.sv += v;
                e.imin = e.imin.min(i);
                e.imax = e.imax.max(i);
                e.jmin = e.jmin.min(j);
                e.jmax = e.jmax.max(j);
                e.kmin = e.kmin.min(k);
                e.kmax = e.kmax.max(k);
            }
        }
    }

    let mut result: Vec<RegionProps> = map
        .into_iter()
        .map(|(label, a)| {
            let n = a.area as f64;
            RegionProps {
                label,
                area: a.area,
                centroid: [a.sx / n, a.sy / n, a.sz / n],
                bounding_box: [a.imin, a.imax, a.jmin, a.jmax, a.kmin, a.kmax],
                mean_value: if n > 0.0 { a.sv / n } else { 0.0 },
            }
        })
        .collect();
    result.sort_by_key(|r| r.label);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn two_regions() {
        let mut img = ImageData::with_dimensions(6, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![1.0, 1.0, 1.0, 2.0, 2.0, 0.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0, 100.0, 200.0, 0.0],
                1,
            )));

        let props = region_properties(&img, "l", Some("v"));
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].area, 3);
        assert!((props[0].mean_value - 20.0).abs() < 1e-10);
        assert_eq!(props[1].area, 2);
    }

    #[test]
    fn no_values() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![1.0, 1.0, 2.0],
                1,
            )));

        let props = region_properties(&img, "l", None);
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn bounding_box() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut labels = vec![0.0; 25];
        labels[6] = 1.0;
        labels[7] = 1.0;
        labels[11] = 1.0;
        labels[12] = 1.0; // 2x2 block at (1,1)
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("l", labels, 1)));

        let props = region_properties(&img, "l", None);
        assert_eq!(props[0].bounding_box[0], 1); // imin
        assert_eq!(props[0].bounding_box[1], 2); // imax
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        assert!(region_properties(&img, "nope", None).is_empty());
    }
}
