use crate::common::core::{Object, VtkMTimeType};

const VTK_INT_MAX: u32 = i32::MAX as u32;

/// VTK: `vtkPerlinNoise`.
#[derive(Debug, Clone, PartialEq)]
pub struct PerlinNoise {
    object: Object,
    frequency: [f64; 3],
    phase: [f64; 3],
    amplitude: f64,
}

impl PerlinNoise {
    /// VTK: `vtkPerlinNoise::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPerlinNoise"),
            frequency: [1.0, 1.0, 1.0],
            phase: [0.0, 0.0, 0.0],
            amplitude: 1.0,
        }
    }

    /// VTK: `vtkPerlinNoise::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let xd = [
            x[0] * self.frequency[0] - self.phase[0] * 2.0,
            x[1] * self.frequency[1] - self.phase[1] * 2.0,
            x[2] * self.frequency[2] - self.phase[2] * 2.0,
        ];
        let noise = perlin_noise(xd);
        noise[3] * self.amplitude
    }

    /// VTK: `vtkPerlinNoise::EvaluateGradient`.
    pub fn evaluate_gradient(&self, _x: [f64; 3]) -> [f64; 3] {
        [0.0, 0.0, 0.0]
    }

    /// VTK: `vtkPerlinNoise::SetFrequency`.
    pub fn set_frequency(&mut self, x: f64, y: f64, z: f64) {
        let frequency = [x, y, z];
        if self.frequency != frequency {
            self.frequency = frequency;
            self.modified();
        }
    }

    /// VTK: `vtkPerlinNoise::SetFrequency`.
    pub fn set_frequency_array(&mut self, frequency: [f64; 3]) {
        self.set_frequency(frequency[0], frequency[1], frequency[2]);
    }

    /// VTK: `vtkPerlinNoise::GetFrequency`.
    pub fn get_frequency(&self) -> [f64; 3] {
        self.frequency
    }

    /// VTK: `vtkPerlinNoise::SetPhase`.
    pub fn set_phase(&mut self, x: f64, y: f64, z: f64) {
        let phase = [x, y, z];
        if self.phase != phase {
            self.phase = phase;
            self.modified();
        }
    }

    /// VTK: `vtkPerlinNoise::SetPhase`.
    pub fn set_phase_array(&mut self, phase: [f64; 3]) {
        self.set_phase(phase[0], phase[1], phase[2]);
    }

    /// VTK: `vtkPerlinNoise::GetPhase`.
    pub fn get_phase(&self) -> [f64; 3] {
        self.phase
    }

    /// VTK: `vtkPerlinNoise::SetAmplitude`.
    pub fn set_amplitude(&mut self, amplitude: f64) {
        if self.amplitude != amplitude {
            self.amplitude = amplitude;
            self.modified();
        }
    }

    /// VTK: `vtkPerlinNoise::GetAmplitude`.
    pub fn get_amplitude(&self) -> f64 {
        self.amplitude
    }

    /// VTK: `vtkPerlinNoise::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Amplitude: {}\nFrequency: ({}, {}, {})\nPhase: ({}, {}, {})\n",
            self.amplitude,
            self.frequency[0],
            self.frequency[1],
            self.frequency[2],
            self.phase[0],
            self.phase[1],
            self.phase[2]
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self::new()
    }
}

fn hermite(p0: f64, p1: f64, r0: f64, r1: f64, t: f64) -> f64 {
    let tt = t * t;
    p0 * ((2.0 * t - 3.0) * tt + 1.0)
        + p1 * (-2.0 * t + 3.0) * tt
        + r0 * ((t - 2.0) * t + 1.0) * t
        + r1 * (t - 1.0) * tt
}

fn frand(s: i32) -> f64 {
    let mut s = s as u32;
    s = (s << 13) ^ s;
    s = s
        .wrapping_mul(s.wrapping_mul(s).wrapping_mul(15_731).wrapping_add(789_221))
        .wrapping_add(1_376_312_589)
        & VTK_INT_MAX;

    1.0 - f64::from(s) / f64::from(VTK_INT_MAX / 2 + 1)
}

fn rand3abcd(x: i32, y: i32, z: i32) -> [f64; 4] {
    [
        frand(seed3(67, x, 59, y, 71, z)),
        frand(seed3(73, x, 79, y, 83, z)),
        frand(seed3(89, x, 97, y, 101, z)),
        frand(seed3(103, x, 107, y, 109, z)),
    ]
}

fn seed3(ax: i32, x: i32, ay: i32, y: i32, az: i32, z: i32) -> i32 {
    ax.wrapping_mul(x)
        .wrapping_add(ay.wrapping_mul(y))
        .wrapping_add(az.wrapping_mul(z))
}

fn interpolate(i: i32, n: i32, xlim: [[i32; 2]; 3], xarg: [f64; 3]) -> [f64; 4] {
    if n == 0 {
        return rand3abcd(
            xlim[0][(i & 1) as usize],
            xlim[1][((i >> 1) & 1) as usize],
            xlim[2][(i >> 2) as usize],
        );
    }

    let n = n - 1;
    debug_assert!((0..=2).contains(&n));
    let f0 = interpolate(i, n, xlim, xarg);
    let f1 = interpolate(i | (1 << n), n, xlim, xarg);
    let n = n as usize;

    [
        (1.0 - xarg[n]) * f0[0] + xarg[n] * f1[0],
        (1.0 - xarg[n]) * f0[1] + xarg[n] * f1[1],
        (1.0 - xarg[n]) * f0[2] + xarg[n] * f1[2],
        hermite(f0[3], f1[3], f0[n], f1[n], xarg[n]),
    ]
}

fn perlin_noise(x: [f64; 3]) -> [f64; 4] {
    let mut xlim = [[0; 2]; 3];
    xlim[0][0] = x[0].floor() as i32;
    xlim[1][0] = x[1].floor() as i32;
    xlim[2][0] = x[2].floor() as i32;

    xlim[0][1] = xlim[0][0].wrapping_add(1);
    xlim[1][1] = xlim[1][0].wrapping_add(1);
    xlim[2][1] = xlim[2][0].wrapping_add(1);

    let xarg = [
        x[0] - f64::from(xlim[0][0]),
        x[1] - f64::from(xlim[1][0]),
        x[2] - f64::from(xlim[2][0]),
    ];

    interpolate(0, 3, xlim, xarg)
}
