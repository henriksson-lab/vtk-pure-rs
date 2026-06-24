//! Swiss army pocket knife (closed, with handle and blade outline).
use crate::data::{CellArray, Points, PolyData};

pub fn pocket_knife(length: f64, width: f64, thickness: f64) -> PolyData {
    let hl = length / 2.0;
    let hw = width / 2.0;
    let ht = thickness / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Handle (rounded rectangle body)
    let bb = pts.len();
    pts.push([-hl, -hw, -ht]);
    pts.push([hl, -hw, -ht]);
    pts.push([hl, hw, -ht]);
    pts.push([-hl, hw, -ht]);
    pts.push([-hl, -hw, ht]);
    pts.push([hl, -hw, ht]);
    pts.push([hl, hw, ht]);
    pts.push([-hl, hw, ht]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 5) as i64, (bb + 4) as i64]);
    polys.push_cell(&[
        (bb + 1) as i64,
        (bb + 2) as i64,
        (bb + 6) as i64,
        (bb + 5) as i64,
    ]);
    polys.push_cell(&[
        (bb + 2) as i64,
        (bb + 3) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
    ]);
    polys.push_cell(&[(bb + 3) as i64, bb as i64, (bb + 4) as i64, (bb + 7) as i64]);
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 7) as i64,
    ]); // top
    polys.push_cell(&[bb as i64, (bb + 3) as i64, (bb + 2) as i64, (bb + 1) as i64]); // bottom
                                                                                      // Blade (folded, peeking out from one end)
    let blade_len = length * 0.9;
    let b0 = pts.len();
    pts.push([hl, 0.0, ht + 0.001]);
    let b1 = pts.len();
    pts.push([hl + blade_len, 0.0, ht + 0.001]);
    let b2 = pts.len();
    pts.push([hl + blade_len, -hw * 0.3, ht + 0.001]);
    polys.push_cell(&[b0 as i64, b1 as i64, b2 as i64]);
    // Cross emblem
    let emblem_s = width * 0.15;
    let ec = [0.0, -hw - 0.001, 0.0];
    let e0 = pts.len();
    pts.push([ec[0] - emblem_s, ec[1], ec[2] - emblem_s * 0.3]);
    let e1 = pts.len();
    pts.push([ec[0] + emblem_s, ec[1], ec[2] - emblem_s * 0.3]);
    let e2 = pts.len();
    pts.push([ec[0] + emblem_s, ec[1], ec[2] + emblem_s * 0.3]);
    let e3 = pts.len();
    pts.push([ec[0] - emblem_s, ec[1], ec[2] + emblem_s * 0.3]);
    lines.push_cell(&[e0 as i64, e1 as i64]);
    lines.push_cell(&[e1 as i64, e2 as i64]);
    lines.push_cell(&[e2 as i64, e3 as i64]);
    lines.push_cell(&[e3 as i64, e0 as i64]);
    // Vertical cross line
    let cv0 = pts.len();
    pts.push([ec[0], ec[1], ec[2] - emblem_s]);
    let cv1 = pts.len();
    pts.push([ec[0], ec[1], ec[2] + emblem_s]);
    lines.push_cell(&[cv0 as i64, cv1 as i64]);
    let ch0 = pts.len();
    pts.push([ec[0] - emblem_s * 0.7, ec[1], ec[2]]);
    let ch1 = pts.len();
    pts.push([ec[0] + emblem_s * 0.7, ec[1], ec[2]]);
    lines.push_cell(&[ch0 as i64, ch1 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_knife() {
        let m = pocket_knife(8.0, 2.0, 1.0);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() >= 7);
    }
}
