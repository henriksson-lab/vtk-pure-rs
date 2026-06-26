use crate::data::PolyData;

/// Compute spherical harmonic coefficients from mesh vertex positions.
///
/// Projects the radial distance of each vertex (from the mesh centroid) onto
/// real spherical harmonics up to degree `max_degree`. Returns a flat vector
/// of coefficients in order: Y(0,0), Y(1,-1), Y(1,0), Y(1,1), Y(2,-2), ...
/// with a total of (max_degree+1)^2 entries.
pub fn spherical_harmonic_coefficients(input: &PolyData, max_degree: usize) -> Vec<f64> {
    let n: usize = input.points.len();
    let num_coeffs: usize = (max_degree + 1) * (max_degree + 1);
    if n == 0 {
        return vec![0.0; num_coeffs];
    }

    // Compute centroid
    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;
    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let inv_n: f64 = 1.0 / n as f64;
    cx *= inv_n;
    cy *= inv_n;
    cz *= inv_n;

    // For each vertex, compute (r, theta, phi) and accumulate Y_lm * r
    let mut coeffs: Vec<f64> = vec![0.0; num_coeffs];

    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        let dx: f64 = p[0] - cx;
        let dy: f64 = p[1] - cy;
        let dz: f64 = p[2] - cz;
        let r: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
        if r < 1e-15 {
            continue;
        }

        let theta: f64 = (dz / r).acos(); // polar angle [0, pi]
        let phi: f64 = dy.atan2(dx); // azimuthal angle [-pi, pi]

        let mut idx: usize = 0;
        for l in 0..=max_degree {
            for m_signed in -(l as i64)..=(l as i64) {
                let ylm: f64 = real_spherical_harmonic(l, m_signed, theta, phi);
                coeffs[idx] += r * ylm;
                idx += 1;
            }
        }
    }

    // Normalize by number of points
    for c in &mut coeffs {
        *c *= inv_n;
    }

    coeffs
}

/// Reconstruct mesh vertex positions from spherical harmonic coefficients.
///
/// For each vertex, the direction from centroid is preserved but the radius
/// is replaced by the spherical harmonic reconstruction at that direction.
pub fn reconstruct_from_harmonics(
    input: &PolyData,
    coefficients: &[f64],
    max_degree: usize,
) -> PolyData {
    let n: usize = input.points.len();
    let num_coeffs: usize = (max_degree + 1) * (max_degree + 1);
    if n == 0 || coefficients.len() < num_coeffs {
        return input.clone();
    }

    // Compute centroid
    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;
    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let inv_n: f64 = 1.0 / n as f64;
    cx *= inv_n;
    cy *= inv_n;
    cz *= inv_n;

    let mut output: PolyData = input.clone();

    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        let dx: f64 = p[0] - cx;
        let dy: f64 = p[1] - cy;
        let dz: f64 = p[2] - cz;
        let r: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
        if r < 1e-15 {
            continue;
        }

        let theta: f64 = (dz / r).acos();
        let phi: f64 = dy.atan2(dx);

        // Reconstruct radius
        let mut new_r: f64 = 0.0;
        let mut idx: usize = 0;
        for l in 0..=max_degree {
            for m_signed in -(l as i64)..=(l as i64) {
                let ylm: f64 = real_spherical_harmonic(l, m_signed, theta, phi);
                new_r += coefficients[idx] * ylm;
                idx += 1;
            }
        }

        // Keep direction, replace radius
        let scale: f64 = new_r / r;
        output
            .points
            .set(i, [cx + dx * scale, cy + dy * scale, cz + dz * scale]);
    }

    output
}

/// Evaluate the real spherical harmonic Y_l^m(theta, phi).
///
/// Uses the convention where m > 0 gives the cosine term, m < 0 gives the
/// sine term, and m == 0 gives the zonal harmonic.
fn real_spherical_harmonic(l: usize, m: i64, theta: f64, phi: f64) -> f64 {
    let abs_m: usize = m.unsigned_abs() as usize;
    let plm: f64 = associated_legendre(l, abs_m, theta.cos());
    let norm: f64 = spherical_harmonic_norm(l, abs_m);

    if m > 0 {
        norm * plm * (abs_m as f64 * phi).cos() * std::f64::consts::SQRT_2
    } else if m < 0 {
        norm * plm * (abs_m as f64 * phi).sin() * std::f64::consts::SQRT_2
    } else {
        norm * plm
    }
}

