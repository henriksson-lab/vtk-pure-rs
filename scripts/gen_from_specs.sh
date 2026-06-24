#!/bin/bash
# Universal module generator from spec files.
#
# SPEC FILE FORMAT:
# Lines starting with # are comments
# Empty lines are ignored
#
# For pointwise image filters (use :: as delimiter to avoid | conflicts):
#   img <name> :: <description> :: <rust_expression_using_buf[0]>
#
# For mesh filter files (inline Rust):
#   mesh <name>
#   <full rust source code>
#   ---
#
# For source files (inline Rust):
#   src <name>
#   <full rust source code>
#   ---
#
# Usage:
#   bash scripts/gen_from_specs.sh specs/round42.txt
#   # Then add matching pub mod entries and run cargo test ...

set -euo pipefail
SPECFILE="${1:?Usage: $0 <specfile>}"
IMG="src/filters/image"
MESH="src/filters/mesh"
SRC="src/filters/core/sources"
COUNT=0
SKIP=0

[ -r "$SPECFILE" ] || { echo "ERROR cannot read spec file: $SPECFILE" >&2; exit 1; }

validate_name() {
    [[ "$1" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]
}

trim() {
    local value=$1
    value=${value#"${value%%[![:space:]]*}"}
    value=${value%"${value##*[![:space:]]}"}
    printf '%s' "$value"
}

register_module() {
    local dir="$1" name="$2"
    local mod_file="${dir}/mod.rs"
    local mod_line="pub mod ${name};"
    [ -f "$mod_file" ] || { echo "ERROR missing module file: $mod_file" >&2; return 1; }
    if ! grep -Fxq "$mod_line" "$mod_file"; then
        printf '%s\n' "$mod_line" >> "$mod_file"
    fi
}

gen_img() {
    local name="$1" desc="$2" op="$3"
    validate_name "$name" || { echo "ERROR invalid Rust module/function name: $name" >&2; return 1; }
    [ -d "$IMG" ] || { echo "ERROR missing directory: $IMG" >&2; return 1; }
    [ -f "$IMG/mod.rs" ] || { echo "ERROR missing module file: $IMG/mod.rs" >&2; return 1; }
    [ -f "$IMG/${name}.rs" ] && { SKIP=$((SKIP+1)); return; }
    cat > "$IMG/${name}.rs" << IEOF
//! ${desc}
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn ${name}(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) { Some(a) if a.num_components()==1=>a, _=>return input.clone() };
    let n = arr.num_tuples(); let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n).map(|i|{arr.tuple_as_f64(i,&mut buf);${op}}).collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0],dims[1],dims[2]).with_spacing(input.spacing()).with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars,data,1)))
}
#[cfg(test)] mod tests { use super::*;
    #[test] fn test() { let img=ImageData::from_function([5,5,1],[1.0,1.0,1.0],[0.0,0.0,0.0],"v",|x,_,_|x+1.0);
        let r=${name}(&img,"v"); assert_eq!(r.dimensions(),[5,5,1]); } }
IEOF
    register_module "$IMG" "$name"
    COUNT=$((COUNT+1))
}

gen_block() {
    local dir="$1" name="$2" content="$3"
    validate_name "$name" || { echo "ERROR invalid Rust module name: $name" >&2; return 1; }
    [ -d "$dir" ] || { echo "ERROR missing directory: $dir" >&2; return 1; }
    [ -f "$dir/mod.rs" ] || { echo "ERROR missing module file: $dir/mod.rs" >&2; return 1; }
    [ -f "${dir}/${name}.rs" ] && { SKIP=$((SKIP+1)); return; }
    printf '%s' "$content" > "${dir}/${name}.rs"
    register_module "$dir" "$name"
    COUNT=$((COUNT+1))
}

MODE=""
BLOCK_NAME=""
BLOCK_DIR=""
BLOCK_CONTENT=""

while IFS= read -r line || [ -n "$line" ]; do
    # Skip comments and empty lines (unless inside a block)
    if [ -z "$MODE" ]; then
        [[ "$line" =~ ^#.*$ ]] && continue
        [[ -z "$line" ]] && continue
    fi

    if [ "$MODE" = "block" ]; then
        if [ "$line" = "---" ]; then
            gen_block "$BLOCK_DIR" "$BLOCK_NAME" "$BLOCK_CONTENT"
            MODE=""
            BLOCK_CONTENT=""
        else
            BLOCK_CONTENT="${BLOCK_CONTENT}${line}
"
        fi
        continue
    fi

    # Parse line type
    if [[ "$line" =~ ^img\ (.+)::(.+)::(.+)$ ]]; then
        local_name=$(trim "${BASH_REMATCH[1]}")
        local_desc=$(trim "${BASH_REMATCH[2]}")
        local_op=$(trim "${BASH_REMATCH[3]}")
        gen_img "$local_name" "$local_desc" "$local_op"
    elif [[ "$line" =~ ^mesh\ (.+)$ ]]; then
        BLOCK_NAME=$(trim "${BASH_REMATCH[1]}")
        BLOCK_DIR="$MESH"
        MODE="block"
        BLOCK_CONTENT=""
    elif [[ "$line" =~ ^src\ (.+)$ ]]; then
        BLOCK_NAME=$(trim "${BASH_REMATCH[1]}")
        BLOCK_DIR="$SRC"
        MODE="block"
        BLOCK_CONTENT=""
    else
        echo "ERROR unsupported spec line: $line" >&2
        exit 1
    fi
done < "$SPECFILE"

if [ -n "$MODE" ]; then
    echo "ERROR unterminated block for: $BLOCK_NAME" >&2
    exit 1
fi

echo "Generated $COUNT new modules ($SKIP skipped)"
echo "Registered generated modules in target mod.rs files. Run: cargo test --features filters-image,filters-mesh --lib"
