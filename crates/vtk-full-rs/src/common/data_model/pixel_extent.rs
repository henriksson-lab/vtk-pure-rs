use std::{
    cmp::{max, min},
    fmt,
    ops::{BitAndAssign, BitOrAssign, Index, IndexMut},
};

/// VTK: `vtkPixelExtent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelExtent {
    data: [i32; 4],
}

impl PixelExtent {
    /// VTK: `vtkPixelExtent::vtkPixelExtent()`.
    pub fn new() -> Self {
        let mut extent = Self { data: [0; 4] };
        extent.clear();
        extent
    }

    /// VTK: `vtkPixelExtent::vtkPixelExtent(const T*)`.
    pub fn new_with_data(data: [i32; 4]) -> Self {
        Self { data }
    }

    /// VTK: `vtkPixelExtent::vtkPixelExtent(T ilo, T ihi, T jlo, T jhi)`.
    pub fn new_with_bounds(ilo: i32, ihi: i32, jlo: i32, jhi: i32) -> Self {
        Self::new_with_data([ilo, ihi, jlo, jhi])
    }

    /// VTK: `vtkPixelExtent::vtkPixelExtent(T width, T height)`.
    pub fn new_with_width_height(width: i32, height: i32) -> Self {
        Self::new_with_bounds(0, width - 1, 0, height - 1)
    }

    /// VTK: `vtkPixelExtent::SetData`.
    pub fn set_data(&mut self, data: [i32; 4]) {
        self.data = data;
    }

    /// VTK: `vtkPixelExtent::SetData`.
    pub fn set_data_bounds(&mut self, ilo: i32, ihi: i32, jlo: i32, jhi: i32) {
        self.set_data([ilo, ihi, jlo, jhi]);
    }

    /// VTK: `vtkPixelExtent::SetData(const vtkPixelExtent&)`.
    pub fn set_data_from_extent(&mut self, other: &Self) {
        self.set_data(other.data);
    }

    /// VTK: `vtkPixelExtent::Clear`.
    pub fn clear(&mut self) {
        self.data = [i32::MAX, i32::MIN, i32::MAX, i32::MIN];
    }

    /// VTK: `vtkPixelExtent::GetData`.
    pub fn get_data(&self) -> [i32; 4] {
        self.data
    }

    /// VTK: `vtkPixelExtent::GetDataU`.
    pub fn get_data_u(&self) -> [u32; 4] {
        self.data
            .map(|value| u32::from_ne_bytes(value.to_ne_bytes()))
    }

    /// VTK: `vtkPixelExtent::GetStartIndex`.
    pub fn get_start_index(&self) -> [i32; 2] {
        [self.data[0], self.data[2]]
    }

    /// VTK: `vtkPixelExtent::GetStartIndex(int first[2], const int origin[2])`.
    pub fn get_start_index_with_origin(&self, origin: [i32; 2]) -> [i32; 2] {
        [self.data[0] - origin[0], self.data[2] - origin[1]]
    }

    /// VTK: `vtkPixelExtent::GetEndIndex`.
    pub fn get_end_index(&self) -> [i32; 2] {
        [self.data[1], self.data[3]]
    }

    /// VTK: `vtkPixelExtent::Empty`.
    pub fn empty(&self) -> i32 {
        if self.data[0] > self.data[1] || self.data[2] > self.data[3] {
            1
        } else {
            0
        }
    }

    /// VTK: `vtkPixelExtent::Contains(const vtkPixelExtent&)`.
    pub fn contains_extent(&self, other: &Self) -> i32 {
        if self.data[0] <= other.data[0]
            && self.data[1] >= other.data[1]
            && self.data[2] <= other.data[2]
            && self.data[3] >= other.data[3]
        {
            1
        } else {
            0
        }
    }

    /// VTK: `vtkPixelExtent::Contains(int i, int j)`.
    pub fn contains(&self, i: i32, j: i32) -> i32 {
        if self.data[0] <= i && self.data[1] >= i && self.data[2] <= j && self.data[3] >= j {
            1
        } else {
            0
        }
    }

    /// VTK: `vtkPixelExtent::Disjoint`.
    pub fn disjoint(&self, other: Self) -> i32 {
        let mut intersection = other;
        intersection &= *self;
        intersection.empty()
    }

