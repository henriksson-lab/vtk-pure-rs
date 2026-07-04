// GPU picking shader: renders VTK WebGPU selector IDs.
// Each pixel encodes {cell, prop, composite, process} IDs with +1 offsets.

struct PickUniforms {
    mvp: mat4x4<f32>,
    actor_id: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<uniform> pick: PickUniforms;

struct PickVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) cell_id: u32,
};

struct PickVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) cell_id: u32,
};

@vertex
fn vs_pick(in: PickVertexInput) -> PickVertexOutput {
    var out: PickVertexOutput;
    out.position = pick.mvp * vec4<f32>(in.position, 1.0);
    out.cell_id = in.cell_id;
    return out;
}

@fragment
fn fs_pick(in: PickVertexOutput) -> @location(0) vec4<u32> {
    return vec4<u32>(
        in.cell_id + 1u,
        pick.actor_id + 1u,
        1u,
        1u
    );
}
