// Compute per-polygon cell normals from vertex positions.
// Input: positions array (3 floats per vertex), offsets/connectivity cell arrays.
// Output: normals array (3 floats per polygon).

@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> offsets: array<u32>;
@group(0) @binding(2) var<storage, read> connectivity: array<u32>;
@group(0) @binding(3) var<storage, read_write> normals: array<f32>;
@group(0) @binding(4) var<uniform> params: vec4<u32>; // x=num_polys

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let poly_idx = gid.x;
    let num_polys = params.x;
    if poly_idx >= num_polys { return; }

    let start = offsets[poly_idx];
    let end = offsets[poly_idx + 1u];

    var n = vec3<f32>(0.0, 0.0, 0.0);
    if end - start >= 3u {
        var point_id = start;
        var common_point_id = start;
        var v1 = vec3<f32>(0.0, 0.0, 0.0);
        var found_edge = false;

        loop {
            if point_id >= end - 2u {
                break;
            }
            let p0_idx = connectivity[point_id] * 3u;
            let p1_idx = connectivity[point_id + 1u] * 3u;
            let p0 = vec3<f32>(
                positions[p0_idx],
                positions[p0_idx + 1u],
                positions[p0_idx + 2u]
            );
            let p1 = vec3<f32>(
                positions[p1_idx],
                positions[p1_idx + 1u],
                positions[p1_idx + 2u]
            );
            v1 = p1 - p0;
            if dot(v1, v1) > 0.0 {
                common_point_id = point_id;
                point_id = point_id + 2u;
                found_edge = true;
                break;
            }
            point_id = point_id + 1u;
        }

        if found_edge && point_id < end {
            let p0_idx = connectivity[common_point_id] * 3u;
            let p0 = vec3<f32>(
                positions[p0_idx],
                positions[p0_idx + 1u],
                positions[p0_idx + 2u]
            );
            loop {
                if point_id >= end {
                    break;
                }
                let p2_idx = connectivity[point_id] * 3u;
                let v2 = vec3<f32>(
                    positions[p2_idx],
                    positions[p2_idx + 1u],
                    positions[p2_idx + 2u]
                ) - p0;
                n = n + cross(v1, v2);
                v1 = v2;
                point_id = point_id + 1u;
            }
        }
    }

    let len = length(n);
    var normal = vec3<f32>(0.0, 0.0, 0.0);
    if len > 0.0 {
        normal = n / len;
    }

    normals[poly_idx * 3u] = normal.x;
    normals[poly_idx * 3u + 1u] = normal.y;
    normals[poly_idx * 3u + 2u] = normal.z;
}
