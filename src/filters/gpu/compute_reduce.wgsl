// Parallel reduction: sum all elements
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: vec4<f32>; // x=count

var<workgroup> sdata: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let count = u32(params.x);
    let idx = gid.x;
    let local = lid.x;

    // Load into workgroup shared memory (`shared` is a WGSL reserved word)
    if idx < count {
        sdata[local] = input[idx];
    } else {
        sdata[local] = 0.0;
    }
    workgroupBarrier();

    // Tree reduction in workgroup shared memory
    var stride = 128u;
    while stride > 0u {
        if local < stride {
            sdata[local] += sdata[local + stride];
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }

    // Write workgroup result
    if local == 0u {
        output[wid.x] = sdata[0];
    }
}
