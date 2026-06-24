use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the strain tensor between two mesh configurations.
///
/// Given original and deformed mesh (same topology), computes per-vertex
/// displacement magnitude and adds "Displacement" and "StrainMagnitude".
pub fn displacement_field(original: &PolyData, deformed: &PolyData) -> PolyData {
    let n = original.points.len();
    if n == 0 || n != deformed.points.len() {
        return original.clone();
    }

    let mut disp = Vec::with_capacity(n * 3);
    let mut mag = Vec::with_capacity(n);

    for i in 0..n {
        let a = original.points.get(i);
        let b = deformed.points.get(i);
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        disp.push(dx);
        disp.push(dy);
        disp.push(dz);
        mag.push((dx * dx + dy * dy + dz * dz).sqrt());
    }

    let mut pd = deformed.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Displacement",
            disp,
            3,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DisplacementMagnitude",
            mag,
            1,
        )));
    pd
}

/// Compute per-cell Green-Lagrange strain approximation.
///
/// Computes the average edge stretch ratio per cell as a simple strain metric.
/// Adds "Strain" cell data.
pub fn cell_strain(original: &PolyData, deformed: &PolyData) -> PolyData {
    let no = original.polys.num_cells();
    let nd = deformed.polys.num_cells();
    if no != nd || no == 0 {
        return deformed.clone();
    }

    let mut strains = Vec::new();
    let mut oi = original.polys.iter();
    let mut di = deformed.polys.iter();

    loop {
        let oc = oi.next();
        let dc = di.next();
        match (oc, dc) {
            (Some(o), Some(d)) if o.len() >= 3 && d.len() >= 3 => {
                let mut total_stretch = 0.0;
                let mut count = 0;
                for k in 0..o.len() {
                    let oa = original.points.get(o[k] as usize);
                    let ob = original.points.get(o[(k + 1) % o.len()] as usize);
                    let da = deformed.points.get(d[k] as usize);
                    let db = deformed.points.get(d[(k + 1) % d.len()] as usize);
                    let lo = ((oa[0] - ob[0]).powi(2)
                        + (oa[1] - ob[1]).powi(2)
                        + (oa[2] - ob[2]).powi(2))
                    .sqrt();
                    let ld = ((da[0] - db[0]).powi(2)
                        + (da[1] - db[1]).powi(2)
                        + (da[2] - db[2]).powi(2))
                    .sqrt();
                    if lo > 1e-15 {
                        total_stretch += (ld / lo - 1.0).abs();
                    }
                    count += 1;
                }
                strains.push(if count > 0 {
                    total_stretch / count as f64
                } else {
                    0.0
                });
            }
            (Some(_), Some(_)) => strains.push(0.0),
            _ => break,
        }
    }

    let mut pd = deformed.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Strain", strains, 1)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displacement_basic() {
        let mut orig = PolyData::new();
        orig.points.push([0.0, 0.0, 0.0]);
        orig.points.push([1.0, 0.0, 0.0]);
        let mut def = PolyData::new();
        def.points.push([0.0, 0.0, 1.0]);
        def.points.push([1.0, 0.0, 1.0]);

        let result = displacement_field(&orig, &def);
        let arr = result
            .point_data()
            .get_array("DisplacementMagnitude")
            .unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn strain_stretch() {
        let mut orig = PolyData::new();
        orig.points.push([0.0, 0.0, 0.0]);
        orig.points.push([1.0, 0.0, 0.0]);
        orig.points.push([0.5, 1.0, 0.0]);
        orig.polys.push_cell(&[0, 1, 2]);
        let mut def = PolyData::new();
        def.points.push([0.0, 0.0, 0.0]);
        def.points.push([2.0, 0.0, 0.0]);
        def.points.push([1.0, 2.0, 0.0]); // 2x scaled
        def.polys.push_cell(&[0, 1, 2]);

        let result = cell_strain(&orig, &def);
        let arr = result.cell_data().get_array("Strain").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.5); // significant strain from 2x scaling
    }

    #[test]
    fn no_deformation() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cell_strain(&pd, &pd);
        let arr = result.cell_data().get_array("Strain").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = displacement_field(&pd, &pd);
        assert_eq!(result.points.len(), 0);
    }
}
