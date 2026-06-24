#!/bin/bash
# Generate multiple simple image filters at once.
# Each line: name|description|operation
# Then add matching pub mod entries in src/filters/image/mod.rs and run:
# cargo test --features filters-image --lib

set -euo pipefail

DIR="src/filters/image"
MOD_FILE="$DIR/mod.rs"

validate_name() {
    [[ "$1" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]
}

register_module() {
    local name=$1
    local mod_line="pub mod ${name};"
    [ -f "$MOD_FILE" ] || { echo "ERROR missing module file: $MOD_FILE" >&2; return 1; }
    if ! grep -Fxq "$mod_line" "$MOD_FILE"; then
        printf '%s\n' "$mod_line" >> "$MOD_FILE"
    fi
}

gen() {
    local name=$1 desc=$2 op=$3
    validate_name "$name" || { echo "ERROR invalid Rust module/function name: $name" >&2; return 1; }
    [ -d "$DIR" ] || { echo "ERROR missing directory: $DIR" >&2; return 1; }
    [ -f "$MOD_FILE" ] || { echo "ERROR missing module file: $MOD_FILE" >&2; return 1; }
    [ -f "$DIR/${name}.rs" ] && { echo "SKIP $name (exists)"; return; }
    cat > "$DIR/${name}.rs" << EOF
//! ${desc}

use crate::data::{AnyDataArray, DataArray, ImageData};

/// ${desc}
pub fn ${name}(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n).map(|i| {
        arr.tuple_as_f64(i, &mut buf);
        ${op}
    }).collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing()).with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_${name}() {
        let img = ImageData::from_function([5,5,1],[1.0,1.0,1.0],[0.0,0.0,0.0],"v",|x,_,_|x+1.0);
        let r = ${name}(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
EOF
    register_module "$name"
    echo "  + $name"
}

echo "Generating image filters..."
gen "image_sqrt" "Square root of pixel values" "buf[0].max(0.0).sqrt()"
gen "image_square" "Square of pixel values" "buf[0] * buf[0]"
gen "image_reciprocal" "Reciprocal (1/x) of pixel values" "if buf[0].abs() > 1e-30 { 1.0 / buf[0] } else { 0.0 }"
gen "image_sin" "Sine of pixel values" "buf[0].sin()"
gen "image_cos" "Cosine of pixel values" "buf[0].cos()"
gen "image_exp_map" "Exponential of pixel values" "buf[0].exp().min(1e30)"
gen "image_ln" "Natural logarithm of pixel values" "(buf[0].abs().max(1e-30)).ln()"
gen "image_floor" "Floor of pixel values" "buf[0].floor()"
gen "image_ceil" "Ceiling of pixel values" "buf[0].ceil()"
gen "image_round" "Round pixel values to nearest integer" "buf[0].round()"
gen "image_sign" "Sign of pixel values (-1, 0, or 1)" "buf[0].signum()"
gen "image_step" "Step function (0 if negative, 1 if non-negative)" "if buf[0] >= 0.0 { 1.0 } else { 0.0 }"

echo "Done. Registered generated modules in $MOD_FILE. Run: cargo test --features filters-image --lib"