    /// VTK: `vtkPixelExtent::Size(T nCells[2])`.
    pub fn size(&self) -> [i32; 2] {
        [
            self.data[1] - self.data[0] + 1,
            self.data[3] - self.data[2] + 1,
        ]
    }

    /// VTK: `vtkPixelExtent::Size()`.
    pub fn number_of_pixels(&self) -> usize {
        Self::size_of_extent(self)
    }

    /// VTK: `vtkPixelExtent::Grow(int n)`.
    pub fn grow(&mut self, n: i32) {
        self.data[0] -= n;
        self.data[1] += n;
        self.data[2] -= n;
        self.data[3] += n;
    }

    /// VTK: `vtkPixelExtent::Grow(int q, int n)`.
    pub fn grow_direction(&mut self, q: usize, n: i32) {
        let q = 2 * q;
        self.data[q] -= n;
        self.data[q + 1] += n;
    }

    /// VTK: `vtkPixelExtent::GrowLow`.
    pub fn grow_low(&mut self, q: usize, n: i32) {
        self.data[2 * q] -= n;
    }

    /// VTK: `vtkPixelExtent::GrowHigh`.
    pub fn grow_high(&mut self, q: usize, n: i32) {
        self.data[2 * q + 1] += n;
    }

    /// VTK: `vtkPixelExtent::Shrink(int n)`.
    pub fn shrink(&mut self, n: i32) {
        self.data[0] += n;
        self.data[1] -= n;
        self.data[2] += n;
        self.data[3] -= n;
    }

    /// VTK: `vtkPixelExtent::Shrink(int q, int n)`.
    pub fn shrink_direction(&mut self, q: usize, n: i32) {
        let q = 2 * q;
        self.data[q] += n;
        self.data[q + 1] -= n;
    }

    /// VTK: `vtkPixelExtent::Shift()`.
    pub fn shift_to_origin(&mut self) {
        for q in 0..2 {
            let qq = q * 2;
            let n = -self.data[qq];
            self.data[qq] += n;
            self.data[qq + 1] += n;
        }
    }

    /// VTK: `vtkPixelExtent::Shift(const vtkPixelExtent&)`.
    pub fn shift_by_extent(&mut self, other: &Self) {
        for q in 0..2 {
            let qq = q * 2;
            let n = -other.data[qq];
            self.data[qq] += n;
            self.data[qq + 1] += n;
        }
    }

    /// VTK: `vtkPixelExtent::Shift(int*)`.
    pub fn shift_by(&mut self, n: [i32; 2]) {
        self.data[0] += n[0];
        self.data[1] += n[0];
        self.data[2] += n[1];
        self.data[3] += n[1];
    }

    /// VTK: `vtkPixelExtent::Shift(int q, int n)`.
    pub fn shift_direction(&mut self, q: usize, n: i32) {
        let q = 2 * q;
        self.data[q] += n;
        self.data[q + 1] += n;
    }

    /// VTK: `vtkPixelExtent::Split(int dir)`.
    pub fn split_direction(&mut self, dir: usize) -> Self {
        let mut half = Self::new();
        let q = 2 * dir;
        let len = self.data[q + 1] - self.data[q] + 1;
        let mut split = len / 2;
        if split != 0 {
            split += self.data[q];
            half = *self;
            half.data[q] = split;
            self.data[q + 1] = split - 1;
        }
        half
    }

    /// VTK: `vtkPixelExtent::CellToNode`.
    pub fn cell_to_node(&mut self) {
        self.data[1] += 1;
        self.data[3] += 1;
    }

    /// VTK: `vtkPixelExtent::NodeToCell`.
    pub fn node_to_cell(&mut self) {
        self.data[1] -= 1;
        self.data[3] -= 1;
    }

    /// VTK: `vtkPixelExtent::Size(const vtkPixelExtent&, T nCells[2])`.
    pub fn extent_size(ext: &Self) -> [i32; 2] {
        ext.size()
    }

    /// VTK: `vtkPixelExtent::Size(const vtkPixelExtent&)`.
    pub fn size_of_extent(ext: &Self) -> usize {
        let size = ext.size();
        (i64::from(size[0]) * i64::from(size[1])) as usize
    }

    /// VTK: `vtkPixelExtent::Grow(const vtkPixelExtent&, int)`.
    pub fn grow_extent(input_ext: &Self, n: i32) -> Self {
        let mut output_ext = *input_ext;
        output_ext.grow_direction(0, n);
        output_ext.grow_direction(1, n);
        output_ext
    }

