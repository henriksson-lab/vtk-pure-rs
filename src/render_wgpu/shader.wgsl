const MAX_LIGHTS: u32 = 8u;
const PI: f32 = 3.14159265;

struct LightData {
    light_type: f32,
    intensity: f32,
    cone_angle: f32,
    exponent: f32,
    position: vec3<f32>,
    _pad0: f32,
    direction: vec3<f32>,
    _pad1: f32,
    color: vec3<f32>,
    _pad2: f32,
};

struct Uniforms {
    mvp: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    opacity: f32,
    mat_ambient: f32,
    mat_diffuse: f32,
    mat_specular: f32,
    mat_specular_power: f32,
    specular_color: vec3<f32>,
    use_lighting: f32,
    num_lights: f32,
    metallic: f32,
    roughness: f32,
    use_pbr: f32,
    flat_shading: f32,
    num_clip_planes: f32,
    fog_enabled: f32,
    fog_mode: f32,
    clip_planes: array<vec4<f32>, 6>,
    fog_color: vec3<f32>,
    fog_near: f32,
    fog_far: f32,
    fog_density: f32,
    shadow_enabled: f32,
    shadow_darkness: f32,
    light_vp: mat4x4<f32>,
    shadow_bias: f32,
    // Spelled out as scalars: a `vec3<f32>` here would be re-aligned to offset
    // 416 by WGSL and push `lights` 16 bytes past where the Rust-side
    // `Uniforms` struct puts it.
    _shadow_pad0: f32,
    _shadow_pad1: f32,
    _shadow_pad2: f32,
    lights: array<LightData, 8>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var shadow_map: texture_depth_2d;
@group(0) @binding(2)
var shadow_sampler: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = (uniforms.model * vec4<f32>(in.position, 1.0)).xyz;
    out.clip_position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos;
    out.world_normal = (uniforms.model * vec4<f32>(in.normal, 0.0)).xyz;
    out.color = in.color;
    return out;
}

// --- PBR helper functions ---

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d + 0.0001);
}

fn geometry_schlick(n_dot_v: f32, roughness: f32) -> f32 {
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k + 0.0001);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    return geometry_schlick(n_dot_v, roughness) * geometry_schlick(n_dot_l, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn shade_pbr(
    n: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    light_color: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
) -> vec3<f32> {
    let h = normalize(view_dir + light_dir);
    let n_dot_l = max(dot(n, light_dir), 0.0);
    let n_dot_v = max(dot(n, view_dir), 0.001);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, view_dir), 0.0);

    // F0: reflectance at normal incidence
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);
    let f = fresnel_schlick(h_dot_v, f0);

    // Cook-Torrance specular BRDF
    let spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 0.0001);

    // Diffuse: only non-metallic surfaces have diffuse
    let ks = f;
    let kd = (vec3<f32>(1.0) - ks) * (1.0 - metallic);
    let diffuse = kd * albedo / PI;

    return (diffuse + spec) * light_color * n_dot_l;
}

