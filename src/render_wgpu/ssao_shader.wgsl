struct SsaoUniforms {
    projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    radius: f32,
    bias: f32,
    intensity: f32,
    num_samples: f32,
    blur: f32,
    texel_size: vec2<f32>,
    near: f32,
    far: f32,
    samples: array<vec4<f32>, 32>,
};

@group(0) @binding(0)
var<uniform> uniforms: SsaoUniforms;
@group(0) @binding(1)
var depth_tex: texture_depth_2d;
@group(0) @binding(2)
var depth_sampler: sampler;
@group(0) @binding(3)
var ao_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(idx & 1u)) * 4.0 - 1.0;
    let y = f32(i32(idx >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Reconstruct view-space position from depth and UV
fn view_pos_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    let view = uniforms.inv_projection * ndc;
    return view.xyz / view.w;
}

// Pseudo-random hash for noise (avoid banding)
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

// SSAO computation
@fragment
fn fs_ssao(in: VertexOutput) -> @location(0) f32 {
    let depth = textureSample(depth_tex, depth_sampler, in.uv);
    if depth <= 0.0 || depth >= 1.0 {
        return 1.0; // background, no occlusion
    }

    let view_pos = view_pos_from_depth(in.uv, depth);

    // Approximate normal from depth derivatives
    let px = view_pos_from_depth(in.uv + vec2<f32>(uniforms.texel_size.x, 0.0),
        textureSample(depth_tex, depth_sampler, in.uv + vec2<f32>(uniforms.texel_size.x, 0.0)));
    let py = view_pos_from_depth(in.uv + vec2<f32>(0.0, uniforms.texel_size.y),
        textureSample(depth_tex, depth_sampler, in.uv + vec2<f32>(0.0, uniforms.texel_size.y)));
    let normal = normalize(cross(py - view_pos, px - view_pos));

    // Random rotation per pixel
    let noise_angle = hash(in.uv * 1000.0) * 6.2831853;
    let cs = cos(noise_angle);
    let sn = sin(noise_angle);
    let random_vec = vec3<f32>(cs, sn, 0.0);
    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);

    var occlusion = 0.0;
    let ns = clamp(u32(uniforms.num_samples), 1u, 32u);

    for (var i = 0u; i < ns; i = i + 1u) {
        var s = uniforms.samples[i].xyz;
        let sample_vec = tangent * s.x + bitangent * s.y + normal * s.z;

        let sample_pos = view_pos + sample_vec * uniforms.radius;

        // Project sample to screen space
        let projected = uniforms.projection * vec4<f32>(sample_pos, 1.0);
        let sample_uv = vec2<f32>(
            projected.x / projected.w * 0.5 + 0.5,
            -projected.y / projected.w * 0.5 + 0.5,
        );

        // Sample depth at projected position
        let sample_depth = textureSample(depth_tex, depth_sampler, sample_uv);
        let sample_view = view_pos_from_depth(sample_uv, sample_depth);

        // Range check + occlusion test
        let range_check = smoothstep(0.0, 1.0, uniforms.radius / abs(view_pos.z - sample_view.z));
        if sample_view.z >= sample_pos.z + uniforms.bias {
            occlusion += range_check;
        }
    }

    occlusion = 1.0 - (occlusion / f32(ns)) * uniforms.intensity;
    return clamp(occlusion, 0.0, 1.0);
}

// Multiply AO onto color (blend state handles multiplication)
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    var ao = textureSample(ao_tex, depth_sampler, in.uv).r;
    if uniforms.blur > 0.5 {
        ao = 0.195346 * textureSample(ao_tex, depth_sampler, in.uv).r
            + 0.077847 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(-1.0, -1.0) * uniforms.texel_size).r
            + 0.077847 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(-1.0, 1.0) * uniforms.texel_size).r
            + 0.077847 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(1.0, -1.0) * uniforms.texel_size).r
            + 0.077847 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(1.0, 1.0) * uniforms.texel_size).r
            + 0.123317 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(-1.0, 0.0) * uniforms.texel_size).r
            + 0.123317 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(1.0, 0.0) * uniforms.texel_size).r
            + 0.123317 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(0.0, -1.0) * uniforms.texel_size).r
            + 0.123317 * textureSample(ao_tex, depth_sampler, in.uv + vec2<f32>(0.0, 1.0) * uniforms.texel_size).r;
    }
    return vec4<f32>(ao, ao, ao, 1.0);
}
