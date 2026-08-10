/// Helper class for structured extent arithmetic.
///
/// VTK origin: `VTK/Common/DataModel/vtkStructuredExtent.h` and
/// `VTK/Common/DataModel/vtkStructuredExtent.cxx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuredExtent;

impl StructuredExtent {
    /// VTK: `vtkStructuredExtent::New`.
    pub fn new() -> Self {
        Self
    }

    /// VTK: `vtkStructuredExtent::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkStructuredExtent::Clamp`.
    pub fn clamp(ext: &mut [i32; 6], whole_ext: &[i32; 6]) {
        ext[0] = ext[0].max(whole_ext[0]);
        ext[1] = ext[1].min(whole_ext[1]);
        ext[2] = ext[2].max(whole_ext[2]);
        ext[3] = ext[3].min(whole_ext[3]);
        ext[4] = ext[4].max(whole_ext[4]);
        ext[5] = ext[5].min(whole_ext[5]);
    }

    /// VTK: `vtkStructuredExtent::StrictlySmaller`.
    pub fn strictly_smaller(ext: &[i32; 6], whole_ext: &[i32; 6]) -> bool {
        Self::smaller(ext, whole_ext)
            && (ext[0] > whole_ext[0]
                || ext[1] < whole_ext[1]
                || ext[2] > whole_ext[2]
                || ext[3] < whole_ext[3]
                || ext[4] > whole_ext[4]
                || ext[5] < whole_ext[5])
    }

    /// VTK: `vtkStructuredExtent::Smaller`.
    pub fn smaller(ext: &[i32; 6], whole_ext: &[i32; 6]) -> bool {
        !(ext[0] < whole_ext[0]
            || ext[0] > whole_ext[1]
            || ext[1] < whole_ext[0]
            || ext[1] > whole_ext[1]
            || ext[2] < whole_ext[2]
            || ext[2] > whole_ext[3]
            || ext[3] < whole_ext[2]
            || ext[3] > whole_ext[3]
            || ext[4] < whole_ext[4]
            || ext[4] > whole_ext[5]
            || ext[5] < whole_ext[4]
            || ext[5] > whole_ext[5])
    }

    /// VTK: `vtkStructuredExtent::Grow(int ext[6], int count)`.
    pub fn grow(ext: &mut [i32; 6], count: i32) {
        ext[0] -= count;
        ext[2] -= count;
        ext[4] -= count;
        ext[1] += count;
        ext[3] += count;
        ext[5] += count;
    }

    /// VTK: `vtkStructuredExtent::Grow(int ext[6], int count, int wholeExt[6])`.
    pub fn grow_with_whole_extent(ext: &mut [i32; 6], count: i32, whole_ext: &[i32; 6]) {
        Self::grow(ext, count);
        Self::clamp(ext, whole_ext);
    }

    /// VTK: `vtkStructuredExtent::Transform`.
    pub fn transform(ext: &mut [i32; 6], whole_ext: &[i32; 6]) {
        ext[0] -= whole_ext[0];
        ext[1] -= whole_ext[0];
        ext[2] -= whole_ext[2];
        ext[3] -= whole_ext[2];
        ext[4] -= whole_ext[4];
        ext[5] -= whole_ext[4];
    }

    /// VTK: `vtkStructuredExtent::GetDimensions`.
    pub fn get_dimensions(ext: [i32; 6]) -> [i32; 3] {
        [
            ext[1] - ext[0] + 1,
            ext[3] - ext[2] + 1,
            ext[5] - ext[4] + 1,
        ]
    }
}
