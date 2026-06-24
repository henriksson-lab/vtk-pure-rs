#!/bin/bash
# Generate image filter modules from a simple spec.
# Usage: ./scripts/gen_image_filter.sh name "description" "operation"
# Example: ./scripts/gen_image_filter.sh sqrt "Square root of pixel values" "buf[0].sqrt()"

set -euo pipefail

NAME=${1:-}
DESC=${2:-}
OP=${3:-}
DIR="src/filters/image"
MOD_FILE="$DIR/mod.rs"

if [ -z "$NAME" ] || [ -z "$DESC" ] || [ -z "$OP" ]; then
    echo "Usage: $0 <name> <description> <operation_on_buf[0]>"
    exit 1
fi

if ! [[ "$NAME" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
    echo "Invalid Rust module/function name: $NAME" >&2
    exit 1
fi

if [ ! -d "$DIR" ]; then
    echo "Missing directory: $DIR" >&2
    exit 1
fi

if [ ! -f "$MOD_FILE" ]; then
    echo "Missing module file: $MOD_FILE" >&2
    exit 1
fi

register_module() {
    local name=$1
    local mod_line="pub mod ${name};"
    if ! grep -Fxq "$mod_line" "$MOD_FILE"; then
        printf '%s\n' "$mod_line" >> "$MOD_FILE"
        echo "Registered $name in $MOD_FILE"
    fi
}

if [ -e "$DIR/${NAME}.rs" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "$DIR/${NAME}.rs already exists; set FORCE=1 to overwrite" >&2
    exit 1
fi

cat > "$DIR/${NAME}.rs" << ENDOFFILE
//! ${DESC}

use crate::data::{AnyDataArray, DataArray, ImageData};

/// ${DESC}
pub fn ${NAME}(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n).map(|i| {
        arr.tuple_as_f64(i, &mut buf);
        ${OP}
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
    fn test_${NAME}() {
        let img = ImageData::from_function([5,5,1],[1.0,1.0,1.0],[0.0,0.0,0.0],"v",|x,_,_|x+1.0);
        let r = ${NAME}(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
ENDOFFILE

register_module "$NAME"
echo "Created $DIR/${NAME}.rs"