    /// VTK: `vtkPixelExtent::Grow(const vtkPixelExtent&, const vtkPixelExtent&, int)`.
    pub fn grow_extent_with_domain(input_ext: &Self, problem_domain: &Self, n: i32) -> Self {
        let mut output_ext = Self::grow_extent(input_ext, n);
        output_ext &= *problem_domain;
        output_ext
    }

    /// VTK: `vtkPixelExtent::GrowLow(const vtkPixelExtent&, int, int)`.
    pub fn grow_low_extent(input_ext: &Self, q: usize, n: i32) -> Self {
        let mut output_ext = *input_ext;
        output_ext.data[2 * q] -= n;
        output_ext
    }

    /// VTK: `vtkPixelExtent::GrowHigh(const vtkPixelExtent&, int, int)`.
    pub fn grow_high_extent(input_ext: &Self, q: usize, n: i32) -> Self {
        let mut output_ext = *input_ext;
        output_ext.data[2 * q + 1] += n;
        output_ext
    }

    /// VTK: `vtkPixelExtent::Shrink(const vtkPixelExtent&, int)`.
    pub fn shrink_extent(input_ext: &Self, n: i32) -> Self {
        Self::grow_extent(input_ext, -n)
    }

    /// VTK: `vtkPixelExtent::Shrink(const vtkPixelExtent&, const vtkPixelExtent&, int)`.
    pub fn shrink_extent_with_domain(input_ext: &Self, problem_domain: &Self, n: i32) -> Self {
        let mut output_ext = *input_ext;
        output_ext.grow_direction(0, -n);
        output_ext.grow_direction(1, -n);
        for i in 0..4 {
            if input_ext.data[i] == problem_domain.data[i] {
                output_ext.data[i] = problem_domain.data[i];
            }
        }
        output_ext
    }

    /// VTK: `vtkPixelExtent::NodeToCell(const vtkPixelExtent&)`.
    pub fn node_to_cell_extent(input_ext: &Self) -> Self {
        let mut output_ext = *input_ext;
        output_ext.data[1] -= 1;
        output_ext.data[3] -= 1;
        output_ext
    }

    /// VTK: `vtkPixelExtent::CellToNode(const vtkPixelExtent&)`.
    pub fn cell_to_node_extent(input_ext: &Self) -> Self {
        let mut output_ext = *input_ext;
        output_ext.data[1] += 1;
        output_ext.data[3] += 1;
        output_ext
    }

    /// VTK: `vtkPixelExtent::Shift(int*, int)`.
    pub fn shift_index(ij: &mut [i32; 2], n: i32) {
        ij[0] += n;
        ij[1] += n;
    }

    /// VTK: `vtkPixelExtent::Shift(int*, int*)`.
    pub fn shift_index_by(ij: &mut [i32; 2], n: [i32; 2]) {
        ij[0] += n[0];
        ij[1] += n[1];
    }

    /// VTK: `vtkPixelExtent::Split(int, int, const vtkPixelExtent&, std::deque<vtkPixelExtent>&)`.
    pub fn split(i1: i32, j1: i32, ext: &Self, new_exts: &mut Vec<Self>) {
        let i0 = i1 - 1;
        let j0 = j1 - 1;
        let mut outside = 1;

        if ext.contains(i0, j0) != 0 {
            new_exts.push(Self::new_with_bounds(ext[0], i0, ext[2], j0));
            outside = 0;
        }
        if ext.contains(i1, j0) != 0 {
            new_exts.push(Self::new_with_bounds(i1, ext[1], ext[2], j0));
            outside = 0;
        }
        if ext.contains(i0, j1) != 0 {
            new_exts.push(Self::new_with_bounds(ext[0], i0, j1, ext[3]));
            outside = 0;
        }
        if ext.contains(i1, j1) != 0 {
            new_exts.push(Self::new_with_bounds(i1, ext[1], j1, ext[3]));
            outside = 0;
        }

        if outside != 0 {
            new_exts.push(*ext);
        }
    }

