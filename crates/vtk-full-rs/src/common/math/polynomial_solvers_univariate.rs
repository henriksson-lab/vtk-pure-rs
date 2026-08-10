use std::sync::Mutex;

use crate::common::core::{
    math::{pi, random_range},
    object::Object,
};

const SQRT3: f64 = 1.7320508075688772;
const INV3: f64 = 1.0 / 3.0;
const ABSOLUTE0: f64 = 10.0 * f64::MIN_POSITIVE;

static DIVISION_TOLERANCE: Mutex<f64> = Mutex::new(1e-8);

/// VTK: `vtkPolynomialSolversUnivariate`.
#[derive(Debug, Clone)]
pub struct PolynomialSolversUnivariate {
    object: Object,
}

impl PolynomialSolversUnivariate {
    /// VTK: `vtkPolynomialSolversUnivariate::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPolynomialSolversUnivariate"),
        }
    }

    /// VTK: `vtkPolynomialSolversUnivariate::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\n(s) DivisionTolerance: {}",
            self.object.get_class_name(),
            Self::get_division_tolerance()
        )
    }

    /// VTK: `vtkPolynomialSolversUnivariate::PrintPolynomial`.
    pub fn print_polynomial(p: &[f64], deg_p: i32) -> String {
        let mut out = format!("\nThe polynomial has degree {deg_p}\n");
        if deg_p < 0 {
            out.push_str("0\n");
            return out;
        }

        let deg_p = deg_p as usize;
        if deg_p == 0 {
            out.push_str(&format!("{}\n", p[0]));
            return out;
        }

        let deg_pm1 = deg_p - 1;
        for i in 0..deg_pm1 {
            if p[i] > 0.0 {
                if i != 0 {
                    out.push('+');
                }
                if p[i] != 1.0 {
                    out.push_str(&format!("{}*", p[i]));
                }
                out.push_str(&format!("x**{}", deg_p - i));
            } else if p[i] < 0.0 {
                out.push_str(&format!("{}*x**{}", p[i], deg_p - i));
            }
        }

        if p[deg_pm1] > 0.0 {
            out.push_str(&format!("+{}*x", p[deg_pm1]));
        } else if p[deg_pm1] < 0.0 {
            out.push_str(&format!("{}*x", p[deg_pm1]));
        }

        if p[deg_p] > 0.0 {
            out.push_str(&format!("+{}", p[deg_p]));
        } else if p[deg_p] < 0.0 {
            out.push_str(&format!("{}", p[deg_p]));
        }
        out.push('\n');
        out
    }

    /// VTK: `vtkPolynomialSolversUnivariate::HabichtBisectionSolve`.
    pub fn habicht_bisection_solve(
        p: &[f64],
        d: i32,
        a: &[f64],
        upper_bnds: &mut [f64],
        tol: f64,
    ) -> i32 {
        vtk_habicht_or_sturm_bisection_solve(p, d, a, upper_bnds, tol, 0, 0, 1)
    }

    /// VTK: `vtkPolynomialSolversUnivariate::HabichtBisectionSolve`.
    pub fn habicht_bisection_solve_interval(
        p: &[f64],
        d: i32,
        a: &[f64],
        upper_bnds: &mut [f64],
        tol: f64,
        interval_type: i32,
    ) -> i32 {
        vtk_habicht_or_sturm_bisection_solve(p, d, a, upper_bnds, tol, interval_type, 0, 1)
    }

    /// VTK: `vtkPolynomialSolversUnivariate::HabichtBisectionSolve`.
    pub fn habicht_bisection_solve_with_divide_gcd(
        p: &[f64],
        d: i32,
        a: &[f64],
        upper_bnds: &mut [f64],
        tol: f64,
        interval_type: i32,
        divide_gcd: bool,
    ) -> i32 {
        vtk_habicht_or_sturm_bisection_solve(
            p,
            d,
            a,
            upper_bnds,
            tol,
            interval_type,
            if divide_gcd { 1 } else { 0 },
            1,
        )
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SturmBisectionSolve`.
    pub fn sturm_bisection_solve(
        p: &[f64],
        d: i32,
        a: &[f64],
        upper_bnds: &mut [f64],
        tol: f64,
    ) -> i32 {
        vtk_habicht_or_sturm_bisection_solve(p, d, a, upper_bnds, tol, 0, 0, 0)
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SturmBisectionSolve`.
    pub fn sturm_bisection_solve_interval(
        p: &[f64],
        d: i32,
        a: &[f64],
        upper_bnds: &mut [f64],
        tol: f64,
        interval_type: i32,
    ) -> i32 {
        vtk_habicht_or_sturm_bisection_solve(p, d, a, upper_bnds, tol, interval_type, 0, 0)
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SturmBisectionSolve`.
    pub fn sturm_bisection_solve_with_divide_gcd(
        p: &[f64],
        d: i32,
        a: &[f64],
        upper_bnds: &mut [f64],
        tol: f64,
        interval_type: i32,
        divide_gcd: bool,
    ) -> i32 {
        vtk_habicht_or_sturm_bisection_solve(
            p,
            d,
            a,
            upper_bnds,
            tol,
            interval_type,
            if divide_gcd { 1 } else { 0 },
            0,
        )
    }

    /// VTK: `vtkPolynomialSolversUnivariate::FilterRoots`.
    pub fn filter_roots(
        p: &[f64],
        mut d: i32,
        upper_bnds: &mut [f64],
        mut root_count: i32,
        diameter: f64,
    ) -> i32 {
        upper_bnds[..root_count as usize].sort_by(polynomial_solvers_univariate_compare_roots);

        let mut j = 1;
        while j < root_count {
            let ju = j as usize;
            if upper_bnds[ju] < upper_bnds[ju - 1] + diameter {
                for k in ju + 1..root_count as usize {
                    upper_bnds[k - 1] = upper_bnds[k];
                }
                j -= 1;
                root_count -= 1;
            }
            j += 1;
        }

        if root_count == 0 {
            return 0;
        }

        for i in (0..=d as usize).rev() {
            if is_zero(p[i]) {
                d -= 1;
            } else {
                break;
            }
        }

        let mut dp = vec![0.0; ((d + 2) * (d + 1) / 2) as usize];
        for i in 0..=d as usize {
            dp[i] = p[i];
        }

        vtk_get_derivative_sequence(&mut dp, d);

        let mut i = 0;
        while i < root_count {
            let iu = i as usize;
            if upper_bnds[iu].abs() < diameter {
                i += 1;
                continue;
            }

            if vtk_get_sign_changes_for_derivative_sequence(&dp, d, upper_bnds[iu])
                == vtk_get_sign_changes_for_derivative_sequence(&dp, d, upper_bnds[iu] - diameter)
            {
                for j in iu + 1..root_count as usize {
                    upper_bnds[j - 1] = upper_bnds[j];
                }
                i -= 1;
                root_count -= 1;
            }
            i += 1;
        }

        root_count
    }

    /// VTK: `vtkPolynomialSolversUnivariate::LinBairstowSolve`.
    pub fn lin_bairstow_solve(c: &mut [f64], d: i32, r: &mut [f64], tolerance: &mut f64) -> i32 {
        if is_zero(c[0]) {
            return 0;
        }

        let dp1 = d + 1;
        for i in 1..dp1 as usize {
            c[i] /= c[0];
        }

        let mut div1 = vec![0.0; dp1 as usize];
        let mut div2 = vec![0.0; dp1 as usize];
        div1[0] = 1.0;
        div2[0] = 1.0;

        let mut i = d;
        while i > 2 {
            let mut r_factor: f64 = 0.0;
            let mut s_factor: f64 = 0.0;
            let mut d_r: f64 = 1.0;
            let mut d_s: f64 = 0.0;
            let mut n_iterations = 1;

            while (d_r.abs() + d_s.abs()) > *tolerance {
                if n_iterations % 100 == 0 {
                    r_factor = random_range(0.0, 2.0);
                    if n_iterations % 200 == 0 {
                        *tolerance *= 4.0;
                    }
                }

                div1[1] = c[1] - r_factor;
                div2[1] = div1[1] - r_factor;

                for j in 2..=i as usize {
                    div1[j] = c[j] - r_factor * div1[j - 1] - s_factor * div1[j - 2];
                    div2[j] = div1[j] - r_factor * div2[j - 1] - s_factor * div2[j - 2];
                }

                let u = div2[(i - 1) as usize] * div2[(i - 3) as usize];
                let v = div2[(i - 2) as usize] * div2[(i - 2) as usize];
                let (det, det_r, det_s) = if are_equal(u, v, 1.0e-6) {
                    (1.0, 1.0, 1.0)
                } else {
                    (
                        u - v,
                        div1[i as usize] * div2[(i - 3) as usize]
                            - div1[(i - 1) as usize] * div2[(i - 2) as usize],
                        div1[(i - 1) as usize] * div2[(i - 1) as usize]
                            - div1[i as usize] * div2[(i - 2) as usize],
                    )
                };

                d_r = det_r / det;
                d_s = det_s / det;

                if d_r.abs() + d_s.abs() > 10.0 {
                    d_r = random_range(-1.0, 1.0);
                    d_s = random_range(-1.0, 1.0);
                }

                r_factor += d_r;
                s_factor += d_s;
                n_iterations += 1;
            }

            for j in 0..(i - 1) as usize {
                c[j] = div1[j];
            }
            c[i as usize] = s_factor;
            c[(i - 1) as usize] = r_factor;
            i -= 2;
        }

        let mut nr = 0;
        let mut i = d;
        while i >= 2 {
            let mut delta = c[(i - 1) as usize] * c[(i - 1) as usize] - 4.0 * c[i as usize];
            if delta >= 0.0 {
                if delta != 0.0 {
                    delta = delta.sqrt();
                    r[nr] = (-c[(i - 1) as usize] - delta) / 2.0;
                    nr += 1;
                    r[nr] = (-c[(i - 1) as usize] + delta) / 2.0;
                    nr += 1;
                } else {
                    r[nr] = -c[1];
                    nr += 1;
                    r[nr] = -c[1];
                    nr += 1;
                }
            }
            i -= 2;
        }

        if d % 2 == 1 {
            r[nr] = -c[1];
            nr += 1;
        }

        nr as i32
    }

    /// VTK: `vtkPolynomialSolversUnivariate::FerrariSolve`.
    pub fn ferrari_solve(c: &[f64], r: &mut [f64], m: &mut [i32], tol: f64) -> i32 {
        if c[3].abs() <= tol {
            if c[2].abs() <= tol {
                if c[1].abs() <= tol {
                    if c[0].abs() <= tol {
                        r[0] = 0.0;
                        m[0] = 4;
                        return 1;
                    }

                    r[0] = -c[1];
                    m[0] = 1;
                    r[1] = 0.0;
                    m[1] = 3;
                    return 2;
                }

                let cc = [1.0, c[0], c[1]];
                let nr = Self::solve_quadratic_coefficients(&cc, r, m);
                r[nr as usize] = 0.0;
                m[nr as usize] = 2;
                return nr + 1;
            }

            let nr = Self::tartaglia_cardan_solve(c, r, m, tol);
            r[nr as usize] = 0.0;
            m[nr as usize] = 1;
            return nr + 1;
        }

        if c[0].abs() <= tol && c[2].abs() <= tol {
            if c[1].abs() <= tol {
                if c[3] < 0.0 {
                    return 0;
                }
                r[0] = c[3].sqrt().sqrt();
                m[0] = 4;
                return 1;
            }

            let cc = [1.0, c[1], c[3]];
            let mut cr = [0.0; 2];
            let mut cm = [0; 2];
            let nr1 = Self::solve_quadratic_coefficients(&cc, &mut cr, &mut cm);
            let mut nr = 0;
            for i in 0..nr1 as usize {
                if cr[i].abs() <= tol {
                    r[nr] = 0.0;
                    m[nr] = 2 * cm[i];
                    nr += 1;
                } else if cr[i] > tol {
                    r[nr] = cr[i].sqrt();
                    m[nr] = cm[i];
                    nr += 1;
                    r[nr] = -cr[i].sqrt();
                    m[nr] = cm[i];
                    nr += 1;
                }
            }
            return nr as i32;
        }

        let p2d8 = c[0] * c[0] * 0.125;
        let qd2 = c[1] * 0.5;
        let a = c[1] - 3.0 * p2d8;
        let b = c[0] * (p2d8 - qd2) + c[2];
        let d = p2d8 * (qd2 - 0.75 * p2d8) - c[0] * c[2] * 0.25 + c[3];

        if b.abs() <= tol {
            let cc = [1.0, a, d];
            let mut cr = [0.0; 2];
            let mut cm = [0; 2];
            let nr1 = Self::solve_quadratic_coefficients(&cc, &mut cr, &mut cm);
            let mut nr = 0;
            let shift = -c[0] * 0.25;
            for i in 0..nr1 as usize {
                if cr[i].abs() <= tol {
                    r[nr] = shift;
                    m[nr] = 2 * cm[i];
                    nr += 1;
                } else if cr[i] > tol {
                    r[nr] = cr[i].sqrt() + shift;
                    m[nr] = cm[i];
                    nr += 1;
                    r[nr] = -cr[i].sqrt() + shift;
                    m[nr] = cm[i];
                    nr += 1;
                }
            }
            return nr as i32;
        }

        let mut cc = [2.0 * a, a * a - 4.0 * d, -b * b];
        let mut cr = [0.0; 3];
        let mut cm = [0; 3];
        let mut nr = Self::tartaglia_cardan_solve(&cc, &mut cr, &mut cm, tol);

        nr -= 1;
        let mut alpha2 = cr[nr as usize];
        while alpha2 < 0.0 && nr != 0 {
            nr -= 1;
            alpha2 = cr[nr as usize];
        }

        cc[0] = 1.0;
        cc[1] = alpha2.sqrt();
        let rho = -b / cc[1];
        cc[2] = (a + alpha2 + rho) * 0.5;
        let nr1 = Self::solve_quadratic_coefficients(&cc, r, m);
        cc[1] = -cc[1];
        cc[2] -= rho;
        nr = nr1
            + Self::solve_quadratic_coefficients(
                &cc,
                &mut r[nr1 as usize..],
                &mut m[nr1 as usize..],
            );
        if nr == 0 {
            return 0;
        }

        let mut unsorted = Vec::with_capacity(nr as usize);
        for i in 0..nr as usize {
            unsorted.push((r[i], m[i]));
        }
        unsorted.sort_by(|a, b| polynomial_solvers_univariate_compare_roots(&a.0, &b.0));

        r[0] = unsorted[0].0;
        m[0] = unsorted[0].1;
        let mut nr1 = 1;
        for i in 1..nr as usize {
            if unsorted[i].0 == unsorted[i - 1].0 {
                m[i - 1] += unsorted[i].1;
                continue;
            }
            r[nr1] = unsorted[i].0;
            m[nr1] = unsorted[i].1;
            nr1 += 1;
        }

        let shift = -c[0] * 0.25;
        for root in r.iter_mut().take(nr1) {
            *root += shift;
        }

        nr1 as i32
    }

    /// VTK: `vtkPolynomialSolversUnivariate::TartagliaCardanSolve`.
    pub fn tartaglia_cardan_solve(c: &[f64], r: &mut [f64], m: &mut [i32], tol: f64) -> i32 {
        if c[2].abs() <= tol {
            r[0] = 0.0;
            if c[1].abs() <= tol {
                if c[0].abs() <= tol {
                    m[0] = 3;
                    return 1;
                }

                m[0] = 2;
                r[1] = -c[0];
                m[1] = 1;
                return 2;
            }

            m[0] = 1;
            let a2 = c[0] * c[0];
            let fourc1 = 4.0 * c[1];
            let mut delta = a2 - fourc1;
            let threshold = tol * a2.max(fourc1.abs());
            if delta > threshold {
                delta = delta.sqrt();
                r[1] = (-delta - c[0]) * 0.5;
                m[1] = 1;
                r[2] = (delta - c[0]) * 0.5;
                m[2] = 1;
                return 3;
            }

            if delta < -threshold {
                return 1;
            }

            r[1] = -c[0] * 0.5;
            m[1] = 2;
            return 2;
        }

        let shift = -c[0] / 3.0;
        let a2 = c[0] * c[0];
        let p = c[1] - a2 / 3.0;
        let q = c[0] * (2.0 * a2 / 9.0 - c[1]) / 3.0 + c[2];

        if p.abs() <= tol {
            if q.abs() <= tol {
                r[0] = shift;
                m[0] = 3;
                return 1;
            }

            let x = if q < 0.0 {
                (-q).powf(INV3)
            } else {
                -q.powf(INV3)
            };
            r[0] = x + shift;
            m[0] = 3;
            return 1;
        }

        if q.abs() <= tol {
            r[0] = shift;
            m[0] = 1;
            if p < 0.0 {
                let x = (-p).sqrt();
                r[1] = x + shift;
                r[2] = -x + shift;
                m[1] = 1;
                m[2] = 1;
                return 3;
            }
            return 1;
        }

        let p_3 = p * INV3;
        let q_2 = q * 0.5;
        let d = p_3 * p_3 * p_3 + q_2 * q_2;

        if d.abs() <= tol {
            let u = if q > 0.0 {
                -q_2.powf(INV3)
            } else {
                (-q_2).powf(INV3)
            };
            r[0] = 2.0 * u + shift;
            m[0] = 1;
            r[1] = -u + shift;
            m[1] = 2;
            return 2;
        }

        if d > 0.0 {
            let mut u = d.sqrt() - q_2;
            u = if u < 0.0 {
                -(-u).powf(INV3)
            } else {
                u.powf(INV3)
            };
            r[0] = u - p_3 / u + shift;
            m[0] = 1;
            return 1;
        }

        let smp_3 = (-p_3).sqrt();
        let argu = (q_2 / (p_3 * smp_3)).acos() * INV3;
        let mut x1 = argu.cos();
        let mut x2 = SQRT3 * (1.0 - x1 * x1).sqrt();
        x1 *= smp_3;
        x2 *= smp_3;
        r[0] = 2.0 * x1 + shift;
        r[1] = x2 - x1 + shift;
        r[2] = r[1] - 2.0 * x2;
        m[0] = 1;
        m[1] = 1;
        m[2] = 1;
        3
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveCubic`.
    pub fn solve_cubic(c0: f64, c1: f64, c2: f64, c3: f64) -> [f64; 5] {
        let mut roots = [0.0; 5];
        let (code, values, num_roots) = Self::solve_cubic_values(c0, c1, c2, c3);
        roots[0] = f64::from(num_roots);
        roots[1] = values[0];
        roots[2] = values[1];
        roots[3] = values[2];
        roots[4] = f64::from(code);
        roots
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveCubic`.
    pub fn solve_cubic_into(
        c0: f64,
        c1: f64,
        c2: f64,
        c3: f64,
        r1: &mut f64,
        r2: &mut f64,
        r3: &mut f64,
        num_roots: &mut i32,
    ) -> i32 {
        let (code, values, count) = Self::solve_cubic_values(c0, c1, c2, c3);
        *r1 = values[0];
        *r2 = values[1];
        *r3 = values[2];
        *num_roots = count;
        code
    }

    fn solve_cubic_values(c0: f64, mut c1: f64, mut c2: f64, mut c3: f64) -> (i32, [f64; 3], i32) {
        let mut roots = [0.0; 3];
        if c0 != 0.0 {
            c1 /= c0;
            c2 /= c0;
            c3 /= c0;

            let q = (c1 * c1 - 3.0 * c2) / 9.0;
            let r = (2.0 * c1 * c1 * c1 - 9.0 * c1 * c2 + 27.0 * c3) / 54.0;
            let r_squared = r * r;
            let q_cubed = q * q * q;

            if r_squared <= q_cubed {
                if q_cubed == 0.0 {
                    roots[0] = -c1 / 3.0;
                    roots[1] = roots[0];
                    roots[2] = roots[0];
                    return (1, roots, 1);
                }

                let theta = (r / q_cubed.sqrt()).acos();
                roots[0] = -2.0 * q.sqrt() * (theta / 3.0).cos() - c1 / 3.0;
                roots[1] = -2.0 * q.sqrt() * ((theta + 2.0 * pi()) / 3.0).cos() - c1 / 3.0;
                roots[2] = -2.0 * q.sqrt() * ((theta - 2.0 * pi()) / 3.0).cos() - c1 / 3.0;

                let mut num_roots = 3;
                if roots[0] == roots[1] {
                    num_roots = 2;
                    roots[1] = roots[2];
                } else if roots[0] == roots[2] {
                    num_roots = 2;
                }

                if roots[1] == roots[2] && num_roots == 3 {
                    num_roots = 2;
                }

                if roots[0] == roots[1] {
                    num_roots = 1;
                }
                return (num_roots, roots, num_roots);
            }

            let a = -vtk_sign(r) * (r.abs() + (r_squared - q_cubed).sqrt()).powf(1.0 / 3.0);
            let b = if a == 0.0 { 0.0 } else { q / a };
            roots[0] = (a + b) - c1 / 3.0;
            roots[1] = -0.5 * (a + b) - c1 / 3.0;
            roots[2] = SQRT3 / 2.0 * (a - b);
            return (-3, roots, 1);
        }

        let (code, values, count) = Self::solve_quadratic_values(c1, c2, c3);
        roots[0] = values[0];
        roots[1] = values[1];
        (code, roots, count)
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveQuadratic`.
    pub fn solve_quadratic(c0: f64, c1: f64, c2: f64) -> [f64; 4] {
        let mut roots = [0.0; 4];
        let (code, values, num_roots) = Self::solve_quadratic_values(c0, c1, c2);
        roots[0] = f64::from(num_roots);
        roots[1] = values[0];
        roots[2] = values[1];
        roots[3] = f64::from(code);
        roots
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveQuadratic`.
    pub fn solve_quadratic_into(
        c0: f64,
        c1: f64,
        c2: f64,
        r1: &mut f64,
        r2: &mut f64,
        num_roots: &mut i32,
    ) -> i32 {
        let (code, values, count) = Self::solve_quadratic_values(c0, c1, c2);
        *r1 = values[0];
        *r2 = values[1];
        *num_roots = count;
        code
    }

    fn solve_quadratic_values(c0: f64, c1: f64, c2: f64) -> (i32, [f64; 2], i32) {
        let mut roots = [0.0; 2];
        if c0 != 0.0 {
            let determinant = c1 * c1 - 4.0 * c0 * c2;
            if determinant >= 0.0 {
                let q = -0.5 * (c1 + vtk_sign(c1) * determinant.sqrt());
                roots[0] = q / c0;
                roots[1] = if q == 0.0 { 0.0 } else { c2 / q };
                let num_roots = if roots[0] == roots[1] { 1 } else { 2 };
                return (num_roots, roots, num_roots);
            }

            return (-2, roots, 0);
        }

        let (code, root, count) = Self::solve_linear_value(c1, c2);
        roots[0] = root;
        (code, roots, count)
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveQuadratic`.
    pub fn solve_quadratic_coefficients(c: &[f64], r: &mut [f64], m: &mut [i32]) -> i32 {
        if c[0] == 0.0 {
            if c[1] != 0.0 {
                r[0] = -c[2] / c[1];
                m[0] = 1;
                return 1;
            }
            if c[2] != 0.0 {
                return 0;
            }
            return -1;
        }

        let mut delta = c[1] * c[1] - 4.0 * c[0] * c[2];
        if delta >= 0.0 {
            let fac = 1.0 / (2.0 * c[0]);
            if delta != 0.0 {
                delta = delta.sqrt();
                r[0] = (-delta - c[1]) * fac;
                m[0] = 1;
                r[1] = (delta - c[1]) * fac;
                m[1] = 1;
                return 2;
            }

            r[0] = -c[1] * fac;
            m[0] = 2;
            return 1;
        }

        0
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveLinear`.
    pub fn solve_linear(c0: f64, c1: f64) -> [f64; 3] {
        let mut roots = [0.0; 3];
        let (code, root, num_roots) = Self::solve_linear_value(c0, c1);
        roots[0] = f64::from(num_roots);
        roots[1] = root;
        roots[2] = f64::from(code);
        roots
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SolveLinear`.
    pub fn solve_linear_into(c0: f64, c1: f64, r1: &mut f64, num_roots: &mut i32) -> i32 {
        let (code, root, count) = Self::solve_linear_value(c0, c1);
        *r1 = root;
        *num_roots = count;
        code
    }

    fn solve_linear_value(c0: f64, c1: f64) -> (i32, f64, i32) {
        if c0 != 0.0 {
            let root = -c1 / c0;
            return (1, root, 1);
        }

        if c1 == 0.0 {
            (-1, 0.0, 0)
        } else {
            (0, 0.0, 0)
        }
    }

    /// VTK: `vtkPolynomialSolversUnivariate::SetDivisionTolerance`.
    pub fn set_division_tolerance(tol: f64) {
        *DIVISION_TOLERANCE
            .lock()
            .expect("division tolerance mutex poisoned") = tol;
    }

    /// VTK: `vtkPolynomialSolversUnivariate::GetDivisionTolerance`.
    pub fn get_division_tolerance() -> f64 {
        *DIVISION_TOLERANCE
            .lock()
            .expect("division tolerance mutex poisoned")
    }
}

impl Default for PolynomialSolversUnivariate {
    fn default() -> Self {
        Self::new()
    }
}

fn vtk_sign(x: f64) -> f64 {
    if x < 0.0 {
        -1.0
    } else {
        1.0
    }
}

fn is_zero(x: f64) -> bool {
    x.abs() < ABSOLUTE0
}

fn are_equal(x: f64, y: f64, r_tol: f64) -> bool {
    let delta = (x - y).abs();
    if delta < ABSOLUTE0 {
        return true;
    }

    let absx = x.abs();
    let absy = y.abs();
    if absx > absy {
        delta <= r_tol * absx
    } else {
        delta <= r_tol * absy
    }
}

fn polynomial_eucli_div(
    a: &[f64],
    m: i32,
    b: &[f64],
    n: i32,
    q: &mut [f64],
    r: &mut [f64],
    rtol: f64,
) -> i32 {
    let m_mn = m - n;
    if m_mn < 0 {
        q[0] = 0.0;
        for i in 0..=m as usize {
            r[i] = a[i];
        }
        return m;
    }

    let i_b0 = 1.0 / b[0];
    if n == 0 {
        for i in 0..=m as usize {
            q[i] = a[i] * i_b0;
        }
        return -1;
    }

    let m_mn = m_mn as usize;
    let n = n as usize;
    let m = m as usize;

    for i in 0..=m_mn {
        let nj = i.min(n);
        q[i] = a[i];
        for j in 1..=nj {
            q[i] -= b[j] * q[i - j];
        }
        q[i] *= i_b0;
    }

    let mut null_coeff = false;
    let mut result_degree = 0;
    for i in 1..=n {
        let mut sum = 0.0;
        let nj = (m_mn + 1).min(i);
        for j in 0..nj {
            sum += b[n - i + 1 + j] * q[m_mn - j];
        }

        if !are_equal(a[m - i + 1], sum, rtol) {
            r[n - i] = a[m - i + 1] - sum;
            result_degree = (i - 1) as i32;
        } else {
            r[n - i] = 0.0;
            if n == i {
                null_coeff = true;
            }
        }
    }

    if result_degree == 0 && null_coeff {
        -1
    } else {
        result_degree
    }
}

fn polynomial_eucli_div_opposite_r(
    a: &[f64],
    m: i32,
    b: &[f64],
    n: i32,
    minus_r: &mut [f64],
    rtol: f64,
) -> i32 {
    let m_mn = m - n;
    if m_mn < 0 {
        for i in 0..=m as usize {
            minus_r[i] = a[i];
        }
        return m;
    }

    if n == 0 {
        return -1;
    }

    let m_mn = m_mn as usize;
    let n = n as usize;
    let m = m as usize;
    let i_b0 = 1.0 / b[0];
    let mut q = vec![0.0; m_mn + 1];

    for i in 0..=m_mn {
        let nj = i.min(n);
        q[i] = a[i];
        for j in 1..=nj {
            q[i] -= b[j] * q[i - j];
        }
        q[i] *= i_b0;
    }

    let mut null_coeff = false;
    let mut result_degree = 0;
    for i in 1..=n {
        let mut sum = 0.0;
        let nj = (m_mn + 1).min(i);
        for j in 0..nj {
            sum += b[n - i + 1 + j] * q[m_mn - j];
        }

        if !are_equal(a[m - i + 1], sum, rtol) {
            minus_r[n - i] = sum - a[m - i + 1];
            result_degree = (i - 1) as i32;
        } else {
            minus_r[n - i] = 0.0;
            if n == i {
                null_coeff = true;
            }
        }
    }

    if result_degree == 0 && null_coeff {
        -1
    } else {
        result_degree
    }
}

fn vtk_normalize_poly_coeff(mut d: f64) -> f64 {
    const HIGH: f64 = 18446744073709551616.0;
    const REALLY_BIG: f64 = 1.0e300;
    const REALLY_BIG_INV: f64 = 1.0 / REALLY_BIG;
    const NOT_THAT_BIG: f64 = 1.0e30;
    const NOT_THAT_BIG_INV: f64 = 1.0e-30;

    if d.abs() < REALLY_BIG {
        while d.abs() > NOT_THAT_BIG {
            d /= HIGH;
        }
    }
    if d.abs() > REALLY_BIG_INV {
        while d.abs() < NOT_THAT_BIG_INV {
            d *= HIGH;
        }
    }
    d
}

fn vtk_normalize_poly_coeff_with_div(mut d: f64, div: &mut f64) -> f64 {
    const HIGH: f64 = 18446744073709551616.0;
    const REALLY_BIG: f64 = 1.0e300;
    const REALLY_BIG_INV: f64 = 1.0 / REALLY_BIG;
    const NOT_THAT_BIG: f64 = 1.0e30;
    const NOT_THAT_BIG_INV: f64 = 1.0e-30;

    if d.abs() < REALLY_BIG {
        while d.abs() > NOT_THAT_BIG {
            d /= HIGH;
            *div /= HIGH;
        }
    }
    if d.abs() > REALLY_BIG_INV {
        while d.abs() < NOT_THAT_BIG_INV {
            d *= HIGH;
            *div *= HIGH;
        }
    }
    d
}

fn polynomial_eucli_div_opposite_r_scaled(
    mul: f64,
    ai: &[f64],
    m: i32,
    b: &[f64],
    n: i32,
    mut div: f64,
    minus_r: &mut [f64],
    rtol: f64,
) -> i32 {
    let m_mn = m - n;
    for i in 0..=m as usize {
        minus_r[i] = mul * ai[i];
    }

    if m_mn < 0 {
        return m;
    }

    if n == 0 {
        return -1;
    }

    div = 1.0 / div;
    let m_mn = m_mn as usize;
    let n = n as usize;
    let m = m as usize;
    let i_b0 = 1.0 / b[0];
    let mut q = vec![0.0; m_mn + 1];

    for i in 0..=m_mn {
        let nj = i.min(n);
        q[i] = minus_r[i];
        for j in 1..=nj {
            q[i] -= b[j] * q[i - j];
        }
        q[i] *= i_b0;
    }

    let mut null_coeff = false;
    let mut result_degree = 0;
    for i in (1..=n).rev() {
        let mut sum = 0.0;
        let nj = (m_mn + 1).min(i);
        for j in 0..nj {
            sum += b[n - i + 1 + j] * q[m_mn - j];
        }

        if !are_equal(minus_r[m - i + 1], sum, rtol) {
            minus_r[n - i] = (sum - minus_r[m - i + 1]) * div;
            if result_degree == 0 {
                minus_r[n - i] = vtk_normalize_poly_coeff_with_div(minus_r[n - i], &mut div);
                result_degree = (i - 1) as i32;
            }
        } else {
            minus_r[n - i] = 0.0;
            if n == i {
                null_coeff = true;
            }
        }
    }

    if result_degree == 0 && null_coeff {
        -1
    } else {
        result_degree
    }
}

fn evaluate_horner(p: &[f64], d: i32, x: f64) -> f64 {
    if d == -1 {
        return 0.0;
    }

    let d = d as usize;
    let mut val = p[0];
    for coeff in p.iter().take(d + 1).skip(1) {
        val = val * x + *coeff;
    }
    val
}

fn vtk_get_sign_changes(
    p: &[f64],
    deg_p: &[i32],
    offsets: &[i32],
    count: i32,
    val: f64,
    fsign: Option<&mut i32>,
) -> i32 {
    let mut old_val = 0;
    let mut changes = 0;
    let mut fsign = fsign;

    for i in 0..count as usize {
        let offset = offsets[i] as usize;
        let v = evaluate_horner(&p[offset..], deg_p[i], val);

        if i == 0 {
            if let Some(sign) = fsign.as_deref_mut() {
                *sign = if is_zero(v) {
                    0
                } else if v > 0.0 {
                    1
                } else {
                    -1
                };
            }
        }

        if v == 0.0 {
            continue;
        }

        if v * f64::from(old_val) < 0.0 {
            changes += 1;
            old_val = -old_val;
        }

        if old_val == 0 {
            old_val = if v < 0.0 { -1 } else { 1 };
        }
    }

    changes
}

fn polynomial_solvers_univariate_compare_roots(a: &f64, b: &f64) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
}

fn vtk_get_habicht_sequence(
    p: &[f64],
    d: i32,
    sss: &mut [f64],
    degrees: &mut [i32],
    offsets: &mut [i32],
    rtol: f64,
) -> i32 {
    degrees[0] = d;
    offsets[0] = 0;

    let dp1 = d + 1;
    let mut t = vec![0.0; dp1 as usize];
    let mut s = vec![0.0; dp1 as usize];

    degrees[1] = d - 1;
    offsets[1] = dp1;
    let mut offset = dp1;

    for m in 0..d as usize {
        sss[m] = p[m];
        sss[m + offset as usize] = f64::from(d - m as i32) * sss[m];
    }
    sss[d as usize] = p[d as usize];
    t[0] = if p[0] > 0.0 { 1.0 } else { -1.0 };
    s[0] = t[0];
    t[1] = sss[offset as usize];
    s[1] = t[1];

    let mut j = 0;
    let deg = d;
    let mut degree = d - 1;
    let mut jp1 = 1;
    let mut ip1 = 0;

    while degree > 0 && j < d - 1 {
        let k = deg - degree;
        if k == jp1 {
            s[jp1 as usize] = t[jp1 as usize];

            let a_start = offsets[ip1 as usize] as usize;
            let a_len = degrees[ip1 as usize] as usize + 1;
            let b_start = offset as usize;
            let b_len = degree as usize + 1;
            let dest_start = (offsets[k as usize] + degree + 1) as usize;
            let ai = sss[a_start..a_start + a_len].to_vec();
            let b = sss[b_start..b_start + b_len].to_vec();
            degrees[k as usize + 1] = polynomial_eucli_div_opposite_r_scaled(
                s[jp1 as usize] * s[jp1 as usize],
                &ai,
                degrees[ip1 as usize],
                &b,
                degree,
                s[j as usize] * t[ip1 as usize],
                &mut sss[dest_start..],
                rtol,
            );
            offsets[k as usize + 1] = offset + 2 * degree - degrees[k as usize + 1];
        } else {
            s[jp1 as usize] = 0.0;
            for delta in 1..k - j {
                let idx = (jp1 + delta) as usize;
                t[idx] = (t[jp1 as usize] * t[(j + delta) as usize]) / s[j as usize];
                t[idx] = vtk_normalize_poly_coeff(t[idx]);
                if delta % 2 != 0 {
                    t[idx] *= -1.0;
                }
            }
            s[k as usize] = t[k as usize];

            offsets[k as usize] = offsets[jp1 as usize] + degrees[jp1 as usize] + 1;
            degrees[k as usize] = degrees[jp1 as usize];
            for dg in 0..=degree as usize {
                sss[offsets[k as usize] as usize + dg] =
                    (s[k as usize] * sss[offset as usize + dg]) / t[jp1 as usize];
            }

            for l in j + 2..k {
                degrees[l as usize] = -1;
                offsets[l as usize] = offsets[k as usize];
                s[l as usize] = 0.0;
            }

            let a_start = offsets[ip1 as usize] as usize;
            let a_len = degrees[ip1 as usize] as usize + 1;
            let b_start = offset as usize;
            let b_len = degree as usize + 1;
            let dest_start = (offsets[k as usize] + degrees[k as usize] + 1) as usize;
            let ai = sss[a_start..a_start + a_len].to_vec();
            let b = sss[b_start..b_start + b_len].to_vec();
            degrees[k as usize + 1] = polynomial_eucli_div_opposite_r_scaled(
                t[jp1 as usize] * s[k as usize],
                &ai,
                degrees[ip1 as usize],
                &b,
                degree,
                s[j as usize] * t[ip1 as usize],
                &mut sss[dest_start..],
                rtol,
            );
            offsets[k as usize + 1] =
                offsets[k as usize] + 2 * degrees[k as usize] - degrees[k as usize + 1];
        }

        t[k as usize + 1] = sss[offsets[k as usize + 1] as usize];
        ip1 = jp1;
        j = k;
        jp1 = j + 1;
        degree = degrees[jp1 as usize];
        offset = offsets[jp1 as usize];
    }

    if degree == 0 {
        jp1 + 1
    } else {
        while degrees[jp1 as usize] < 0 {
            jp1 -= 1;
        }
        jp1 + 1
    }
}

fn vtk_get_sturm_sequence(
    p: &[f64],
    d: i32,
    sss: &mut [f64],
    degrees: &mut [i32],
    offsets: &mut [i32],
    rtol: f64,
) -> i32 {
    degrees[0] = d;
    offsets[0] = 0;

    let dp1 = d + 1;
    let dm1 = d - 1;
    degrees[1] = dm1;
    offsets[1] = dp1;
    let mut offset = dp1;
    let mut n_sss = 1;

    for k in 0..d as usize {
        sss[k] = p[k];
        sss[k + offset as usize] = f64::from(d - k as i32) * p[k];
    }
    sss[d as usize] = p[d as usize];

    let mut degree = dm1;
    while degrees[n_sss as usize] > 0 {
        n_sss += 1;
        let a_start = offsets[n_sss as usize - 2] as usize;
        let a_len = degrees[n_sss as usize - 2] as usize + 1;
        let b_start = offset as usize;
        let b_len = degree as usize + 1;
        let dest_start = (offset + degree + 1) as usize;
        let a = sss[a_start..a_start + a_len].to_vec();
        let b = sss[b_start..b_start + b_len].to_vec();
        degrees[n_sss as usize] = polynomial_eucli_div_opposite_r(
            &a,
            degrees[n_sss as usize - 2],
            &b,
            degree,
            &mut sss[dest_start..],
            rtol,
        );

        offsets[n_sss as usize] = offset + 2 * degree - degrees[n_sss as usize];

        offset = offsets[n_sss as usize];
        degree = degrees[n_sss as usize];
    }

    if degrees[n_sss as usize] < 0 {
        n_sss
    } else {
        n_sss + 1
    }
}

fn vtk_habicht_or_sturm_bisection_solve(
    p: &[f64],
    mut d: i32,
    a: &[f64],
    upper_bnds: &mut [f64],
    tol: f64,
    interval_type: i32,
    divide_gcd: i32,
    method: i32,
) -> i32 {
    if tol <= 0.0 {
        return -1;
    }

    if is_zero(p[0]) {
        return -1;
    }

    if d < 1 {
        return -1;
    }

    if a[1] < a[0] + tol {
        return -1;
    }

    let mut zero_root = false;
    if is_zero(p[d as usize]) {
        zero_root = true;
        while is_zero(p[d as usize]) {
            d -= 1;
        }
    }

    if d == 0 {
        if zero_root {
            upper_bnds[0] = 0.0;
            return 1;
        }
        return 0;
    }

    let sequence_len = (((d + 1) * (d + 2)) / 2 + 2) as usize;
    let mut sss = vec![0.0; sequence_len];
    let mut degrees = vec![0; d as usize + 1];
    let mut offsets = vec![0; d as usize + 1];
    let mut bounds = [a[0], a[1]];

    let n_sss = if method == 0 {
        vtk_get_sturm_sequence(
            p,
            d,
            &mut sss,
            &mut degrees,
            &mut offsets,
            PolynomialSolversUnivariate::get_division_tolerance(),
        )
    } else {
        vtk_get_habicht_sequence(
            p,
            d,
            &mut sss,
            &mut degrees,
            &mut offsets,
            PolynomialSolversUnivariate::get_division_tolerance(),
        )
    };

    if degrees[n_sss as usize - 1] > 0 && divide_gcd == 1 {
        let mut r = vec![0.0; d as usize + 1];
        let mut q = vec![0.0; d as usize + 1];
        let gcd_offset = offsets[n_sss as usize - 1] as usize;
        let gcd_degree = degrees[n_sss as usize - 1];
        let gcd_len = gcd_degree as usize + 1;
        let _deg = polynomial_eucli_div(
            &sss,
            d,
            &sss[gcd_offset..gcd_offset + gcd_len],
            gcd_degree,
            &mut q,
            &mut r,
            PolynomialSolversUnivariate::get_division_tolerance(),
        );
        let deg = d - gcd_degree;

        if !is_zero(q[0]) {
            let rval = vtk_habicht_or_sturm_bisection_solve(
                &q,
                deg,
                a,
                upper_bnds,
                tol,
                interval_type,
                0,
                method,
            );
            if zero_root {
                upper_bnds[rval as usize] = 0.0;
                return rval + 1;
            }
            return rval;
        }
    }

    let perturbation =
        ((bounds[0].abs() * 1.0e-12).max(bounds[1].abs() * 1.0e-12)).max(0.5 * tol / f64::from(d));

    let mut var_sgn = [
        vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, bounds[0], None),
        vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, bounds[1], None),
    ];

    for k in 0..=1 {
        if is_zero(evaluate_horner(&sss, d, bounds[k])) {
            let mut left_var_sgn = var_sgn[k];
            let mut right_var_sgn = var_sgn[k];
            let mut leftx = bounds[k];
            let mut rightx = bounds[k];
            while is_zero(evaluate_horner(&sss, d, leftx))
                || is_zero(evaluate_horner(&sss, d, rightx))
                || left_var_sgn <= right_var_sgn
                || ((left_var_sgn == var_sgn[k] || right_var_sgn == var_sgn[k])
                    && left_var_sgn - right_var_sgn != 1)
            {
                leftx -= perturbation;
                rightx += perturbation;
                left_var_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, leftx, None);
                right_var_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, rightx, None);
            }

            if ((interval_type & 2) == 0 && k == 1) || ((interval_type & 1) != 0 && k == 0) {
                bounds[k] = leftx;
                var_sgn[k] = left_var_sgn;
            } else {
                bounds[k] = rightx;
                var_sgn[k] = right_var_sgn;
            }
        }
    }

    let n_roots = var_sgn[0] - var_sgn[1];
    if n_roots < 1 {
        upper_bnds[0] = 0.0;
        if zero_root {
            return 1;
        }
        return 0;
    }

    let mut lower_bnds = vec![bounds[0]; n_roots as usize];
    for upper in upper_bnds.iter_mut().take(n_roots as usize) {
        *upper = bounds[1];
    }

    let mut nloc = n_roots - 1;
    while nloc >= 1 {
        let idx = nloc as usize;
        if upper_bnds[idx] - lower_bnds[idx] <= tol
            || ((nloc < 1 || upper_bnds[idx - 1] < lower_bnds[idx] - tol)
                && (nloc >= n_roots - 1 || upper_bnds[idx] < lower_bnds[idx + 1] - tol))
        {
            nloc -= 1;
            continue;
        }

        let mut leftx = (upper_bnds[idx] + lower_bnds[idx]) / 2.0;
        if leftx >= upper_bnds[idx] || leftx <= lower_bnds[idx] {
            nloc -= 1;
            continue;
        }

        let mut rightx = leftx;
        let mut hit_root = false;
        let mut temp_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, rightx, None);
        let mut left_var_sgn = temp_sgn;
        let mut right_var_sgn = temp_sgn;

        if is_zero(leftx)
            || is_zero(evaluate_horner(&sss, d, leftx))
            || temp_sgn > var_sgn[0]
            || temp_sgn < nloc
        {
            let mut step = 2;
            let mut pos = 1.0;
            let mut p2 = 4.0;
            let mut mid = upper_bnds[idx] / p2 + (p2 - pos) * lower_bnds[idx] / p2;
            let mut found = false;
            left_var_sgn =
                vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, lower_bnds[idx], None);
            right_var_sgn =
                vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, upper_bnds[idx], None);
            temp_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, mid, None);

            while step < 10
                && (temp_sgn > left_var_sgn
                    || temp_sgn < right_var_sgn
                    || is_zero(evaluate_horner(&sss, d, mid))
                    || is_zero(mid))
            {
                pos += 2.0;
                if pos > p2 {
                    pos = 1.0;
                    step += 1;
                    p2 *= 2.0;
                }
                mid = pos * upper_bnds[idx] / p2 + (p2 - pos) * lower_bnds[idx] / p2;
                temp_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, mid, None);
            }

            if step < 10 {
                found = true;
                leftx = mid;
                rightx = mid;
                left_var_sgn = temp_sgn;
                right_var_sgn = temp_sgn;
                if var_sgn[0] - left_var_sgn <= nloc {
                    lower_bnds[idx] = leftx;
                }
                if var_sgn[0] - right_var_sgn > nloc {
                    upper_bnds[idx] = rightx;
                }
            }

            hit_root = !found;
            while !found
                && (is_zero(evaluate_horner(&sss, d, leftx))
                    || is_zero(evaluate_horner(&sss, d, rightx))
                    || left_var_sgn <= right_var_sgn
                    || left_var_sgn > var_sgn[0]
                    || right_var_sgn < var_sgn[1])
            {
                leftx -= perturbation;
                rightx += perturbation;
                if rightx - leftx > 2.0 * tol {
                    break;
                }
                left_var_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, leftx, None);
                right_var_sgn = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, rightx, None);
            }

            if rightx - leftx > 2.0 * tol {
                left_var_sgn = left_var_sgn.min(var_sgn[0]);
                right_var_sgn = right_var_sgn.max(var_sgn[1]);
                if right_var_sgn > var_sgn[0] {
                    right_var_sgn = var_sgn[0] - nloc + 1;
                }
                if left_var_sgn < var_sgn[1] {
                    left_var_sgn = var_sgn[0] - nloc;
                }
                rightx += tol;
                leftx -= tol;
            }

            if hit_root {
                lower_bnds[idx] = mid;
                upper_bnds[idx] = mid;
            }
        } else {
            if var_sgn[0] - left_var_sgn <= nloc {
                lower_bnds[idx] = leftx;
            }
            if var_sgn[0] - right_var_sgn > nloc {
                upper_bnds[idx] = rightx;
            }
        }

        if rightx != leftx {
            let start = var_sgn[0] - left_var_sgn;
            let end = var_sgn[0] - right_var_sgn - 1;
            for i in start..=end {
                if i >= 0 {
                    let i = i as usize;
                    if i > 0 && lower_bnds[i - 1] < leftx {
                        lower_bnds[i] = leftx;
                    }
                    upper_bnds[i] = upper_bnds[i].min(rightx);
                }
            }
        }

        let mut i = var_sgn[0] - right_var_sgn;
        while i >= 0 && i < n_roots {
            let iu = i as usize;
            if lower_bnds[iu] < rightx && upper_bnds[iu] > rightx {
                lower_bnds[iu] = rightx;
            }
            i += 1;
        }

        let mut i = 0;
        while i < var_sgn[0] - left_var_sgn && var_sgn[0] - left_var_sgn <= nloc {
            let iu = i as usize;
            if upper_bnds[iu] > leftx && lower_bnds[iu] < leftx {
                upper_bnds[iu] = leftx;
            }
            i += 1;
        }

        if left_var_sgn - right_var_sgn == 1 || hit_root {
            nloc -= 1;
        }
    }

    let mut n_intervals = n_roots;
    let mut bisection = vec![false; n_roots as usize];

    for nloc in 0..n_roots {
        let idx = nloc as usize;
        if upper_bnds[idx] - lower_bnds[idx] < tol {
            continue;
        }

        let mut zv = evaluate_horner(p, d, upper_bnds[idx]);
        let lv = evaluate_horner(p, d, lower_bnds[idx]);
        let mut z;

        if is_zero(zv) {
            lower_bnds[idx] = upper_bnds[idx];
            continue;
        }

        if is_zero(lv) {
            upper_bnds[idx] = lower_bnds[idx];
            continue;
        }

        let mut us = if zv > 0.0 { 1 } else { -1 };
        let mut ls = if lv > 0.0 { 1 } else { -1 };
        let mut bisect = false;

        if us * ls > 0 {
            let mut zs = 0;
            while upper_bnds[idx] - lower_bnds[idx] > tol {
                z = (upper_bnds[idx] + lower_bnds[idx]) / 2.0;
                if z >= upper_bnds[idx] || z <= lower_bnds[idx] {
                    break;
                }
                let zc = vtk_get_sign_changes(&sss, &degrees, &offsets, n_sss, z, Some(&mut zs));

                if zs == 0 {
                    upper_bnds[idx] = z;
                    lower_bnds[idx] = z;
                    break;
                }

                if var_sgn[0] - zc == nloc + 1 {
                    us = zs;
                    upper_bnds[idx] = z;
                } else {
                    ls = zs;
                    lower_bnds[idx] = z;
                }

                if us * ls < 0 {
                    bisect = true;
                    break;
                }
            }

            bisection[idx] = false;
            if !bisect {
                continue;
            }
        } else {
            bisect = true;
        }

        if bisect {
            let mut tempu = zv;
            while upper_bnds[idx] - lower_bnds[idx] > tol {
                z = (upper_bnds[idx] + lower_bnds[idx]) / 2.0;
                if z >= upper_bnds[idx] || z <= lower_bnds[idx] {
                    break;
                }
                zv = evaluate_horner(p, d, z);
                if is_zero(zv) {
                    upper_bnds[idx] = z;
                    lower_bnds[idx] = z;
                    break;
                }

                if zv * tempu > 0.0 {
                    tempu = zv;
                    upper_bnds[idx] = z;
                } else {
                    lower_bnds[idx] = z;
                }
            }
            bisection[idx] = true;
        }
    }

    upper_bnds[..n_intervals as usize].sort_by(polynomial_solvers_univariate_compare_roots);
    lower_bnds[..n_intervals as usize].sort_by(polynomial_solvers_univariate_compare_roots);

    let mut j = 1;
    while j < n_intervals {
        let ju = j as usize;
        if upper_bnds[ju] < upper_bnds[ju - 1] + 2.0 * tol
            || lower_bnds[ju] < lower_bnds[ju - 1] + 2.0 * tol
            || (zero_root && upper_bnds[ju].abs() < 2.0 * tol)
        {
            for k in ju + 1..n_intervals as usize {
                upper_bnds[k - 1] = upper_bnds[k];
                lower_bnds[k - 1] = lower_bnds[k];
            }
            j -= 1;
            n_intervals -= 1;
        }
        j += 1;
    }

    if zero_root && upper_bnds[0].abs() < 2.0 * tol {
        for k in 1..n_intervals as usize {
            upper_bnds[k - 1] = upper_bnds[k];
        }
    }

    if zero_root {
        upper_bnds[n_intervals as usize] = 0.0;
        n_intervals += 1;
    }

    n_intervals
}

fn vtk_get_derivative_sequence(dp: &mut [f64], p: i32) {
    let mut offset_a = 0;
    let mut offset_b = p + 1;

    for i in 1..=p {
        for j in 0..=p - i {
            dp[(offset_b + j) as usize] =
                f64::from(p - i - j + 1) * dp[(offset_a + j) as usize] / f64::from(i);
        }

        offset_a = offset_b;
        offset_b += p - i + 1;
    }
}

fn vtk_get_sign_changes_for_derivative_sequence(dp: &[f64], count: i32, val: f64) -> i32 {
    let mut old_val = 0;
    let mut changes = 0;
    let mut offset = 0;

    for i in 0..=count {
        let v = evaluate_horner(&dp[offset as usize..], count - i, val);

        if v * f64::from(old_val) < 0.0 {
            changes += 1;
            old_val = -old_val;
        }
        if old_val == 0 {
            if v < 0.0 {
                old_val = -1;
            } else {
                old_val = 1;
            }
        }
        offset += count - i + 1;
    }

    changes
}
