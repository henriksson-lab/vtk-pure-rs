use std::f64::consts::PI;

use crate::common::core::object::Object;

pub type ScalarNumber = f64;

/// VTK: `vtkFFT::ComplexNumber`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct ComplexNumber {
    pub r: ScalarNumber,
    pub i: ScalarNumber,
}

impl ComplexNumber {
    fn scale(self, rhs: ScalarNumber) -> Self {
        Self {
            r: self.r * rhs,
            i: self.i * rhs,
        }
    }
}

impl std::ops::Add for ComplexNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            i: self.i + rhs.i,
        }
    }
}

impl std::ops::Sub for ComplexNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r - rhs.r,
            i: self.i - rhs.i,
        }
    }
}

impl std::ops::Mul for ComplexNumber {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r - self.i * rhs.i,
            i: self.r * rhs.i + self.i * rhs.r,
        }
    }
}

impl std::ops::Mul<ScalarNumber> for ComplexNumber {
    type Output = Self;

    fn mul(self, rhs: ScalarNumber) -> Self::Output {
        self.scale(rhs)
    }
}

impl std::ops::Div for ComplexNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let divisor = rhs.r * rhs.r + rhs.i * rhs.i;
        Self {
            r: (self.r * rhs.r + self.i * rhs.i) / divisor,
            i: (self.i * rhs.r - self.r * rhs.i) / divisor,
        }
    }
}

impl std::ops::Div<ScalarNumber> for ComplexNumber {
    type Output = Self;

    fn div(self, rhs: ScalarNumber) -> Self::Output {
        Self {
            r: self.r / rhs,
            i: self.i / rhs,
        }
    }
}

pub trait FftValue: Copy {
    fn zero() -> Self;
    fn to_complex(self) -> ComplexNumber;
}

impl FftValue for ScalarNumber {
    fn zero() -> Self {
        0.0
    }

    fn to_complex(self) -> ComplexNumber {
        ComplexNumber { r: self, i: 0.0 }
    }
}

impl FftValue for ComplexNumber {
    fn zero() -> Self {
        ComplexNumber { r: 0.0, i: 0.0 }
    }

    fn to_complex(self) -> ComplexNumber {
        self
    }
}

/// VTK: `vtkFFT::Octave`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Octave {
    Hz31_5 = 5,
    Hz63 = 6,
    Hz125 = 7,
    Hz250 = 8,
    Hz500 = 9,
    KHz1 = 10,
    KHz2 = 11,
    KHz4 = 12,
    KHz8 = 13,
    KHz16 = 14,
}

/// VTK: `vtkFFT::OctaveSubdivision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OctaveSubdivision {
    Full,
    FirstHalf,
    SecondHalf,
    FirstThird,
    SecondThird,
    ThirdThird,
}

/// VTK: `vtkFFT::Scaling`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Scaling {
    Density = 0,
    Spectrum = 1,
}

/// VTK: `vtkFFT::SpectralMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SpectralMode {
    Stft = 0,
    Psd = 1,
}

pub type WindowGenerator = fn(usize, usize) -> ScalarNumber;

/// VTK: `vtkFFT`.
#[derive(Debug, Clone)]
pub struct Fft {
    object: Object,
}

