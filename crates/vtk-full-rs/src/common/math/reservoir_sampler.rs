use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::core::{AnyArray, VtkIdType, VtkTypeUInt32};

static RANDOM_SEED_COUNTER: AtomicU64 = AtomicU64::new(0);

/// VTK: `vtkReservoirSamplerBase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservoirSamplerBase;

impl ReservoirSamplerBase {
    /// VTK: `vtkReservoirSamplerBase::RandomSeed`.
    pub(crate) fn random_seed() -> VtkTypeUInt32 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        let counter = RANDOM_SEED_COUNTER.fetch_add(1, Ordering::Relaxed);
        (mix64(nanos ^ counter.rotate_left(17)) & u32::MAX as u64) as VtkTypeUInt32
    }
}

/// Rust error equivalent for `vtkReservoirSampler` invalid-argument throws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ReservoirSamplerError {
    #[error("negative sample counts are disallowed")]
    NegativeSampleCount,
    #[error("null arrays are disallowed")]
    NullArray,
    #[error("array size would overflow integer type")]
    ArraySizeOverflow,
}

/// VTK: `vtkReservoirSampler<Integer, Monotonic>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservoirSampler<const MONOTONIC: bool = true>;

impl<const MONOTONIC: bool> ReservoirSampler<MONOTONIC> {
    /// Rust constructor for the stateless VTK sampler template.
    pub fn new() -> Self {
        Self
    }

    /// VTK: `vtkReservoirSampler<Integer, Monotonic>::operator()(Integer kk, Integer nn)`.
    pub fn sample(
        &self,
        kk: VtkIdType,
        nn: VtkIdType,
    ) -> Result<Vec<VtkIdType>, ReservoirSamplerError> {
        self.generate_sample(kk, nn)
    }

    /// VTK: `vtkReservoirSampler<Integer, Monotonic>::operator()(Integer kk, vtkAbstractArray* array)`.
    pub fn sample_array(
        &self,
        kk: VtkIdType,
        array: Option<&AnyArray>,
    ) -> Result<Vec<VtkIdType>, ReservoirSamplerError> {
        let Some(array) = array else {
            return Err(ReservoirSamplerError::NullArray);
        };
        let nn = array.get_number_of_tuples();
        if nn > VtkIdType::MAX {
            return Err(ReservoirSamplerError::ArraySizeOverflow);
        }
        self.generate_sample(kk, nn)
    }

    /// VTK: `vtkReservoirSampler<Integer, Monotonic>::GenerateSample`.
    pub(crate) fn generate_sample(
        &self,
        mut kk: VtkIdType,
        nn: VtkIdType,
    ) -> Result<Vec<VtkIdType>, ReservoirSamplerError> {
        if nn < kk {
            kk = nn;
        }
        if kk < 0 {
            return Err(ReservoirSamplerError::NegativeSampleCount);
        }

        let kk_usize = usize::try_from(kk).expect("sample count must fit usize");
        let mut data = vec![0; kk_usize];
        if kk == 0 {
            return Ok(data);
        }

        let mut ii = 0;
        for value in &mut data {
            *value = ii;
            ii += 1;
        }
        if kk == nn {
            return Ok(data);
        }

        let mut generator = SplitMix64::new(ReservoirSamplerBase::random_seed() as u64);
        let mut w = (generator.unit_uniform().ln() / kk as f64).exp();

        loop {
            let delta = (generator.unit_uniform().ln() / (1.0 - w).ln()).floor() + 1.0;
            if delta < 0.0 || delta > VtkIdType::MAX as f64 {
                break;
            }
            let int_delta = delta as VtkIdType;
            if nn - ii > int_delta {
                let jj = generator.uniform_int(kk);
                ii += int_delta;
                data[jj] = ii;
                w *= (generator.unit_uniform().ln() / kk as f64).exp();
            } else {
                break;
            }
        }

        if MONOTONIC {
            data.sort_unstable();
        }
        Ok(data)
    }
}

impl<const MONOTONIC: bool> Default for ReservoirSampler<MONOTONIC> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix64(self.state)
    }

    fn unit_uniform(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    fn uniform_int(&mut self, upper_exclusive: VtkIdType) -> usize {
        let upper = u64::try_from(upper_exclusive).expect("upper bound must be positive");
        let zone = u64::MAX - u64::MAX % upper;
        loop {
            let value = self.next_u64();
            if value < zone {
                return usize::try_from(value % upper).expect("sample index must fit usize");
            }
        }
    }
}

fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
