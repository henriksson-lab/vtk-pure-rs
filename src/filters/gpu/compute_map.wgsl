// Element-wise math operations: abs, sqrt, log, exp, clamp, pow
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: vec4<f32>; // x=op, y=param1, z=param2, w=count

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let count = u32(params.w);
    if idx >= count { return; }

    let v = input[idx];
    let op = u32(params.x);

    switch op {
        case 0u: { output[idx] = abs(v); }             // Abs
        case 1u: { output[idx] = sqrt(max(v, 0.0)); }  // Sqrt
        case 2u: { output[idx] = log(max(v, 1e-30)); }  // Log
        case 3u: { output[idx] = exp(v); }              // Exp
        case 4u: { output[idx] = clamp(v, params.y, params.z); } // Clamp
        case 5u: { output[idx] = pow(max(v, 0.0), params.y); }   // Pow
        case 6u: { output[idx] = sin(v); }              // Sin
        case 7u: { output[idx] = cos(v); }              // Cos
        case 8u: { output[idx] = v * v; }               // Square
        case 9u: { output[idx] = sign(v) / max(abs(v), 1e-30); } // Reciprocal
        default: { output[idx] = v; }
    }
}
