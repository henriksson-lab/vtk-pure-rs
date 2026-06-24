//! Shipping container (ISO standard proportions).
use crate::data::{CellArray, Points, PolyData};
pub fn shipping_container(length: f64, width: f64, height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([length, 0.0, 0.0]);
    pts.push([length, width, 0.0]);
    pts.push([0.0, width, 0.0]);
    pts.push([0.0, 0.0, height]);
    pts.push([length, 0.0, height]);
    pts.push([length, width, height]);
    pts.push([0.0, width, height]);
    let faces = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64, f[3] as i64]);
    }
    // Corrugation lines on sides
    let mut lines = CellArray::new();
    let corr_n = 10;
    for i in 0..corr_n {
        let t = (i + 1) as f64 / (corr_n + 1) as f64;
        let x = t * length;
        let b = pts.len();
        pts.push([x, 0.0, height * 0.1]);
        pts.push([x, 0.0, height * 0.9]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        let b2 = pts.len();
        pts.push([x, width, height * 0.1]);
        pts.push([x, width, height * 0.9]);
        lines.push_cell(&[b2 as i64, (b2 + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
pub fn iso_20ft_container() -> PolyData {
    shipping_container(6.058, 2.438, 2.591)
}
pub fn iso_40ft_container() -> PolyData {
    shipping_container(12.192, 2.438, 2.591)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = shipping_container(6.0, 2.4, 2.6);
        assert_eq!(c.polys.num_cells(), 6);
    }
    #[test]
    fn test_20ft() {
        let c = iso_20ft_container();
        assert_eq!(c.polys.num_cells(), 6);
    }
    #[test]
    fn test_40ft() {
        let c = iso_40ft_container();
        assert_eq!(c.polys.num_cells(), 6);
    }
}
