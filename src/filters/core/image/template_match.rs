use crate::data::{AnyDataArray, DataArray, ImageData};

/// Template matching on 2D ImageData using normalized cross-correlation.
///
/// Slides a template over the image and computes NCC at each position.
/// Adds "MatchScore" array (same size as input, NCC values).
pub fn image_template_match(
    input: &ImageData,
    scalars: &str,
    template: &ImageData,
    template_scalars: &str,
) -> ImageData {
    let ia = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let ta = match template.point_data().get_array(template_scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let td = template.dimensions();
    let tw = td[0] as usize;
    let th = td[1] as usize;

    let mut ibuf = [0.0f64];
    let mut tbuf = [0.0f64];
    let img_v: Vec<f64> = (0..nx * ny)
        .map(|i| {
            ia.tuple_as_f64(i, &mut ibuf);
            ibuf[0]
        })
        .collect();
    let tpl_v: Vec<f64> = (0..tw * th)
        .map(|i| {
            ta.tuple_as_f64(i, &mut tbuf);
            tbuf[0]
        })
        .collect();

    // Template statistics
    let t_mean: f64 = tpl_v.iter().sum::<f64>() / (tw * th) as f64;
    let t_std: f64 =
        (tpl_v.iter().map(|v| (v - t_mean).powi(2)).sum::<f64>() / (tw * th) as f64).sqrt();

    let hw = tw / 2;
    let hh = th / 2;
    let mut scores = vec![0.0f64; nx * ny];

    if t_std < 1e-15 {
        // uniform template
        let mut img = input.clone();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "MatchScore",
                scores,
                1,
            )));
        return img;
    }

    for j in hh..ny.saturating_sub(hh) {
        for i in hw..nx.saturating_sub(hw) {
            let mut sum_it = 0.0;
            let mut sum_i = 0.0;
            let mut sum_i2 = 0.0;
            let mut count = 0;

            for tj in 0..th {
                for ti in 0..tw {
                    let ii = i + ti - hw;
                    let jj = j + tj - hh;
                    if ii < nx && jj < ny {
                        let iv = img_v[jj * nx + ii];
                        let tv = tpl_v[tj * tw + ti] - t_mean;
                        sum_it += iv * tv;
                        sum_i += iv;
                        sum_i2 += iv * iv;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                let i_mean = sum_i / count as f64;
                let i_std = (sum_i2 / count as f64 - i_mean * i_mean).max(0.0).sqrt();
                if i_std > 1e-15 && t_std > 1e-15 {
                    scores[j * nx + i] = sum_it / (count as f64 * i_std * t_std);
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MatchScore",
            scores,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_match_high_score() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                values.clone(),
                1,
            )));

        let mut tpl = ImageData::with_dimensions(3, 3, 1);
        // Center 3x3 patch
        let tpl_vals = vec![
            values[6], values[7], values[8], values[11], values[12], values[13], values[16],
            values[17], values[18],
        ];
        tpl.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", tpl_vals, 1)));

        let result = image_template_match(&img, "v", &tpl, "v");
        let arr = result.point_data().get_array("MatchScore").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf); // center should have high NCC
        assert!(buf[0] > 0.5);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let tpl = ImageData::with_dimensions(3, 3, 1);
        let r = image_template_match(&img, "nope", &tpl, "nope");
        assert!(r.point_data().get_array("MatchScore").is_none());
    }
}
