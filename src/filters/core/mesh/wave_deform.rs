use crate::data::PolyData;

/// Deform mesh vertices with a sine wave pattern.
///
/// Each vertex is displaced along `wave_direction` by an amount proportional to
/// `amplitude * sin(2*pi * dot(position, axis) / wavelength + phase)`.
///
/// `axis` determines the propagation direction of the wave (projects vertex
/// position onto this axis to compute the wave phase). `wave_direction` is the
/// direction of displacement. Both vectors are normalized internally.
pub fn wave_deform(
    input: &PolyData,
    axis: [f64; 3],
    wave_direction: [f64; 3],
    amplitude: f64,
    wavelength: f64,
    phase: f64,
) -> PolyData {
    let mut output: PolyData = input.clone();
    let n: usize = input.points.len();

    if n == 0 || wavelength.abs() < 1e-15 {
        return output;
    }

    // Normalize axis
    let axis_len: f64 = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if axis_len < 1e-15 {
        return output;
    }
    let ax: [f64; 3] = [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len];

    // Normalize wave direction
    let wd_len: f64 = (wave_direction[0] * wave_direction[0]
        + wave_direction[1] * wave_direction[1]
        + wave_direction[2] * wave_direction[2])
        .sqrt();
    if wd_len < 1e-15 {
        return output;
    }
    let wd: [f64; 3] = [
        wave_direction[0] / wd_len,
        wave_direction[1] / wd_len,
        wave_direction[2] / wd_len,
    ];

    let two_pi: f64 = 2.0 * std::f64::consts::PI;

    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        let proj: f64 = p[0] * ax[0] + p[1] * ax[1] + p[2] * ax[2];
        let displacement: f64 = amplitude * (two_pi * proj / wavelength + phase).sin();
        output.points.set(
            i,
            [
                p[0] + wd[0] * displacement,
                p[1] + wd[1] * displacement,
                p[2] + wd[2] * displacement,
            ],
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_line_mesh() -> PolyData {
        let mut pd = PolyData::new();
        // 11 points along the X axis from 0 to 10
        for i in 0..11 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd
    }

    #[test]
    fn wave_displaces_along_direction() {
        let pd = make_line_mesh();
        let result = wave_deform(
            &pd,
            [1.0, 0.0, 0.0], // wave propagates along X
            [0.0, 1.0, 0.0], // displacement along Y
            1.0,             // amplitude
            10.0,            // wavelength
            0.0,             // phase
        );

        // At x=0, sin(0) = 0 => y should be 0
        let p0: [f64; 3] = result.points.get(0);
        assert!(
            (p0[1]).abs() < 1e-10,
            "y at x=0 should be ~0, got {}",
            p0[1]
        );

        // At x=2.5, sin(pi/2) = 1 => y should be ~1
        let p25: [f64; 3] = result.points.get(2); // x=2 is close but not exact
                                                  // x=2.5 would be sin(2*pi*2.5/10) = sin(pi/2) = 1
                                                  // x=2 gives sin(2*pi*2/10) = sin(2*pi/5) ~ 0.951
        assert!(p25[1] > 0.9, "y at x=2 should be > 0.9, got {}", p25[1]);

        // X and Z coordinates should be unchanged
        for i in 0..11 {
            let p: [f64; 3] = result.points.get(i);
            assert!((p[0] - i as f64).abs() < 1e-10, "x should be unchanged");
            assert!((p[2]).abs() < 1e-10, "z should be unchanged");
        }
    }

    #[test]
    fn zero_amplitude_is_identity() {
        let pd = make_line_mesh();
        let result = wave_deform(&pd, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0, 5.0, 0.0);
        for i in 0..pd.points.len() {
            let orig: [f64; 3] = pd.points.get(i);
            let deformed: [f64; 3] = result.points.get(i);
            assert!((orig[0] - deformed[0]).abs() < 1e-15);
            assert!((orig[1] - deformed[1]).abs() < 1e-15);
            assert!((orig[2] - deformed[2]).abs() < 1e-15);
        }
    }

    #[test]
    fn phase_shifts_the_wave() {
        let pd = make_line_mesh();
        let half_pi: f64 = std::f64::consts::PI / 2.0;

        let r1 = wave_deform(&pd, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 10.0, 0.0);
        let r2 = wave_deform(&pd, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 10.0, half_pi);

        // At x=0: r1 gives sin(0)=0, r2 gives sin(pi/2)=1
        let p1: [f64; 3] = r1.points.get(0);
        let p2: [f64; 3] = r2.points.get(0);
        assert!((p1[2]).abs() < 1e-10, "r1 at x=0 z should be ~0");
        assert!((p2[2] - 1.0).abs() < 1e-10, "r2 at x=0 z should be ~1");
    }
}
