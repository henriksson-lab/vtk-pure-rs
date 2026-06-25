//! Haar wavelet decomposition and reconstruction for images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Perform one level of 2D Haar wavelet decomposition.
/// Returns (LL, LH, HL, HH) sub-bands as separate images.
pub fn haar_decompose(
    input: &ImageData,
    scalars: &str,
) -> (ImageData, ImageData, ImageData, ImageData) {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => {
            let c = input.clone();
            return (c.clone(), c.clone(), c.clone(), c);
        }
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let hnx = nx / 2;
    let hny = ny / 2;
    if hnx == 0 || hny == 0 {
        let c = input.clone();
        return (c.clone(), c.clone(), c.clone(), c);
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut ll = vec![0.0; hnx * hny];
    let mut lh = vec![0.0; hnx * hny];
    let mut hl = vec![0.0; hnx * hny];
    let mut hh = vec![0.0; hnx * hny];

    for iy in 0..hny {
        for ix in 0..hnx {
            let a = vals[2 * ix + 2 * iy * nx];
            let b = vals[2 * ix + 1 + 2 * iy * nx];
            let c = vals[2 * ix + (2 * iy + 1) * nx];
            let d = vals[2 * ix + 1 + (2 * iy + 1) * nx];
            let idx = ix + iy * hnx;
            ll[idx] = (a + b + c + d) * 0.25;
            lh[idx] = (a - b + c - d) * 0.25;
            hl[idx] = (a + b - c - d) * 0.25;
            hh[idx] = (a - b - c + d) * 0.25;
        }
    }

    let sp = input.spacing();
    let make = |name: &str, data: Vec<f64>| {
        ImageData::with_dimensions(hnx, hny, 1)
            .with_spacing([sp[0] * 2.0, sp[1] * 2.0, sp[2]])
            .with_origin(input.origin())
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)))
    };
    (
        make("LL", ll),
        make("LH", lh),
        make("HL", hl),
        make("HH", hh),
    )
}

/// Reconstruct image from Haar wavelet sub-bands.
pub fn haar_reconstruct(
    ll: &ImageData,
    lh: &ImageData,
    hl: &ImageData,
    hh: &ImageData,
) -> ImageData {
    let dims = ll.dimensions();
    let (hnx, hny) = (dims[0], dims[1]);
    let nx = hnx * 2;
    let ny = hny * 2;

    let read = |img: &ImageData, name: &str| -> Vec<f64> {
        let arr = img.point_data().get_array(name).unwrap();
        let mut buf = [0.0f64];
        (0..arr.num_tuples())
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            })
            .collect()
    };
    let ll_v = read(ll, "LL");
    let lh_v = read(lh, "LH");
    let hl_v = read(hl, "HL");
    let hh_v = read(hh, "HH");

    let mut data = vec![0.0; nx * ny];
    for iy in 0..hny {
        for ix in 0..hnx {
            let idx = ix + iy * hnx;
            let l = ll_v[idx];
            let lhv = lh_v[idx];
            let hlv = hl_v[idx];
            let hhv = hh_v[idx];
            data[2 * ix + 2 * iy * nx] = l + lhv + hlv + hhv;
            data[2 * ix + 1 + 2 * iy * nx] = l - lhv + hlv - hhv;
            data[2 * ix + (2 * iy + 1) * nx] = l + lhv - hlv - hhv;
            data[2 * ix + 1 + (2 * iy + 1) * nx] = l - lhv - hlv + hhv;
        }
    }

    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing(ll.spacing())
        .with_origin(ll.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Reconstructed",
            data,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decompose() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let (ll, lh, hl, hh) = haar_decompose(&img, "v");
        assert_eq!(ll.dimensions(), [4, 4, 1]);
        assert_eq!(lh.dimensions(), [4, 4, 1]);
        assert_eq!(hl.dimensions(), [4, 4, 1]);
        assert_eq!(hh.dimensions(), [4, 4, 1]);
    }
    #[test]
    fn test_roundtrip() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x * 10.0 + y,
        );
        let (ll, lh, hl, hh) = haar_decompose(&img, "v");
        let rec = haar_reconstruct(&ll, &lh, &hl, &hh);
        assert_eq!(rec.dimensions(), [8, 8, 1]);
        let orig = img.point_data().get_array("v").unwrap();
        let recon = rec.point_data().get_array("Reconstructed").unwrap();
        let mut b1 = [0.0];
        let mut b2 = [0.0];
        for i in 0..64 {
            orig.tuple_as_f64(i, &mut b1);
            recon.tuple_as_f64(i, &mut b2);
            assert!((b1[0] - b2[0]).abs() < 1e-10, "mismatch at {i}");
        }
    }
}
