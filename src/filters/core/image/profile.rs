use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a 1D profile along a row (fixed j, k) from ImageData.
pub fn image_profile_row(input: &ImageData, scalars: &str, row: usize, slice: usize) -> Vec<f64> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return vec![],
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    if nx == 0 || ny == 0 || nz == 0 {
        return vec![];
    }
    let j = row.min(ny - 1);
    let k = slice.min(nz - 1);
    let mut buf = [0.0f64];
    (0..nx)
        .map(|i| {
            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
            buf[0]
        })
        .collect()
}

/// Extract a 1D profile along a column (fixed i, k) from ImageData.
pub fn image_profile_column(
    input: &ImageData,
    scalars: &str,
    col: usize,
    slice: usize,
) -> Vec<f64> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return vec![],
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    if nx == 0 || ny == 0 || nz == 0 {
        return vec![];
    }
    let i = col.min(nx - 1);
    let k = slice.min(nz - 1);
    let mut buf = [0.0f64];
    (0..ny)
        .map(|j| {
            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
            buf[0]
        })
        .collect()
}

/// Extract a diagonal profile from corner to corner.
pub fn image_profile_diagonal(input: &ImageData, scalars: &str) -> Vec<f64> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return vec![],
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    if nx == 0 || ny == 0 || dims[2] == 0 {
        return vec![];
    }
    let n = nx.min(ny);
    let mut buf = [0.0f64];
    (0..n)
        .map(|i| {
            let j = i * (ny - 1) / (n - 1).max(1);
            let ii = i * (nx - 1) / (n - 1).max(1);
            arr.tuple_as_f64(j * nx + ii, &mut buf);
            buf[0]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_profile() {
        let mut img = ImageData::with_dimensions(4, 3, 1);
        let values: Vec<f64> = (0..12).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let profile = image_profile_row(&img, "v", 1, 0);
        assert_eq!(profile.len(), 4);
        assert_eq!(profile[0], 4.0); // row 1, col 0
    }

    #[test]
    fn column_profile() {
        let mut img = ImageData::with_dimensions(3, 4, 1);
        let values: Vec<f64> = (0..12).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let profile = image_profile_column(&img, "v", 1, 0);
        assert_eq!(profile.len(), 4);
        assert_eq!(profile[0], 1.0); // row 0, col 1
    }

    #[test]
    fn diagonal_profile() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let values: Vec<f64> = (0..9).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let profile = image_profile_diagonal(&img, "v");
        assert_eq!(profile.len(), 3);
        assert_eq!(profile[0], 0.0); // (0,0)
        assert_eq!(profile[2], 8.0); // (2,2)
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        assert!(image_profile_row(&img, "nope", 0, 0).is_empty());
    }
}