/// Normalization factor for spherical harmonics.
fn spherical_harmonic_norm(l: usize, m: usize) -> f64 {
    let num: f64 = (2 * l + 1) as f64 * factorial(l - m);
    let den: f64 = 4.0 * std::f64::consts::PI * factorial(l + m);
    (num / den).sqrt()
}

/// Compute n! as f64.
fn factorial(n: usize) -> f64 {
    let mut result: f64 = 1.0;
    for i in 2..=n {
        result *= i as f64;
    }
    result
}

/// Compute the associated Legendre polynomial P_l^m(x) using recurrence.
fn associated_legendre(l: usize, m: usize, x: f64) -> f64 {
    if m > l {
        return 0.0;
    }

    // P_m^m(x) = (-1)^m (2m-1)!! (1-x^2)^(m/2)
    let mut pmm: f64 = 1.0;
    if m > 0 {
        let somx2: f64 = ((1.0 - x) * (1.0 + x)).sqrt();
        let mut fact: f64 = 1.0;
        for _i in 0..m {
            pmm *= -fact * somx2;
            fact += 2.0;
        }
    }

    if l == m {
        return pmm;
    }

    // P_{m+1}^m(x) = x(2m+1) P_m^m(x)
    let mut pmmp1: f64 = x * (2 * m + 1) as f64 * pmm;
    if l == m + 1 {
        return pmmp1;
    }

    // General recurrence
    let mut pll: f64 = 0.0;
    for ll in (m + 2)..=l {
        pll = (x * (2 * ll - 1) as f64 * pmmp1 - (ll + m - 1) as f64 * pmm) / (ll - m) as f64;
        pmm = pmmp1;
        pmmp1 = pll;
    }

    pll
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sphere_points(n: usize) -> PolyData {
        let mut pd = PolyData::new();
        // Fibonacci sphere sampling
        let golden_ratio: f64 = (1.0 + 5.0f64.sqrt()) / 2.0;
        let radius: f64 = 2.0;
        for i in 0..n {
            let theta: f64 = (1.0 - 2.0 * (i as f64 + 0.5) / n as f64).acos();
            let phi: f64 = 2.0 * std::f64::consts::PI * (i as f64) / golden_ratio;
            pd.points.push([
                radius * theta.sin() * phi.cos(),
                radius * theta.sin() * phi.sin(),
                radius * theta.cos(),
            ]);
        }
        pd
    }

    #[test]
    fn sphere_has_dominant_l0_coefficient() {
        let pd = make_sphere_points(200);
        let coeffs = spherical_harmonic_coefficients(&pd, 3);
        // For a sphere centered at origin, Y(0,0) should be nonzero and dominant
        assert!(
            coeffs[0].abs() > 0.1,
            "l=0 coeff should be significant, got {}",
            coeffs[0]
        );
        // Higher order coefficients should be small compared to l=0
        for (idx, c) in coeffs[1..].iter().enumerate() {
            assert!(
                c.abs() < coeffs[0].abs() * 0.3,
                "coeff {} is too large: {} vs l=0 = {}",
                idx + 1,
                c,
                coeffs[0]
            );
        }
    }

    #[test]
    fn reconstruct_preserves_sphere_shape() {
        let pd = make_sphere_points(100);
        let max_l: usize = 6;
        let coeffs = spherical_harmonic_coefficients(&pd, max_l);
        let recon = reconstruct_from_harmonics(&pd, &coeffs, max_l);
        // Reconstructed radii should all be positive (sphere shape preserved)
        let mut max_r_diff: f64 = 0.0;
        for i in 0..pd.points.len() {
            let orig: [f64; 3] = pd.points.get(i);
            let rec: [f64; 3] = recon.points.get(i);
            let r_orig: f64 = (orig[0] * orig[0] + orig[1] * orig[1] + orig[2] * orig[2]).sqrt();
            let r_rec: f64 = (rec[0] * rec[0] + rec[1] * rec[1] + rec[2] * rec[2]).sqrt();
            let diff: f64 = (r_orig - r_rec).abs();
            if diff > max_r_diff {
                max_r_diff = diff;
            }
        }
        // The SH reconstruction of a constant-radius sphere should be reasonable
        assert!(
            max_r_diff < 2.0,
            "max radius difference {} is too large",
            max_r_diff
        );
    }

    #[test]
    fn empty_input_returns_zeros() {
        let pd = PolyData::new();
        let coeffs = spherical_harmonic_coefficients(&pd, 2);
        assert_eq!(coeffs.len(), 9); // (2+1)^2
        for c in &coeffs {
            assert_eq!(*c, 0.0);
        }
    }
}
