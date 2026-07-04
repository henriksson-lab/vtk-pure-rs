//! Signed distance field from binary image.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute approximate signed distance field from a binary image.
/// Positive inside, negative outside (or vice versa).
pub fn signed_distance_from_binary(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    if nx == 0 || ny == 0 || nz == 0 {
        return input.clone();
    }
    let n = nx * ny * nz;
    let mut buf = [0.0f64];
    let fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();

    // Two-pass distance transform approximation
    let big = (nx + ny + nz) as f64;
    let mut dist_inside = vec![big; n];
    let mut dist_outside = vec![big; n];

    // Initialize boundary pixels
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let idx = flat_index(ix, iy, iz, nx, ny);
                let is_boundary = is_border(&fg, ix, iy, iz, nx, ny, nz);
                if fg[idx] && is_boundary {
                    dist_inside[idx] = 0.0;
                }
                if !fg[idx] && is_boundary {
                    dist_outside[idx] = 0.0;
                }
            }
        }
    }

    // Forward pass
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let idx = flat_index(ix, iy, iz, nx, ny);
                if ix > 0 {
                    propagate(&mut dist_inside, idx, idx - 1, 1.0);
                    propagate(&mut dist_outside, idx, idx - 1, 1.0);
                }
                if iy > 0 {
                    propagate(&mut dist_inside, idx, idx - nx, 1.0);
                    propagate(&mut dist_outside, idx, idx - nx, 1.0);
                }
                if iz > 0 {
                    propagate(&mut dist_inside, idx, idx - nx * ny, 1.0);
                    propagate(&mut dist_outside, idx, idx - nx * ny, 1.0);
                }
            }
        }
    }
    // Backward pass
    for iz in (0..nz).rev() {
        for iy in (0..ny).rev() {
            for ix in (0..nx).rev() {
                let idx = flat_index(ix, iy, iz, nx, ny);
                if ix + 1 < nx {
                    propagate(&mut dist_inside, idx, idx + 1, 1.0);
                    propagate(&mut dist_outside, idx, idx + 1, 1.0);
                }
                if iy + 1 < ny {
                    propagate(&mut dist_inside, idx, idx + nx, 1.0);
                    propagate(&mut dist_outside, idx, idx + nx, 1.0);
                }
                if iz + 1 < nz {
                    propagate(&mut dist_inside, idx, idx + nx * ny, 1.0);
                    propagate(&mut dist_outside, idx, idx + nx * ny, 1.0);
                }
            }
        }
    }

    let data: Vec<f64> = (0..n)
        .map(|i| {
            if fg[i] {
                dist_inside[i]
            } else {
                -dist_outside[i]
            }
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "SignedDistance",
            data,
            1,
        )))
}

fn flat_index(ix: usize, iy: usize, iz: usize, nx: usize, ny: usize) -> usize {
    iz * ny * nx + iy * nx + ix
}

fn is_border(
    fg: &[bool],
    ix: usize,
    iy: usize,
    iz: usize,
    nx: usize,
    ny: usize,
    nz: usize,
) -> bool {
    let v = fg[flat_index(ix, iy, iz, nx, ny)];
    if ix > 0 && fg[flat_index(ix - 1, iy, iz, nx, ny)] != v {
        return true;
    }
    if ix + 1 < nx && fg[flat_index(ix + 1, iy, iz, nx, ny)] != v {
        return true;
    }
    if iy > 0 && fg[flat_index(ix, iy - 1, iz, nx, ny)] != v {
        return true;
    }
    if iy + 1 < ny && fg[flat_index(ix, iy + 1, iz, nx, ny)] != v {
        return true;
    }
    if iz > 0 && fg[flat_index(ix, iy, iz - 1, nx, ny)] != v {
        return true;
    }
    if iz + 1 < nz && fg[flat_index(ix, iy, iz + 1, nx, ny)] != v {
        return true;
    }
    false
}

fn propagate(dist: &mut [f64], dst: usize, src: usize, cost: f64) {
    let d = dist[src] + cost;
    if d < dist[dst] {
        dist[dst] = d;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sdf() {
        let img = ImageData::from_function(
            [11, 11, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 5.0).abs() < 2.5 && (y - 5.0).abs() < 2.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let r = signed_distance_from_binary(&img, "v");
        let arr = r.point_data().get_array("SignedDistance").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5 + 5 * 11, &mut buf);
        assert!(buf[0] > 0.0); // inside
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.0); // outside
    }
}