    /// VTK: `vtkPixelExtent::Subtract`.
    pub fn subtract(a: &Self, b: &Self, c: &mut Vec<Self>) {
        let mut intersection = *a;
        intersection &= *b;

        if intersection.empty() != 0 {
            c.push(*a);
            return;
        }
        if b.contains_extent(a) != 0 {
            return;
        }

        intersection.cell_to_node();

        let mut tmp_a0 = vec![*a];
        for q in 0..4 {
            const IDS: [usize; 8] = [0, 2, 1, 2, 1, 3, 0, 3];
            let qq = 2 * q;
            let i = intersection[IDS[qq]];
            let j = intersection[IDS[qq + 1]];
            let mut tmp_a1 = Vec::new();
            while let Some(ext) = tmp_a0.pop() {
                Self::split(i, j, &ext, &mut tmp_a1);
            }
            tmp_a0 = tmp_a1;
        }

        for ext in tmp_a0 {
            if b.contains_extent(&ext) == 0 {
                c.push(ext);
            }
        }
    }

    /// VTK: `vtkPixelExtent::Merge`.
    pub fn merge(exts: &mut Vec<Self>) {
        let mut ne = exts.len();
        let mut tmp_exts = Vec::with_capacity(ne);
        for ext in exts.iter() {
            let mut ext = *ext;
            ext.cell_to_node();
            tmp_exts.push(ext);
        }

        for q in 0..2 {
            let qq = 2 * q;
            for t in 0..ne {
                let mut next_pass = false;
                if tmp_exts[t].empty() != 0 {
                    continue;
                }

                for c in 0..ne {
                    if c == t || tmp_exts[c].empty() != 0 {
                        continue;
                    }

                    if tmp_exts[t][qq] == tmp_exts[c][qq]
                        && tmp_exts[t][qq + 1] == tmp_exts[c][qq + 1]
                    {
                        let mut overlap = tmp_exts[t];
                        overlap &= tmp_exts[c];
                        if overlap.empty() == 0 {
                            let mut merged = tmp_exts[t];
                            merged |= tmp_exts[c];
                            tmp_exts.push(merged);
                            ne += 1;
                            tmp_exts[t].clear();
                            tmp_exts[c].clear();
                            next_pass = true;
                        }
                    }
                    if next_pass {
                        break;
                    }
                }
            }
        }

        exts.clear();
        for mut ext in tmp_exts.into_iter().take(ne) {
            if ext.empty() == 0 {
                ext.node_to_cell();
                exts.push(ext);
            }
        }
    }
}

impl Default for PixelExtent {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for PixelExtent {
    type Output = i32;

    /// VTK: `vtkPixelExtent::operator[]`.
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for PixelExtent {
    /// VTK: `vtkPixelExtent::operator[]`.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl BitAndAssign for PixelExtent {
    /// VTK: `vtkPixelExtent::operator&=`.
    fn bitand_assign(&mut self, other: Self) {
        if self.empty() != 0 {
            return;
        }
        if other.empty() != 0 {
            self.clear();
            return;
        }

        self.data[0] = max(self.data[0], other.data[0]);
        self.data[1] = min(self.data[1], other.data[1]);
        self.data[2] = max(self.data[2], other.data[2]);
        self.data[3] = min(self.data[3], other.data[3]);

        if self.empty() != 0 {
            self.clear();
        }
    }
}

impl BitOrAssign for PixelExtent {
    /// VTK: `vtkPixelExtent::operator|=`.
    fn bitor_assign(&mut self, other: Self) {
        if other.empty() != 0 {
            return;
        }
        if self.empty() != 0 {
            self.set_data(other.data);
            return;
        }

        self.data[0] = min(self.data[0], other.data[0]);
        self.data[1] = max(self.data[1], other.data[1]);
        self.data[2] = min(self.data[2], other.data[2]);
        self.data[3] = max(self.data[3], other.data[3]);
    }
}

impl PartialOrd for PixelExtent {
    /// VTK: `operator<(const vtkPixelExtent&, const vtkPixelExtent&)`.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.number_of_pixels()
            .partial_cmp(&other.number_of_pixels())
    }
}

impl fmt::Display for PixelExtent {
    /// VTK: `operator<<(std::ostream&, const vtkPixelExtent&)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.empty() != 0 {
            write!(f, "(empty)")
        } else {
            write!(
                f,
                "({}, {}, {}, {})",
                self.data[0], self.data[1], self.data[2], self.data[3]
            )
        }
    }
}
