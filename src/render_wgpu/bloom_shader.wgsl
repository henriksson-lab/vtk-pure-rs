// NOTE: field offsets must match `BloomUniforms` in bloom_pass.rs byte for byte.
// Padding is spelled out as scalars (rather than vec3) because a vec3 member
// would be re-aligned to 16 by WGSL and silently shift everything after it.
// `weights` is a 16-tap f32 kernel, declared as 4 x vec4 because arrays in the
// uniform address space require a stride that is a multiple of 16.
struct BloomUniforms {
    threshold: f32,
    intensity: f32,
    texel_size: vec2<f32>,
    horizontal: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    weights: array<vec4<f32>, 4>,
    num_weights: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: BloomUniforms;
@group(0) @binding(1)
var source_tex: texture_2d<f32>;
@group(0) @binding(2)
var source_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle (3 vertices, no vertex buffer needed)
@vertex
fn vs_fullscreen(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate full-screen triangle covering [-1,1] x [-1,1]
    let x = f32(i32(idx & 1u)) * 4.0 - 1.0;
    let y = f32(i32(idx >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Pass 1: Extract bright pixels
@fragment
fn fs_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(source_tex, source_sampler, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > uniforms.threshold {
        return vec4<f32>(color.rgb * (brightness - uniforms.threshold), 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// Pass 2/3: Gaussian blur (horizontal or vertical)
@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    let nw = i32(uniforms.num_weights);
    let center = nw / 2;
    var result = vec3<f32>(0.0);

    for (var i = 0; i < nw && i < 16; i = i + 1) {
        let offset = f32(i - center);
        var uv_offset: vec2<f32>;
        if uniforms.horizontal > 0.5 {
            uv_offset = vec2<f32>(offset * uniforms.texel_size.x, 0.0);
        } else {
            uv_offset = vec2<f32>(0.0, offset * uniforms.texel_size.y);
        }
        let sample = textureSample(source_tex, source_sampler, in.uv + uv_offset);
        result += sample.rgb * uniforms.weights[i / 4][i % 4];
    }

    return vec4<f32>(result, 1.0);
}

// Pass 4: Additive composite (pipeline has additive blend state)
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let bloom = textureSample(source_tex, source_sampler, in.uv);
    return vec4<f32>(bloom.rgb * uniforms.intensity, 1.0);
}