impl Fft {
    /// VTK: `vtkFFT::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkFFT"),
        }
    }

    /// VTK: `vtkFFT::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.get_class_name().to_string()
    }

    /// VTK: `vtkFFT::Fft`.
    pub fn fft<T: FftValue>(input: &[T]) -> Vec<ComplexNumber> {
        if input.len() <= 1 {
            return Vec::new();
        }
        dft(
            input.iter().map(|value| value.to_complex()).collect(),
            false,
        )
    }

    /// VTK: `vtkFFT::Fft`.
    pub fn fft_into<T: FftValue>(input: &[T], result: &mut [ComplexNumber]) {
        if input.len() <= 1 {
            return;
        }
        let fft = Self::fft(input);
        let count = fft.len().min(result.len());
        result[..count].copy_from_slice(&fft[..count]);
    }

    /// VTK: `vtkFFT::RFft`.
    pub fn rfft(input: &[ScalarNumber]) -> Vec<ComplexNumber> {
        if input.len() <= 1 {
            return Vec::new();
        }
        let out_size = input.len() / 2 + 1;
        let fft = Self::fft(input);
        fft[..out_size].to_vec()
    }

    /// VTK: `vtkFFT::RFft`.
    pub fn rfft_into(input: &[ScalarNumber], result: &mut [ComplexNumber]) {
        if input.len() <= 1 {
            return;
        }
        let rfft = Self::rfft(input);
        let count = rfft.len().min(result.len());
        result[..count].copy_from_slice(&rfft[..count]);
    }

    /// VTK: `vtkFFT::RFft`.
    pub fn rfft_complex_into(
        _input: &[ComplexNumber],
        _result: &mut [ComplexNumber],
    ) -> Result<(), &'static str> {
        Err("vtkFFT::RFft does not accept complex numbers as its input.")
    }

    /// VTK: `vtkFFT::IFft`.
    pub fn ifft(input: &[ComplexNumber]) -> Vec<ComplexNumber> {
        if input.is_empty() {
            return Vec::new();
        }
        dft(input.to_vec(), true)
            .into_iter()
            .map(|value| value / input.len() as ScalarNumber)
            .collect()
    }

    /// VTK: `vtkFFT::IRFft`.
    pub fn irfft(input: &[ComplexNumber]) -> Vec<ScalarNumber> {
        if input.len() < 2 {
            return Vec::new();
        }
        let out_size = (input.len() - 1) * 2;
        let mut two_sided = vec![ComplexNumber { r: 0.0, i: 0.0 }; out_size];
        two_sided[..input.len()].copy_from_slice(input);
        for i in 1..(input.len() - 1) {
            two_sided[out_size - i] = Self::conjugate(input[i]);
        }
        Self::ifft(&two_sided)
            .into_iter()
            .map(|value| value.r)
            .collect()
    }

    /// VTK: `vtkFFT::Abs`.
    pub fn abs(input: ComplexNumber) -> ScalarNumber {
        (input.r * input.r + input.i * input.i).sqrt()
    }

    /// VTK: `vtkFFT::SquaredAbs`.
    pub fn squared_abs(input: ComplexNumber) -> ScalarNumber {
        input.r * input.r + input.i * input.i
    }

    /// VTK: `vtkFFT::Conjugate`.
    pub fn conjugate(input: ComplexNumber) -> ComplexNumber {
        ComplexNumber {
            r: input.r,
            i: -input.i,
        }
    }

    /// VTK: `vtkFFT::FftFreq`.
    pub fn fft_freq(window_length: i32, sample_spacing: f64) -> Vec<ScalarNumber> {
        if window_length < 1 {
            return Vec::new();
        }

        let window_length = window_length as usize;
        let freq = 1.0 / (window_length as f64 * sample_spacing);
        let nshan = window_length / 2 + 1;
        let mut result = vec![0.0; window_length];
        for i in 1..nshan {
            let val = i as ScalarNumber * freq;
            result[i] = val;
            result[window_length - i] = -val;
        }
        result
    }

    /// VTK: `vtkFFT::RFftFreq`.
    pub fn rfft_freq(window_length: i32, sample_spacing: f64) -> Vec<ScalarNumber> {
        if window_length < 1 {
            return Vec::new();
        }

        let val = 1.0 / (window_length as f64 * sample_spacing);
        let size = window_length / 2 + 1;
        (0..size).map(|i| i as ScalarNumber * val).collect()
    }

    /// VTK: `vtkFFT::GetOctaveFrequencyRange`.
    pub fn get_octave_frequency_range(
        octave: Octave,
        octave_subdivision: OctaveSubdivision,
        base_two: bool,
    ) -> [f64; 2] {
        let mut band_number = octave as i32 * 3;
        let factor = match octave_subdivision {
            OctaveSubdivision::FirstThird
            | OctaveSubdivision::SecondThird
            | OctaveSubdivision::ThirdThird => {
                if octave_subdivision == OctaveSubdivision::FirstThird {
                    band_number -= 1;
                }
                if octave_subdivision == OctaveSubdivision::ThirdThird {
                    band_number += 1;
                }
                if base_two {
                    2.0_f64.powf(1.0 / 6.0)
                } else {
                    10.0_f64.powf(0.05)
                }
            }
            _ => {
                if base_two {
                    2.0_f64.sqrt()
                } else {
                    10.0_f64.powf(0.15)
                }
            }
        };

        let midband_frequency = if base_two {
            1000.0 * 2.0_f64.powf((band_number - 30) as f64 / 3.0)
        } else {
            10.0_f64.powf(band_number as f64 / 10.0)
        };
        let mut lower_frequency = midband_frequency / factor;
        let mut upper_frequency = midband_frequency * factor;

        if octave_subdivision == OctaveSubdivision::FirstHalf {
            upper_frequency = midband_frequency;
        } else if octave_subdivision == OctaveSubdivision::SecondHalf {
            lower_frequency = midband_frequency;
        }

        [lower_frequency, upper_frequency]
    }

    /// VTK: `vtkFFT::OverlappingFft`.
    pub fn overlapping_fft<T, TW>(
        signal: &[T],
        window: &[TW],
        noverlap: usize,
        detrend: bool,
        mut onesided: bool,
        shape: Option<&mut [u32; 2]>,
    ) -> Vec<ComplexNumber>
    where
        T: FftValue
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<ScalarNumber, Output = T>
            + std::ops::Mul<TW, Output = T>,
        TW: Copy,
    {
        if window.is_empty() || noverlap >= window.len() || signal.len() <= noverlap {
            if let Some(shape) = shape {
                *shape = [0, 0];
            }
            return Vec::new();
        }

        onesided = onesided && std::mem::size_of::<T>() == std::mem::size_of::<ScalarNumber>();
        let segment_offset = window.len() - noverlap;
        let nsegment = (signal.len() - noverlap) / segment_offset;
        let nfft = if onesided {
            window.len() / 2 + 1
        } else {
            window.len()
        };
        if let Some(shape) = shape {
            *shape = [nsegment as u32, nfft as u32];
        }

        let mut result = vec![ComplexNumber { r: 0.0, i: 0.0 }; nsegment * nfft];
        for i in 0..nsegment {
            let offset = i * segment_offset;
            Self::preprocess_and_dispatch_fft(
                &signal[offset..offset + window.len()],
                window,
                detrend,
                onesided,
                &mut result[i * nfft..(i + 1) * nfft],
            );
        }
        result
    }

    /// VTK: `vtkFFT::Spectrogram`.
    pub fn spectrogram<T, TW>(
        signal: &[T],
        window: &[TW],
        sample_rate: f64,
        mut noverlap: i32,
        detrend: bool,
        onesided: bool,
        scaling: Scaling,
        mode: SpectralMode,
        shape: Option<&mut [u32; 2]>,
        transpose: bool,
    ) -> Vec<ComplexNumber>
    where
        T: FftValue
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<ScalarNumber, Output = T>
            + std::ops::Mul<TW, Output = T>,
        TW: Copy + Into<ScalarNumber>,
    {
        if signal.len() <= 1 || window.len() <= 1 || window.len() > signal.len() {
            if let Some(shape) = shape {
                *shape = [0, 0];
            }
            return Vec::new();
        }

        if noverlap < 0 || noverlap >= window.len() as i32 {
            noverlap = window.len() as i32 / 2;
        }

        let mut local_shape = [0, 0];
        let shape_ref = shape.unwrap_or(&mut local_shape);
        let mut result = Self::overlapping_fft(
            signal,
            window,
            noverlap as usize,
            detrend,
            onesided,
            Some(shape_ref),
        );
        Self::scale_fft(
            &mut result,
            shape_ref,
            window,
            sample_rate,
            onesided,
            scaling,
            mode,
        );
        if transpose {
            Self::transpose(&mut result, shape_ref);
        }
        result
    }

    /// VTK: `vtkFFT::Csd`.
    pub fn csd<T, TW>(
        signal: &[T],
        window: &[TW],
        sample_rate: f64,
        noverlap: i32,
        detrend: bool,
        onesided: bool,
        scaling: Scaling,
    ) -> Vec<ScalarNumber>
    where
        T: FftValue
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<ScalarNumber, Output = T>
            + std::ops::Mul<TW, Output = T>,
        TW: Copy + Into<ScalarNumber>,
    {
        if signal.len() <= 1 || window.len() <= 1 || window.len() > signal.len() {
            return Vec::new();
        }

        let mut shape = [0, 0];
        let result = Self::spectrogram(
            signal,
            window,
            sample_rate,
            noverlap,
            detrend,
            onesided,
            scaling,
            SpectralMode::Psd,
            Some(&mut shape),
            false,
        );
        let mean_factor = 1.0 / shape[0] as ScalarNumber;
        let mut average = vec![0.0; shape[1] as usize];
        for i in 0..shape[0] as usize {
            for j in 0..shape[1] as usize {
                average[j] += Self::abs(result[i * shape[1] as usize + j]) * mean_factor;
            }
        }
        average
    }

    /// VTK: `vtkFFT::Transpose`.
    pub fn transpose<T: Copy>(data: &mut [T], shape: &mut [u32; 2]) {
        let size = (shape[0] * shape[1]) as usize;
        if size <= 1 || data.len() < size {
            shape.swap(0, 1);
            return;
        }
        let mn1 = size - 1;
        let mut visited = vec![false; size];
        for cycle in 0..size {
            if visited[cycle] {
                continue;
            }
            let mut current = cycle;
            loop {
                current = if current == mn1 {
                    mn1
                } else {
                    (shape[0] as usize * current) % mn1
                };
                data.swap(current, cycle);
                visited[current] = true;
                if current == cycle {
                    break;
                }
            }
        }
        shape.swap(0, 1);
    }

    /// VTK: `vtkFFT::HanningGenerator`.
    pub fn hanning_generator(x: usize, size: usize) -> ScalarNumber {
        0.5 * (1.0 - (2.0 * PI * x as f64 / (size - 1) as f64).cos())
    }

    /// VTK: `vtkFFT::BartlettGenerator`.
    pub fn bartlett_generator(x: usize, size: usize) -> ScalarNumber {
        2.0 * x as f64 / (size - 1) as f64
    }

    /// VTK: `vtkFFT::SineGenerator`.
    pub fn sine_generator(x: usize, size: usize) -> ScalarNumber {
        (PI * x as f64 / (size - 1) as f64).sin()
    }

    /// VTK: `vtkFFT::BlackmanGenerator`.
    pub fn blackman_generator(x: usize, size: usize) -> ScalarNumber {
        let cosin = (2.0 * PI * x as f64 / (size - 1) as f64).cos();
        0.42 - 0.5 * cosin + 0.08 * (2.0 * cosin * cosin - 1.0)
    }

    /// VTK: `vtkFFT::RectangularGenerator`.
    pub fn rectangular_generator(_x: usize, _size: usize) -> ScalarNumber {
        1.0
    }

    /// VTK: `vtkFFT::GenerateKernel1D`.
    pub fn generate_kernel_1d<T>(kernel: &mut [T], n: usize, generator: WindowGenerator)
    where
        T: From<ScalarNumber> + Copy,
    {
        let half = n / 2 + n % 2;
        for i in 0..half {
            let value = T::from(generator(i, n));
            kernel[i] = value;
            kernel[n - 1 - i] = value;
        }
    }

    /// VTK: `vtkFFT::GenerateKernel2D`.
    pub fn generate_kernel_2d<T>(kernel: &mut [T], n: usize, m: usize, generator: WindowGenerator)
    where
        T: From<ScalarNumber> + Copy,
    {
        let half_x = n / 2 + n % 2;
        let half_y = m / 2 + m % 2;
        for i in 0..half_x {
            for j in 0..half_y {
                let value = T::from(generator(i, n) * generator(j, m));
                kernel[i * m + j] = value;
                kernel[(n - 1 - i) * m + j] = value;
                kernel[i * m + (m - 1 - j)] = value;
                kernel[(n - 1 - i) * m + (m - 1 - j)] = value;
            }
        }
    }

    /// VTK: `vtkFFT::ComputeScaling`.
    pub fn compute_scaling<T>(window: &[T], scaling: Scaling, fs: f64) -> ScalarNumber
    where
        T: Copy + Into<ScalarNumber>,
    {
        if scaling == Scaling::Density {
            let sum_squares: ScalarNumber = window
                .iter()
                .map(|value| {
                    let value = (*value).into();
                    value * value
                })
                .sum();
            1.0 / (fs * sum_squares)
        } else {
            let sum: ScalarNumber = window.iter().map(|value| (*value).into()).sum();
            1.0 / sum.powi(2)
        }
    }

    /// VTK: `vtkFFT::PreprocessAndDispatchFft`.
    pub fn preprocess_and_dispatch_fft<T, TW>(
        signal: &[T],
        window: &[TW],
        detrend: bool,
        onesided: bool,
        result: &mut [ComplexNumber],
    ) where
        T: FftValue
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<ScalarNumber, Output = T>
            + std::ops::Mul<TW, Output = T>,
        TW: Copy,
    {
        let mean = if detrend {
            signal
                .iter()
                .copied()
                .fold(T::zero(), |sum, value| sum + value)
                / window.len() as ScalarNumber
        } else {
            T::zero()
        };
        let segment: Vec<T> = signal
            .iter()
            .copied()
            .zip(window.iter().copied())
            .map(|(value, window)| (value - mean) * window)
            .collect();
        if onesided {
            let scalar_segment: Vec<ScalarNumber> =
                segment.iter().map(|value| value.to_complex().r).collect();
            Self::rfft_into(&scalar_segment, result);
        } else {
            Self::fft_into(&segment, result);
        }
    }

    /// VTK: `vtkFFT::ScaleFft`.
    pub fn scale_fft<T>(
        fft: &mut [ComplexNumber],
        shape: &[u32; 2],
        window: &[T],
        sample_rate: f64,
        onesided: bool,
        scaling: Scaling,
        mode: SpectralMode,
    ) where
        T: Copy + Into<ScalarNumber>,
    {
        let mut scale = Self::compute_scaling(window, scaling, sample_rate);
        if mode == SpectralMode::Stft {
            scale = scale.sqrt();
        } else if mode == SpectralMode::Psd && onesided {
            scale *= 2.0;
        }
        let total_size = (shape[0] * shape[1]) as usize;
        if mode == SpectralMode::Psd {
            for value in fft.iter_mut().take(total_size) {
                *value = Self::conjugate(*value) * *value * scale;
            }
            if onesided {
                for i in 0..shape[0] as usize {
                    let idx = i * shape[1] as usize;
                    fft[idx] = fft[idx] * 0.5;
                    if window.len() % 2 == 0 {
                        let nyquist = idx + shape[1] as usize - 1;
                        fft[nyquist] = fft[nyquist] * 0.5;
                    }
                }
            }
        } else if mode == SpectralMode::Stft {
            for value in fft.iter_mut().take(total_size) {
                *value = *value * scale;
            }
        }
    }
}

fn dft(input: Vec<ComplexNumber>, inverse: bool) -> Vec<ComplexNumber> {
    let n = input.len();
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut result = vec![ComplexNumber { r: 0.0, i: 0.0 }; n];
    for (k, output) in result.iter_mut().enumerate() {
        let mut sum = ComplexNumber { r: 0.0, i: 0.0 };
        for (sample, value) in input.iter().enumerate() {
            let angle = sign * 2.0 * PI * k as f64 * sample as f64 / n as f64;
            let twiddle = ComplexNumber {
                r: angle.cos(),
                i: angle.sin(),
            };
            sum = sum + *value * twiddle;
        }
        *output = sum;
    }
    result
}