fn compute_shadow(world_pos: vec3<f32>) -> f32 {
    if uniforms.shadow_enabled < 0.5 {
        return 1.0;
    }
    let light_clip = uniforms.light_vp * vec4<f32>(world_pos, 1.0);
    let ndc = light_clip.xyz / light_clip.w;
    // Map from [-1,1] to [0,1] UV space
    let shadow_uv = vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);

    // Out of shadow map bounds → no shadow
    if shadow_uv.x < 0.0 || shadow_uv.x > 1.0 || shadow_uv.y < 0.0 || shadow_uv.y > 1.0 {
        return 1.0;
    }

    let depth = ndc.z;
    // 3x3 PCF for soft shadows
    let texel_size = 1.0 / f32(textureDimensions(shadow_map).x);
    var shadow = 0.0;
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow += textureSampleCompare(
                shadow_map, shadow_sampler,
                shadow_uv + offset,
                depth - uniforms.shadow_bias,
            );
        }
    }
    shadow /= 9.0;
    return 1.0 - uniforms.shadow_darkness * (1.0 - shadow);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Clip planes
    let ncp = u32(uniforms.num_clip_planes);
    for (var ci = 0u; ci < ncp && ci < 6u; ci = ci + 1u) {
        let plane = uniforms.clip_planes[ci];
        if dot(plane.xyz, in.world_position) + plane.w < 0.0 {
            discard;
        }
    }

    if uniforms.use_lighting < 0.5 {
        return vec4<f32>(in.color, uniforms.opacity);
    }

    // Flat shading: compute face normal from screen-space derivatives
    var n: vec3<f32>;
    if uniforms.flat_shading > 0.5 {
        let dx = dpdx(in.world_position);
        let dy = dpdy(in.world_position);
        n = normalize(cross(dx, dy));
        // Ensure normal faces the camera
        if dot(n, normalize(uniforms.camera_pos - in.world_position)) < 0.0 {
            n = -n;
        }
    } else {
        n = normalize(in.world_normal);
    }
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    let num = u32(uniforms.num_lights);

    var total_color = vec3<f32>(0.0, 0.0, 0.0);

    for (var i = 0u; i < num && i < MAX_LIGHTS; i = i + 1u) {
        let light = uniforms.lights[i];
        let lt = u32(light.light_type);

        if lt == 3u {
            // Ambient
            if uniforms.use_pbr > 0.5 {
                total_color += in.color * light.color * light.intensity * 0.03;
            } else {
                total_color += in.color * light.color * light.intensity * uniforms.mat_ambient;
            }
            continue;
        }

        var light_dir: vec3<f32>;
        var attenuation = 1.0;

        if lt == 0u {
            light_dir = normalize(-light.direction);
        } else {
            let to_light = light.position - in.world_position;
            let dist = length(to_light);
            light_dir = to_light / max(dist, 0.0001);
            attenuation = 1.0 / (1.0 + 0.01 * dist * dist);

            if lt == 2u {
                let spot_cos = dot(-light_dir, normalize(light.direction));
                let cone_cos = cos(radians(light.cone_angle));
                if spot_cos < cone_cos {
                    attenuation = 0.0;
                } else {
                    attenuation *= pow(spot_cos, light.exponent);
                }
            }
        }

        if uniforms.use_pbr > 0.5 {
            // PBR path
            total_color += shade_pbr(
                n, view_dir, light_dir,
                light.color * light.intensity * attenuation,
                in.color,
                uniforms.metallic,
                max(uniforms.roughness, 0.04),
            );
        } else {
            // Blinn-Phong path
            let n_dot_l = max(dot(n, light_dir), 0.0);
            let diffuse = in.color * n_dot_l * uniforms.mat_diffuse;

            var specular = vec3<f32>(0.0, 0.0, 0.0);
            if n_dot_l > 0.0 {
                let half_vec = normalize(light_dir + view_dir);
                let n_dot_h = max(dot(n, half_vec), 0.0);
                specular = uniforms.specular_color * pow(n_dot_h, uniforms.mat_specular_power) * uniforms.mat_specular;
            }

            total_color += (diffuse + specular) * light.color * light.intensity * attenuation;
        }
    }

    if num == 0u {
        let fallback_dir = normalize(vec3<f32>(0.3, 0.7, 0.5));
        let d = max(dot(n, fallback_dir), 0.0);
        total_color = in.color * (0.2 + 0.8 * d);
    }

    // Apply shadow
    let shadow_factor = compute_shadow(in.world_position);
    total_color *= shadow_factor;

    var final_color = clamp(total_color, vec3<f32>(0.0), vec3<f32>(1.0));

    // Apply distance fog
    if uniforms.fog_enabled > 0.5 {
        let dist = length(uniforms.camera_pos - in.world_position);
        var fog_factor: f32;
        if uniforms.fog_mode < 0.5 {
            // Linear
            fog_factor = clamp((dist - uniforms.fog_near) / (uniforms.fog_far - uniforms.fog_near), 0.0, 1.0);
        } else if uniforms.fog_mode < 1.5 {
            // Exponential
            fog_factor = 1.0 - exp(-uniforms.fog_density * dist);
        } else {
            // Exponential squared
            let d = uniforms.fog_density * dist;
            fog_factor = 1.0 - exp(-d * d);
        }
        final_color = mix(final_color, uniforms.fog_color, fog_factor);
    }

    return vec4<f32>(final_color, uniforms.opacity);
}
