/// VTK: `vtkHashCombiner`.
#[derive(Debug, Clone, Copy, Default)]
pub struct HashCombiner;

impl HashCombiner {
    /// VTK: `vtkHashCombiner::operator()(T& h, std::size_t k)` for 64-bit `T`.
    pub fn combine_u64(&self, h: &mut u64, k: usize) {
        const M: u64 = 0xc6a4a7935bd1e995;
        const R: u32 = 47;

        let mut kk = k as u64;
        kk = kk.wrapping_mul(M);
        kk ^= kk >> R;
        kk = kk.wrapping_mul(M);

        *h ^= kk;
        *h = h.wrapping_mul(M);
        *h = h.wrapping_add(0xe6546b64);
    }

    /// VTK: `vtkHashCombiner::operator()(T& h, std::size_t k)` for 32-bit `T`.
    pub fn combine_u32(&self, h: &mut u32, k: usize) {
        const C1: u32 = 0xcc9e2d51;
        const C2: u32 = 0x1b873593;
        const R1: u32 = 15;
        const R2: u32 = 13;

        let mut kk = k as u32;
        kk = kk.wrapping_mul(C1);
        kk = kk.rotate_left(R1);
        kk = kk.wrapping_mul(C2);

        *h ^= kk;
        *h = h.rotate_left(R2);
        *h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    /// VTK: `vtkHashCombiner::operator()(T& h, std::size_t k)` for `std::size_t` hashes.
    #[cfg(target_pointer_width = "64")]
    pub fn combine_usize(&self, h: &mut usize, k: usize) {
        let mut hash = *h as u64;
        self.combine_u64(&mut hash, k);
        *h = hash as usize;
    }

    /// VTK: `vtkHashCombiner::operator()(T& h, std::size_t k)` for `std::size_t` hashes.
    #[cfg(target_pointer_width = "32")]
    pub fn combine_usize(&self, h: &mut usize, k: usize) {
        let mut hash = *h as u32;
        self.combine_u32(&mut hash, k);
        *h = hash as usize;
    }
}
