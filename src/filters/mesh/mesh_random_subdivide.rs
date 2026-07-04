//! Randomly subdivide selected triangles by inserting centroids.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn random_subdivide(mesh: &PolyData, fraction: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    if mesh.polys.is_empty() {
        return mesh.clone();
    }
    let fraction = fraction.clamp(0.0, 1.0);
    let mut rng = seed;
    let mut next_rand = || -> f64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    };
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        pts.push(mesh.points.get(i).try_into().unwrap());
    }
    let mut polys = CellArray::new();
    let mut inserted_centroids = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            polys.push_cell(cell);
            continue;
        }
        let Ok(a) = usize::try_from(cell[0]) else {
            polys.push_cell(cell);
            continue;
        };
        let Ok(b) = usize::try_from(cell[1]) else {
            polys.push_cell(cell);
            continue;
        };
        let Ok(c) = usize::try_from(cell[2]) else {
            polys.push_cell(cell);
            continue;
        };
        if a >= n || b >= n || c >= n {
            polys.push_cell(cell);
            continue;
        }
        if next_rand() < fraction {
            // Subdivide: insert centroid, create 3 sub-triangles
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let center = pts.len();
            pts.push([
                (pa[0] + pb[0] + pc[0]) / 3.0,
                (pa[1] + pb[1] + pc[1]) / 3.0,
                (pa[2] + pb[2] + pc[2]) / 3.0,
            ]);
            inserted_centroids.push([a, b, c]);
            polys.push_cell(&[a as i64, b as i64, center as i64]);
            polys.push_cell(&[b as i64, c as i64, center as i64]);
            polys.push_cell(&[c as i64, a as i64, center as i64]);
        } else {
            polys.push_cell(&[a as i64, b as i64, c as i64]);
        }
    }
    let mut m = mesh.clone();
    m.points = pts;
    m.polys = polys;
    m.point_data_mut().clear();
    interpolate_point_data(mesh, &mut m, &inserted_centroids);
    m.cell_data_mut().clear();
    m
}

fn interpolate_point_data(
    input: &PolyData,
    output: &mut PolyData,
    inserted_centroids: &[[usize; 3]],
) {
    let n = input.points.len();
    let mut point_data = DataSetAttributes::new();
    for array in input.point_data().iter() {
        if array.num_tuples() != n {
            continue;
        }
        let Some(interpolated) = interpolate_array(array, inserted_centroids) else {
            continue;
        };
        let name = interpolated.name().to_string();
        point_data.add_array(interpolated);
        copy_active_attribute(input.point_data(), &mut point_data, &name);
    }
    *output.point_data_mut() = point_data;
}

fn interpolate_array(
    array: &AnyDataArray,
    inserted_centroids: &[[usize; 3]],
) -> Option<AnyDataArray> {
    macro_rules! interpolate_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            Some(AnyDataArray::$variant(interpolate_typed_array(
                a,
                inserted_centroids,
            )?))
        }};
    }

    match array {
        AnyDataArray::F32(_) => interpolate_variant!(F32),
        AnyDataArray::F64(_) => interpolate_variant!(F64),
        AnyDataArray::I8(_) => interpolate_variant!(I8),
        AnyDataArray::I16(_) => interpolate_variant!(I16),
        AnyDataArray::I32(_) => interpolate_variant!(I32),
        AnyDataArray::I64(_) => interpolate_variant!(I64),
        AnyDataArray::U8(_) => interpolate_variant!(U8),
        AnyDataArray::U16(_) => interpolate_variant!(U16),
        AnyDataArray::U32(_) => interpolate_variant!(U32),
        AnyDataArray::U64(_) => interpolate_variant!(U64),
    }
}

fn interpolate_typed_array<T: Scalar>(
    array: &DataArray<T>,
    inserted_centroids: &[[usize; 3]],
) -> Option<DataArray<T>> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity((array.num_tuples() + inserted_centroids.len()) * nc);
    data.extend_from_slice(array.as_slice());
    let mut tuple = vec![T::default(); nc];
    for &[a, b, c] in inserted_centroids {
        if a >= array.num_tuples() || b >= array.num_tuples() || c >= array.num_tuples() {
            return None;
        }
        let ta = array.tuple(a);
        let tb = array.tuple(b);
        let tc = array.tuple(c);
        for i in 0..nc {
            tuple[i] = T::from_f64((ta[i].to_f64() + tb[i].to_f64() + tc[i].to_f64()) / 3.0);
        }
        data.extend_from_slice(&tuple);
    }
    Some(DataArray::from_vec(array.name(), data, nc))
}

fn copy_active_attribute(source: &DataSetAttributes, target: &mut DataSetAttributes, name: &str) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.normals().map(|a| a.name()) == Some(name) {
        target.set_active_normals(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn test_random_sub() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = random_subdivide(&mesh, 1.0, 42); // subdivide all
        assert_eq!(r.polys.num_cells(), 6); // 2 tris * 3 = 6
        assert_eq!(r.points.len(), 6); // 4 original + 2 centroids
    }

    #[test]
    fn interpolates_point_data_for_inserted_centroids() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "value",
                vec![0.0, 3.0, 6.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("value");

        let r = random_subdivide(&mesh, 1.0, 42);
        let value = r.point_data().scalars().unwrap();
        let mut buf = [0.0];
        value.tuple_as_f64(3, &mut buf);
        assert_eq!(value.num_tuples(), r.points.len());
        assert_eq!(buf[0], 3.0);
    }
}
