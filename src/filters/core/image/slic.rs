use crate::data::{AnyDataArray, DataArray, ImageData};

/// Simple Linear Iterative Clustering (SLIC) superpixel segmentation.
///
/// Clusters voxels based on spatial proximity and scalar similarity.
/// Returns an ImageData with "SuperpixelId" labels. 2D only (nz=1).
pub fn image_slic(
    input: &ImageData,
    scalars: &str,
    n_superpixels: usize,
    compactness: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;
    let sp = input.spacing();

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    // Initialize cluster centers on a grid
    let k = n_superpixels.max(1);
    let step = ((n as f64 / k as f64).sqrt()).ceil() as usize;
    let step = step.max(1);

    let mut centers: Vec<(f64, f64, f64)> = Vec::new(); // (x, y, value)
    for j in (step / 2..ny).step_by(step) {
        for i in (step / 2..nx).step_by(step) {
            centers.push((i as f64, j as f64, values[j * nx + i]));
        }
    }
    if centers.is_empty() {
        centers.push((nx as f64 / 2.0, ny as f64 / 2.0, values[0]));
    }

    let n_centers = centers.len();
    let mut labels = vec![0usize; n];
    let mut dists = vec![f64::MAX; n];

    let m2 = compactness * compactness;
    let s2 = (step * step) as f64;

    // SLIC iterations
    for _ in 0..10 {
        dists.fill(f64::MAX);

        for (ci, &(cx, cy, cv)) in centers.iter().enumerate() {
            let i_min = ((cx - step as f64 * 2.0).max(0.0)) as usize;
            let i_max = ((cx + step as f64 * 2.0).min(nx as f64 - 1.0)) as usize;
            let j_min = ((cy - step as f64 * 2.0).max(0.0)) as usize;
            let j_max = ((cy + step as f64 * 2.0).min(ny as f64 - 1.0)) as usize;

            for j in j_min..=j_max {
                for i in i_min..=i_max {
                    let idx = j * nx + i;
                    let ds = ((i as f64 - cx).powi(2) + (j as f64 - cy).powi(2)) / s2;
                    let dc = (values[idx] - cv).powi(2);
                    let d = dc + m2 * ds;
                    if d < dists[idx] {
                        dists[idx] = d;
                        labels[idx] = ci;
                    }
                }
            }
        }

        // Update centers
        let mut sums = vec![(0.0f64, 0.0f64, 0.0f64, 0usize); n_centers];
        for j in 0..ny {
            for i in 0..nx {
                let ci = labels[j * nx + i];
                sums[ci].0 += i as f64;
                sums[ci].1 += j as f64;
                sums[ci].2 += values[j * nx + i];
                sums[ci].3 += 1;
            }
        }

        for (ci, s) in sums.iter().enumerate() {
            if s.3 > 0 {
                let nf = s.3 as f64;
                centers[ci] = (s.0 / nf, s.1 / nf, s.2 / nf);
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SuperpixelId",
            labels_f,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slic_basic() {
        let mut img = ImageData::with_dimensions(10, 10, 1);
        let values: Vec<f64> = (0..100).map(|i| (i / 50) as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_slic(&img, "v", 4, 10.0);
        assert!(result.point_data().get_array("SuperpixelId").is_some());
    }

    #[test]
    fn uniform_field() {
        let mut img = ImageData::with_dimensions(8, 8, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 64],
                1,
            )));

        let result = image_slic(&img, "v", 4, 10.0);
        let arr = result.point_data().get_array("SuperpixelId").unwrap();
        assert_eq!(arr.num_tuples(), 64);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let r = image_slic(&img, "nope", 4, 10.0);
        assert!(r.point_data().get_array("SuperpixelId").is_none());
    }
}
