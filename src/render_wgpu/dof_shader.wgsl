// Depth-of-field post-processing shader.
//
// Translation of VTK's vtkDepthOfFieldPassFS.glsl. The shader computes a
// signed circle of confusion directly from the depth-buffer value, then gathers
// nearby samples with the same stochastic 9x9 kernel used by VTK.

struct DofUniforms {
    world_to_tcoord: vec2<f32>,
    pixel_to_tcoord: vec2<f32>,
    near_c: f32,
    far_c: f32,
    focal_disk: f32,
    focal_distance: f32,
};

@group(0) @binding(0) var<uniform> u: DofUniforms;
@group(0) @binding(1) var color_tex: texture_2d<f32>;
@group(0) @binding(2) var depth_tex: texture_2d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(idx & 1u)) * 4.0 - 1.0;
    let y = f32(i32(idx >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn rand2(co: vec2<f32>) -> vec2<f32> {
    let a = 12.9898;
    let b = 78.233;
    let c = 43758.5453;
    let dt = dot(co.xy, vec2<f32>(a, b));
    let sn = dt - floor(dt / 3.14) * 3.14;
    let dt2 = dot(co.xy, vec2<f32>(b, a));
    let sn2 = dt2 - floor(dt2 / 3.14) * 3.14;
    return vec2<f32>(fract(sin(sn) * c), fract(sin(sn2) * c));
}

fn vtk_depth_of_field(tcoord_vc: vec2<f32>) -> vec4<f32> {
    var fcolor = textureSample(color_tex, tex_sampler, tcoord_vc);
    var fsum = 1.0;

    var fdist = u.focal_distance;
    // Use automatic focalDistance when focalDistance == 0, matching VTK.
    if fdist == 0.0 {
        let center_depth = textureSample(depth_tex, tex_sampler, vec2<f32>(0.5, 0.5)).r;
        fdist = -u.far_c * u.near_c / (center_depth * (u.far_c - u.near_c) - u.far_c);
    }

    let coc_scale = u.focal_disk * fdist * (u.far_c - u.near_c) / (u.far_c * u.near_c);
    let coc_bias = u.focal_disk * (u.near_c - fdist) / u.near_c;

    let cdepth = textureSample(depth_tex, tex_sampler, tcoord_vc).r;
    let coc = coc_scale * cdepth + coc_bias;

    for (var i = 0; i < 9; i = i + 1) {
        for (var j = 0; j < 9; j = j + 1) {
            let new_offset = u.pixel_to_tcoord * (vec2<f32>(f32(i - 4), f32(j - 4)) * 2.0 + rand2(tcoord_vc));
            let new_tc = tcoord_vc + new_offset;
            let tdepth = textureSample(depth_tex, tex_sampler, new_tc).r;
            let t_coc = coc_scale * tdepth + coc_bias;
            // Is the sample in range?
            let close = abs(t_coc) - length(new_offset / u.world_to_tcoord);
            if close > 0.0 {
                // Is the sample to be blended in front? Or, if behind, not too far behind.
                if t_coc < 0.0 || (coc > 0.0 && t_coc < (coc * 2.0)) {
                    let weight = close / abs(t_coc);
                    fcolor = fcolor + weight * textureSample(color_tex, tex_sampler, new_tc);
                    fsum = fsum + weight;
                }
            }
        }
    }

    return fcolor / fsum;
}

@fragment
fn fs_coc(in: VertexOutput) -> @location(0) vec4<f32> {
    return vtk_depth_of_field(in.uv);
}

@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    return vtk_depth_of_field(in.uv);
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(color_tex, tex_sampler, in.uv);
}
