#!/bin/bash
# Fast batch module generator. Reads specs from stdin or file.
# Format per line:
#   gi NAME :: DESCRIPTION :: OPERATION
#
# Usage: bash scripts/gen_round.sh specs.txt
# Or:    bash scripts/gen_round.sh < specs.txt
# Or:    echo "gi image_foo :: Desc :: buf[0]" | bash scripts/gen_round.sh

set -euo pipefail

IMG="src/filters/image"
MOD_FILE="$IMG/mod.rs"
COUNT=0
INPUT=${1:-/dev/stdin}

[ -r "$INPUT" ] || { echo "ERROR cannot read spec input: $INPUT" >&2; exit 1; }

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
    local name=$1
    local mod_line="pub mod ${name};"
    [ -f "$MOD_FILE" ] || { echo "ERROR missing module file: $MOD_FILE" >&2; return 1; }
    if ! grep -Fxq "$mod_line" "$MOD_FILE"; then
        printf '%s\n' "$mod_line" >> "$MOD_FILE"
    fi
}

gi() {
    local n=$1 d=$2 o=$3
    validate_name "$n" || { echo "ERROR invalid Rust module/function name: $n" >&2; return 1; }
    [ -d "$IMG" ] || { echo "ERROR missing directory: $IMG" >&2; return 1; }
    [ -f "$MOD_FILE" ] || { echo "ERROR missing module file: $MOD_FILE" >&2; return 1; }
    [ -f "$IMG/${n}.rs" ] && return
    cat > "$IMG/${n}.rs" << IEOF
//! ${d}
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn ${n}(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) { Some(a) if a.num_components()==1=>a, _=>return input.clone() };
    let n = arr.num_tuples(); let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n).map(|i|{arr.tuple_as_f64(i,&mut buf);${o}}).collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0],dims[1],dims[2]).with_spacing(input.spacing()).with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars,data,1)))
}
#[cfg(test)] mod tests { use super::*;
    #[test] fn test() { let img=ImageData::from_function([5,5,1],[1.0,1.0,1.0],[0.0,0.0,0.0],"v",|x,_,_|x+1.0);
        let r=${n}(&img,"v"); assert_eq!(r.dimensions(),[5,5,1]); } }
IEOF
    register_module "$n"
    COUNT=$((COUNT+1))
}

while IFS= read -r line; do
    case "$line" in
        ""|\#*) continue ;;
        gi\ *)
            if ! [[ "$line" =~ ^gi[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]*::[[:space:]]*(.*)[[:space:]]::[[:space:]]*(.+)$ ]]; then
                echo "Invalid line, expected: gi NAME :: DESCRIPTION :: OPERATION" >&2
                exit 1
            fi
            name=${BASH_REMATCH[1]}
            desc=$(trim "${BASH_REMATCH[2]}")
            op=$(trim "${BASH_REMATCH[3]}")
            gi "$name" "$desc" "$op"
            ;;
        *)
            echo "Unsupported line: $line" >&2
            exit 1
            ;;
    esac
done < "$INPUT"

echo "Generated $COUNT modules"
echo "Registered generated modules in $MOD_FILE."
