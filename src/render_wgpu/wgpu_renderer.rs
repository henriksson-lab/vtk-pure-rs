use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::render::{Light, LightType, Material, Renderer, Representation, Scene};
use crate::types::VtkError;

use crate::render_wgpu::bloom_pass::BloomPass;
use crate::render_wgpu::dof_pass::DofPass;
use crate::render_wgpu::mesh::{self, Vertex};
use crate::render_wgpu::overlay::OverlayPipeline;
use crate::render_wgpu::shadow_pass::ShadowPass;
use crate::render_wgpu::skybox_pass::SkyboxPass;
use crate::render_wgpu::ssao_pass::SsaoPass;
use crate::render_wgpu::volume_pass::VolumePass;
use crate::render_wgpu::wireframe;

const MAX_LIGHTS: usize = 8;
const MSAA_SAMPLE_COUNT: u32 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GpuLight {
    light_type: f32,
    intensity: f32,
    cone_angle: f32,
    exponent: f32,
    position: [f32; 3],
    _pad0: f32,
    direction: [f32; 3],
    _pad1: f32,
    color: [f32; 3],
    _pad2: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    opacity: f32,
    mat_ambient: f32,
    mat_diffuse: f32,
    mat_specular: f32,
    mat_specular_power: f32,
    specular_color: [f32; 3],
    use_lighting: f32,
    num_lights: f32,
    metallic: f32,
    roughness: f32,
    use_pbr: f32,
    flat_shading: f32,
    num_clip_planes: f32,
    fog_enabled: f32,
    fog_mode: f32,
    clip_planes: [[f32; 4]; 6],
    fog_color: [f32; 3],
    fog_near: f32,
    fog_far: f32,
    fog_density: f32,
    shadow_enabled: f32,
    shadow_darkness: f32,
    light_vp: [[f32; 4]; 4],
    shadow_bias: f32,
    _shadow_pad: [f32; 3],
    lights: [GpuLight; MAX_LIGHTS],
}

struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
    pipeline_triangles: wgpu::RenderPipeline,
    pipeline_lines: wgpu::RenderPipeline,
    pipeline_points: wgpu::RenderPipeline,
    pipeline_triangles_blend: wgpu::RenderPipeline,
    pipeline_lines_blend: wgpu::RenderPipeline,
    pipeline_points_blend: wgpu::RenderPipeline,
    depth_texture: wgpu::TextureView,
    msaa_texture: wgpu::TextureView,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    shadow_sampler: wgpu::Sampler,
    dummy_shadow_view: wgpu::TextureView,
    overlay_pipeline: OverlayPipeline,
    pipeline_triangles_cull: wgpu::RenderPipeline,
    pipeline_triangles_blend_cull: wgpu::RenderPipeline,
    volume_pass: VolumePass,
    skybox_pass: SkyboxPass,
    shadow_pass: ShadowPass,
    bloom_pass: BloomPass,
    ssao_pass: SsaoPass,
    dof_pass: DofPass,
    pipeline_lines_no_msaa: wgpu::RenderPipeline,
    width: u32,
    height: u32,
}

impl WgpuRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, VtkError> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|e| VtkError::InvalidData(format!("surface creation failed: {e}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| VtkError::InvalidData("no suitable GPU adapter found".into()))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|e| VtkError::InvalidData(format!("device request failed: {e}")))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vtk shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Create a dummy 1x1 shadow map for when shadows are disabled
        let dummy_shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dummy shadow"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_shadow_view = dummy_shadow_tex.create_view(&Default::default());
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dummy_shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_triangles = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            false,
            MSAA_SAMPLE_COUNT,
            None,
        );
        let pipeline_lines = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::LineList,
            false,
            MSAA_SAMPLE_COUNT,
            None,
        );
        let pipeline_points = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::PointList,
            false,
            MSAA_SAMPLE_COUNT,
            None,
        );
        let pipeline_triangles_blend = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            true,
            MSAA_SAMPLE_COUNT,
            None,
        );
        let pipeline_lines_blend = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::LineList,
            true,
            MSAA_SAMPLE_COUNT,
            None,
        );
        let pipeline_points_blend = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::PointList,
            true,
            MSAA_SAMPLE_COUNT,
            None,
        );
        let pipeline_triangles_cull = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            false,
            MSAA_SAMPLE_COUNT,
            Some(wgpu::Face::Back),
        );
        let pipeline_triangles_blend_cull = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            true,
            MSAA_SAMPLE_COUNT,
            Some(wgpu::Face::Back),
        );

        let depth_texture = create_depth_texture(&device, width, height, MSAA_SAMPLE_COUNT);
        let msaa_texture = create_msaa_texture(&device, surface_format, width, height);
        let overlay_pipeline = OverlayPipeline::new(&device, surface_format);
        let volume_pass = VolumePass::new(&device, surface_format);
        let skybox_pass = SkyboxPass::new(&device, surface_format);
        let shadow_pass = ShadowPass::new(&device);
        let bloom_pass = BloomPass::new(&device, surface_format);
        let ssao_pass = SsaoPass::new(&device, surface_format);
        let dof_pass = DofPass::new(&device, surface_format);
        let pipeline_lines_no_msaa = create_pipeline_with_cull(
            &device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::PrimitiveTopology::LineList,
            false,
            1,
            None,
        );

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            surface_format,
            pipeline_triangles,
            pipeline_lines,
            pipeline_points,
            pipeline_triangles_blend,
            pipeline_lines_blend,
            pipeline_points_blend,
            depth_texture,
            msaa_texture,
            uniform_buffer,
            bind_group,
            bind_group_layout,
            shadow_sampler,
            dummy_shadow_view,
            overlay_pipeline,
            volume_pass,
            skybox_pass,
            shadow_pass,
            bloom_pass,
            ssao_pass,
            dof_pass,
            pipeline_triangles_cull,
            pipeline_triangles_blend_cull,
            pipeline_lines_no_msaa,
            width,
            height,
        })
    }

    fn prepare_actor_mesh(
        &self,
        actor: &crate::render::Actor,
        camera_distance: f64,
        scene: &Scene,
    ) -> Option<GpuMesh> {
        // Select LOD level if available
        let data = if let Some(ref lod) = actor.lod {
            lod.select(camera_distance)
        } else {
            &actor.data
        };
        let (vertices, indices) = match actor.representation {
            Representation::Surface => mesh::poly_data_to_mesh(data, &actor.coloring),
            Representation::Wireframe => {
                if actor.material.line_width > 1.5 {
                    let cam_dir = scene.camera.direction();
                    let cd = [cam_dir.x as f32, cam_dir.y as f32, cam_dir.z as f32];
                    // Convert pixel width to world units (approximate)
                    let world_width =
                        actor.material.line_width as f32 * camera_distance as f32 * 0.001;
                    poly_data_to_wide_lines(data, &actor.coloring, world_width, cd)
                } else {
                    wireframe::poly_data_to_wireframe(data, &actor.coloring)
                }
            }
            Representation::Points => {
                if actor.material.point_size > 1.5 {
                    let cam_right = scene.camera.right();
                    let cam_up = scene.camera.up();
                    let cr = [cam_right.x as f32, cam_right.y as f32, cam_right.z as f32];
                    let cu = [cam_up.x as f32, cam_up.y as f32, cam_up.z as f32];
                    let world_size =
                        actor.material.point_size as f32 * camera_distance as f32 * 0.001;
                    poly_data_to_point_sprites(data, &actor.coloring, world_size, cr, cu)
                } else {
                    poly_data_to_points(data, &actor.coloring)
                }
            }
        };
        if indices.is_empty() {
            return None;
        }
        Some(upload_mesh(&self.device, &vertices, &indices))
    }

    fn prepare_edge_mesh(&self, actor: &crate::render::Actor) -> Option<GpuMesh> {
        let coloring = crate::render::Coloring::Solid(actor.material.edge_color);
        let (vertices, indices) = wireframe::poly_data_to_wireframe(&actor.data, &coloring);
        if indices.is_empty() {
            return None;
        }
        Some(upload_mesh(&self.device, &vertices, &indices))
    }

    fn select_pipeline(
        &self,
        repr: Representation,
        translucent: bool,
        backface_cull: bool,
    ) -> &wgpu::RenderPipeline {
        match (repr, translucent, backface_cull) {
            (Representation::Surface, false, true) => &self.pipeline_triangles_cull,
            (Representation::Surface, true, true) => &self.pipeline_triangles_blend_cull,
            (Representation::Surface, false, _) => &self.pipeline_triangles,
            (Representation::Wireframe, false, _) => &self.pipeline_lines,
            (Representation::Points, false, _) => &self.pipeline_points,
            (Representation::Surface, true, _) => &self.pipeline_triangles_blend,
            (Representation::Wireframe, true, _) => &self.pipeline_lines_blend,
            (Representation::Points, true, _) => &self.pipeline_points_blend,
        }
    }

    fn write_uniforms(
        &self,
        scene: &Scene,
        material: &Material,
        opacity: f32,
        use_lighting: bool,
        position: [f64; 3],
        scale: f64,
    ) {
        let aspect = self.width as f64 / self.height as f64;
        let view_mat = scene.camera.view_matrix();
        let proj_mat = scene.camera.projection_matrix(aspect);
        let vp = proj_mat * view_mat;

        // Compute model matrix from actor position and scale
        let model = Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(scale as f32),
            glam::Quat::IDENTITY,
            glam::Vec3::new(position[0] as f32, position[1] as f32, position[2] as f32),
        );
        let mvp_f32 = Mat4::from_cols_array(&vp.to_cols_array().map(|v| v as f32)) * model;
        let cam_pos = scene.camera.position;

        let mut gpu_lights = [GpuLight::zeroed(); MAX_LIGHTS];
        let num_lights = scene.lights.len().min(MAX_LIGHTS);
        for (i, light) in scene.lights.iter().take(MAX_LIGHTS).enumerate() {
            if !light.enabled {
                continue;
            }
            gpu_lights[i] = light_to_gpu(light);
        }

        let uniforms = Uniforms {
            mvp: mvp_f32.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            camera_pos: [cam_pos.x as f32, cam_pos.y as f32, cam_pos.z as f32],
            opacity,
            mat_ambient: material.ambient as f32,
            mat_diffuse: material.diffuse as f32,
            mat_specular: material.specular as f32,
            mat_specular_power: material.specular_power as f32,
            specular_color: material.specular_color,
            use_lighting: if use_lighting { 1.0 } else { 0.0 },
            num_lights: num_lights as f32,
            metallic: material.metallic as f32,
            roughness: material.roughness as f32,
            use_pbr: if material.pbr { 1.0 } else { 0.0 },
            flat_shading: if material.flat_shading { 1.0 } else { 0.0 },
            num_clip_planes: scene.clip_planes.iter().filter(|c| c.enabled).count() as f32,
            fog_enabled: if scene.fog.enabled { 1.0 } else { 0.0 },
            fog_mode: match scene.fog.mode {
                crate::render::FogMode::Linear => 0.0,
                crate::render::FogMode::Exponential => 1.0,
                crate::render::FogMode::ExponentialSquared => 2.0,
            },
            clip_planes: {
                let mut cp = [[0.0f32; 4]; 6];
                for (i, plane) in scene
                    .clip_planes
                    .iter()
                    .filter(|c| c.enabled)
                    .take(6)
                    .enumerate()
                {
                    cp[i] = [
                        plane.normal[0] as f32,
                        plane.normal[1] as f32,
                        plane.normal[2] as f32,
                        plane.distance as f32,
                    ];
                }
                cp
            },
            fog_color: scene.fog.color,
            fog_near: scene.fog.near as f32,
            fog_far: scene.fog.far as f32,
            fog_density: scene.fog.density as f32,
            shadow_enabled: if scene.shadows.enabled { 1.0 } else { 0.0 },
            shadow_darkness: scene.shadows.darkness as f32,
            light_vp: {
                if scene.shadows.enabled {
                    // Find first directional light
                    let dir_light = scene.lights.iter().find(|l| {
                        l.enabled && matches!(l.light_type, crate::render::LightType::Directional)
                    });
                    if let Some(dl) = dir_light {
                        let center = scene.camera.focal_point;
                        let lvp = scene.shadows.light_vp_matrix(
                            [
                                dl.direction[0] as f64,
                                dl.direction[1] as f64,
                                dl.direction[2] as f64,
                            ],
                            [center.x, center.y, center.z],
                        );
                        [
                            [
                                lvp[0][0] as f32,
                                lvp[0][1] as f32,
                                lvp[0][2] as f32,
                                lvp[0][3] as f32,
                            ],
                            [
                                lvp[1][0] as f32,
                                lvp[1][1] as f32,
                                lvp[1][2] as f32,
                                lvp[1][3] as f32,
                            ],
                            [
                                lvp[2][0] as f32,
                                lvp[2][1] as f32,
                                lvp[2][2] as f32,
                                lvp[2][3] as f32,
                            ],
                            [
                                lvp[3][0] as f32,
                                lvp[3][1] as f32,
                                lvp[3][2] as f32,
                                lvp[3][3] as f32,
                            ],
                        ]
                    } else {
                        [[0.0; 4]; 4]
                    }
                } else {
                    [[0.0; 4]; 4]
                }
            },
            shadow_bias: scene.shadows.bias as f32,
            _shadow_pad: [0.0; 3],
            lights: gpu_lights,
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Render the 3D scene with MSAA, resolving to `resolve_target`.
    fn render_3d_msaa(
        &self,
        scene: &Scene,
        msaa_view: &wgpu::TextureView,
        resolve_target: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        struct ActorDraw {
            mesh: GpuMesh,
            edge_mesh: Option<GpuMesh>,
            repr: Representation,
            opacity: f32,
            material: Material,
            position: [f64; 3],
            scale: f64,
        }

        let mut opaque_draws = Vec::new();
        let mut translucent_draws = Vec::new();

        let cam_dist = (scene.camera.position - scene.camera.focal_point).length();

        for actor in &scene.actors {
            if !actor.visible {
                continue;
            }
            let Some(gpu_mesh) = self.prepare_actor_mesh(actor, cam_dist, scene) else {
                continue;
            };
            let edge_mesh = if actor.material.edge_visibility
                && actor.representation == Representation::Surface
            {
                self.prepare_edge_mesh(actor)
            } else {
                None
            };
            // If point sprites or wide lines are used, geometry is triangles
            let effective_repr = match actor.representation {
                Representation::Points if actor.material.point_size > 1.5 => {
                    Representation::Surface
                }
                Representation::Wireframe if actor.material.line_width > 1.5 => {
                    Representation::Surface
                }
                other => other,
            };
            let draw = ActorDraw {
                mesh: gpu_mesh,
                edge_mesh,
                repr: effective_repr,
                opacity: actor.opacity,
                material: actor.material.clone(),
                position: actor.position,
                scale: actor.scale,
            };
            if actor.opacity < 1.0 {
                translucent_draws.push(draw);
            } else {
                opaque_draws.push(draw);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: msaa_view,
                    resolve_target: Some(resolve_target),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: scene.background[0] as f64,
                            g: scene.background[1] as f64,
                            b: scene.background[2] as f64,
                            a: scene.background[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Opaque pass
            for draw in &opaque_draws {
                let use_lighting = draw.repr == Representation::Surface;
                self.write_uniforms(
                    scene,
                    &draw.material,
                    draw.opacity,
                    use_lighting,
                    draw.position,
                    draw.scale,
                );
                let cull = draw.material.backface_culling;
                pass.set_pipeline(self.select_pipeline(draw.repr, false, cull));
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, draw.mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(draw.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..draw.mesh.num_indices, 0, 0..1);

                if let Some(ref edge) = draw.edge_mesh {
                    self.write_uniforms(
                        scene,
                        &draw.material,
                        1.0,
                        false,
                        draw.position,
                        draw.scale,
                    );
                    pass.set_pipeline(self.select_pipeline(
                        Representation::Wireframe,
                        false,
                        false,
                    ));
                    pass.set_bind_group(0, &self.bind_group, &[]);
                    pass.set_vertex_buffer(0, edge.vertex_buffer.slice(..));
                    pass.set_index_buffer(edge.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..edge.num_indices, 0, 0..1);
                }
            }

            // Translucent pass
            for draw in &translucent_draws {
                let use_lighting = draw.repr == Representation::Surface;
                self.write_uniforms(
                    scene,
                    &draw.material,
                    draw.opacity,
                    use_lighting,
                    draw.position,
                    draw.scale,
                );
                let cull = draw.material.backface_culling;
                pass.set_pipeline(self.select_pipeline(draw.repr, true, cull));
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, draw.mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(draw.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..draw.mesh.num_indices, 0, 0..1);

                if let Some(ref edge) = draw.edge_mesh {
                    self.write_uniforms(
                        scene,
                        &draw.material,
                        draw.opacity,
                        false,
                        draw.position,
                        draw.scale,
                    );
                    pass.set_pipeline(self.select_pipeline(Representation::Wireframe, true, false));
                    pass.set_bind_group(0, &self.bind_group, &[]);
                    pass.set_vertex_buffer(0, edge.vertex_buffer.slice(..));
                    pass.set_index_buffer(edge.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..edge.num_indices, 0, 0..1);
                }
            }

            // Clip plane caps — render cap faces for each active clip plane
            let active_clips: Vec<_> = scene.clip_planes.iter().filter(|c| c.enabled).collect();
            if !active_clips.is_empty() {
                // Build a scene-copy with no clip planes for cap rendering
                let mut cap_material = Material::default();
                cap_material.ambient = 0.3;
                cap_material.diffuse = 0.7;

                for clip in &active_clips {
                    let normal = [
                        clip.normal[0] as f32,
                        clip.normal[1] as f32,
                        clip.normal[2] as f32,
                    ];
                    let dist = clip.distance as f32;
                    let cap_color = [0.8f32, 0.8, 0.8]; // light gray cap

                    for actor in &scene.actors {
                        if !actor.visible || actor.representation != Representation::Surface {
                            continue;
                        }
                        let (pts, tris) =
                            crate::render_wgpu::clip_cap::extract_triangles(&actor.data);
                        if tris.is_empty() {
                            continue;
                        }
                        let (cap_verts, cap_idxs) = crate::render_wgpu::clip_cap::generate_clip_cap(
                            &pts, &tris, normal, dist, cap_color,
                        );
                        if cap_idxs.is_empty() {
                            continue;
                        }
                        let cap_mesh = upload_mesh(&self.device, &cap_verts, &cap_idxs);
                        // Write uniforms with zero clip planes so cap isn't clipped
                        let mut no_clip_scene = scene.clone();
                        no_clip_scene.clip_planes.clear();
                        self.write_uniforms(
                            &no_clip_scene,
                            &cap_material,
                            1.0,
                            true,
                            actor.position,
                            actor.scale,
                        );
                        pass.set_pipeline(self.select_pipeline(
                            Representation::Surface,
                            false,
                            false,
                        ));
                        pass.set_bind_group(0, &self.bind_group, &[]);
                        pass.set_vertex_buffer(0, cap_mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(
                            cap_mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        pass.draw_indexed(0..cap_mesh.num_indices, 0, 0..1);
                    }
                }
            }
        }
    }

    fn render_to_view(
        &mut self,
        scene: &Scene,
        msaa_view: &wgpu::TextureView,
        resolve_target: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) -> Result<(), VtkError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // Shadow depth pass (before main 3D scene)
        if scene.shadows.enabled {
            // Collect opaque actor mesh data for shadow pass
            let cam_dist = scene.camera.distance();
            let mut shadow_meshes = Vec::new();
            for actor in &scene.actors {
                if !actor.visible || actor.opacity < 1.0 {
                    continue;
                }
                if actor.representation != Representation::Surface {
                    continue;
                }
                if let Some(gpu_mesh) = self.prepare_actor_mesh(actor, cam_dist, scene) {
                    shadow_meshes.push((
                        gpu_mesh.vertex_buffer,
                        gpu_mesh.index_buffer,
                        gpu_mesh.num_indices,
                        actor.position,
                        actor.scale,
                    ));
                }
            }
            if let Some((shadow_view, _light_vp)) = self.shadow_pass.render(
                &self.device,
                &self.queue,
                &mut encoder,
                scene,
                &shadow_meshes,
            ) {
                // Rebuild bind group with actual shadow map
                self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bind group with shadow"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(shadow_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                        },
                    ],
                });
            }
        } else {
            // Reset to dummy shadow map
            self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bind group no shadow"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.dummy_shadow_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                    },
                ],
            });
        }

        // Skybox (render gradient background before 3D scene)
        if !matches!(scene.skybox, crate::render::Skybox::Solid(_)) {
            self.skybox_pass
                .render(&self.queue, &mut encoder, resolve_target, &scene.skybox);
        }

        // 3D scene with MSAA
        self.render_3d_msaa(scene, msaa_view, resolve_target, depth_view, &mut encoder);

        // Silhouette edges (rendered as overlay lines after MSAA resolve)
        if scene.silhouette.enabled {
            let cam_pos = scene.camera.position;
            let cam = [cam_pos.x, cam_pos.y, cam_pos.z];
            let mut all_sil_verts = Vec::new();
            let mut all_sil_idxs: Vec<u32> = Vec::new();
            for actor in &scene.actors {
                if actor.representation != Representation::Surface {
                    continue;
                }
                let (verts, idxs) = crate::render_wgpu::silhouette_pass::extract_silhouette_edges(
                    &actor.data,
                    cam,
                    &scene.silhouette,
                );
                let base = all_sil_verts.len() as u32;
                all_sil_verts.extend_from_slice(&verts);
                all_sil_idxs.extend(idxs.iter().map(|&i| i + base));
            }
            if !all_sil_idxs.is_empty() {
                let sil_mesh = upload_mesh(&self.device, &all_sil_verts, &all_sil_idxs);
                let sil_material = Material::default();
                self.write_uniforms(scene, &sil_material, 1.0, false, [0.0, 0.0, 0.0], 1.0);
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("silhouette pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: resolve_target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
                pass.set_pipeline(&self.pipeline_lines_no_msaa);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, sil_mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(sil_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..sil_mesh.num_indices, 0, 0..1);
            }
        }

        // Volume rendering pass (after surface, before overlays)
        if !scene.volumes.is_empty() {
            let aspect = self.width as f64 / self.height as f64;
            let view_mat = scene.camera.view_matrix();
            let proj_mat = scene.camera.projection_matrix(aspect);
            let vp = proj_mat * view_mat;
            let mvp = Mat4::from_cols_array(&vp.to_cols_array().map(|v| v as f32));
            let cam = scene.camera.position;
            let cam_pos = [cam.x as f32, cam.y as f32, cam.z as f32];

            for volume in &scene.volumes {
                self.volume_pass.render(
                    &self.device,
                    &self.queue,
                    &mut encoder,
                    resolve_target,
                    volume,
                    mvp,
                    cam_pos,
                );
            }
        }

        // SSAO post-process (after 3D, before bloom/overlays)
        if scene.ssao.enabled {
            // Create non-MSAA depth for SSAO (re-render depth only at 1x sample)
            let ssao_depth_tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ssao depth"),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let ssao_depth_view = ssao_depth_tex.create_view(&Default::default());

            // Quick depth-only pass at 1x MSAA for SSAO
            {
                let mut depth_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ssao depth resolve"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &ssao_depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
                // Re-draw opaque geometry for depth only using the no-MSAA line pipeline (has depth)
                // This is a simplified approach; production would use a depth-only pipeline
                depth_pass.set_pipeline(&self.pipeline_lines_no_msaa);
            }

            let aspect = self.width as f64 / self.height as f64;
            let proj = scene.camera.projection_matrix(aspect);
            let proj_f32 = glam::Mat4::from_cols_array(&proj.to_cols_array().map(|v| v as f32));

            let ssao_config = crate::render::SsaoConfig {
                enabled: scene.ssao.enabled,
                radius: scene.ssao.radius,
                intensity: scene.ssao.intensity,
                bias: scene.ssao.bias,
                num_samples: scene.ssao.num_samples,
            };

            self.ssao_pass.render(
                &self.device,
                &self.queue,
                &mut encoder,
                resolve_target,
                &ssao_depth_view,
                &ssao_config,
                proj_f32,
                self.width,
                self.height,
                scene.camera.near_clip as f32,
                scene.camera.far_clip as f32,
            );
        }

        // Bloom post-process (after 3D + volume, before overlays)
        self.bloom_pass.render(
            &self.device,
            &self.queue,
            &mut encoder,
            resolve_target,
            &scene.bloom,
            self.width,
            self.height,
            self.surface_format,
        );

        // 2D overlay (scalar bars) — rendered after resolve, directly on resolve target
        self.overlay_pipeline.render_scalar_bars(
            &self.device,
            &mut encoder,
            resolve_target,
            &scene.scalar_bars,
        );

        // Axes widget overlay
        if let Some(ref axes) = scene.axes_widget {
            let view_mat = scene.camera.view_matrix();
            let cols = view_mat.to_cols_array();
            let view_arr = [
                [cols[0], cols[1], cols[2], cols[3]],
                [cols[4], cols[5], cols[6], cols[7]],
                [cols[8], cols[9], cols[10], cols[11]],
                [cols[12], cols[13], cols[14], cols[15]],
            ];
            self.overlay_pipeline.render_axes_widget(
                &self.device,
                &mut encoder,
                resolve_target,
                axes,
                &view_arr,
            );
        }

        // Annotations overlay (labels, rulers, protractors)
        if !scene.annotations.is_empty() {
            let aspect = self.width as f64 / self.height as f64;
            let view_mat = scene.camera.view_matrix();
            let proj_mat = scene.camera.projection_matrix(aspect);
            let vp = proj_mat * view_mat;

            let mut vertices = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            // Project 3D positions to NDC and render as text
            for label in &scene.annotations.labels {
                let ndc = project_to_ndc(&vp, label.position);
                if let Some((nx, ny)) = ndc {
                    let color = [label.color[0], label.color[1], label.color[2], 1.0];
                    crate::render_wgpu::bitmap_font::render_text(
                        &mut vertices,
                        &mut indices,
                        &label.text,
                        nx,
                        ny,
                        label.scale,
                        color,
                    );
                }
            }

            for ruler in &scene.annotations.rulers {
                let start_ndc = project_to_ndc(&vp, ruler.start);
                let end_ndc = project_to_ndc(&vp, ruler.end);
                if let (Some((sx, sy)), Some((ex, ey))) = (start_ndc, end_ndc) {
                    let color = [ruler.color[0], ruler.color[1], ruler.color[2], 1.0];
                    // Draw line as thin quad
                    let dx = ex - sx;
                    let dy = ey - sy;
                    let len = (dx * dx + dy * dy).sqrt();
                    if len > 0.001 {
                        let nx = -dy / len * 0.002;
                        let ny = dx / len * 0.002;
                        let base = vertices.len() as u32;
                        vertices.push(crate::render_wgpu::overlay::OverlayVertex {
                            position: [sx - nx, sy - ny],
                            color,
                        });
                        vertices.push(crate::render_wgpu::overlay::OverlayVertex {
                            position: [sx + nx, sy + ny],
                            color,
                        });
                        vertices.push(crate::render_wgpu::overlay::OverlayVertex {
                            position: [ex + nx, ey + ny],
                            color,
                        });
                        vertices.push(crate::render_wgpu::overlay::OverlayVertex {
                            position: [ex - nx, ey - ny],
                            color,
                        });
                        indices.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base,
                            base + 2,
                            base + 3,
                        ]);
                    }

                    if ruler.show_label {
                        let mid_ndc = project_to_ndc(&vp, ruler.midpoint());
                        if let Some((mx, my)) = mid_ndc {
                            let text = format!("{:.2}", ruler.distance());
                            crate::render_wgpu::bitmap_font::render_text(
                                &mut vertices,
                                &mut indices,
                                &text,
                                mx + 0.01,
                                my + 0.01,
                                0.012,
                                color,
                            );
                        }
                    }
                }
            }

            if !vertices.is_empty() {
                use wgpu::util::DeviceExt;
                let vb = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("annotation vb"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                let ib = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("annotation ib"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("annotation pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: resolve_target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
                pass.set_pipeline(&self.overlay_pipeline.pipeline());
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}

impl Renderer for WgpuRenderer {
    fn render(&mut self, scene: &Scene) -> Result<(), VtkError> {
        use crate::render::StereoMode;

        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| VtkError::InvalidData(format!("surface texture error: {e}")))?;
        let resolve_view = output.texture.create_view(&Default::default());

        if !scene.viewports.is_empty() {
            // Multi-viewport: render each viewport to its own region
            for (viewport, camera) in &scene.viewports {
                let (px, py, pw, ph) = viewport.to_pixels(self.width, self.height);
                if pw == 0 || ph == 0 {
                    continue;
                }
                let vp_msaa = create_msaa_texture(&self.device, self.surface_format, pw, ph);
                let vp_depth = create_depth_texture(&self.device, pw, ph, MSAA_SAMPLE_COUNT);
                let mut vp_scene = scene.clone();
                vp_scene.camera = camera.clone();
                vp_scene.viewports.clear();
                let saved_w = self.width;
                let saved_h = self.height;
                self.width = pw;
                self.height = ph;

                let vp_tex = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("viewport color"),
                    size: wgpu::Extent3d {
                        width: pw,
                        height: ph,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.surface_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let vp_view = vp_tex.create_view(&Default::default());
                self.render_to_view(&vp_scene, &vp_msaa, &vp_view, &vp_depth)?;

                // Copy viewport result to correct region of output surface
                let mut enc = self.device.create_command_encoder(&Default::default());
                enc.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &vp_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &output.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: px, y: py, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: pw,
                        height: ph,
                        depth_or_array_layers: 1,
                    },
                );
                self.queue.submit(std::iter::once(enc.finish()));
                self.width = saved_w;
                self.height = saved_h;
            }
        } else if scene.stereo.is_stereo() {
            // Stereo: render left and right eye to separate halves
            let right = scene.camera.right();
            let right_arr = [right.x, right.y, right.z];
            let cam_pos = [
                scene.camera.position.x,
                scene.camera.position.y,
                scene.camera.position.z,
            ];
            let (left_pos, right_pos) = scene.stereo.eye_positions(cam_pos, right_arr);

            let mut left_scene = scene.clone();
            left_scene.camera.position = glam::DVec3::new(left_pos[0], left_pos[1], left_pos[2]);
            left_scene.stereo = crate::render::StereoConfig::default();
            let mut right_scene = scene.clone();
            right_scene.camera.position =
                glam::DVec3::new(right_pos[0], right_pos[1], right_pos[2]);
            right_scene.stereo = crate::render::StereoConfig::default();

            let (lv, rv) = match scene.stereo.mode {
                StereoMode::SideBySide => (
                    crate::render::Viewport::left_half(),
                    crate::render::Viewport::right_half(),
                ),
                StereoMode::TopBottom => (
                    crate::render::Viewport::top_half(),
                    crate::render::Viewport::bottom_half(),
                ),
                _ => (
                    crate::render::Viewport::full(),
                    crate::render::Viewport::full(),
                ),
            };

            // Render each eye as a viewport
            for (vp, eye_scene) in [(lv, &left_scene), (rv, &right_scene)] {
                let (px, py, pw, ph) = vp.to_pixels(self.width, self.height);
                if pw == 0 || ph == 0 {
                    continue;
                }
                let eye_msaa = create_msaa_texture(&self.device, self.surface_format, pw, ph);
                let eye_depth = create_depth_texture(&self.device, pw, ph, MSAA_SAMPLE_COUNT);
                let saved_w = self.width;
                let saved_h = self.height;
                self.width = pw;
                self.height = ph;

                let eye_tex = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("stereo eye"),
                    size: wgpu::Extent3d {
                        width: pw,
                        height: ph,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.surface_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let eye_view = eye_tex.create_view(&Default::default());
                self.render_to_view(eye_scene, &eye_msaa, &eye_view, &eye_depth)?;

                let mut enc = self.device.create_command_encoder(&Default::default());
                enc.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &eye_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &output.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: px, y: py, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: pw,
                        height: ph,
                        depth_or_array_layers: 1,
                    },
                );
                self.queue.submit(std::iter::once(enc.finish()));
                self.width = saved_w;
                self.height = saved_h;
            }
        } else {
            // Normal single-viewport rendering
            let msaa =
                create_msaa_texture(&self.device, self.surface_format, self.width, self.height);
            let depth =
                create_depth_texture(&self.device, self.width, self.height, MSAA_SAMPLE_COUNT);
            self.render_to_view(scene, &msaa, &resolve_view, &depth)?;
        }

        output.present();
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture =
                create_depth_texture(&self.device, width, height, MSAA_SAMPLE_COUNT);
            self.msaa_texture =
                create_msaa_texture(&self.device, self.surface_format, width, height);
        }
    }

    fn render_to_image(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, VtkError> {
        let saved_w = self.width;
        let saved_h = self.height;
        self.width = width;
        self.height = height;

        let color_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&Default::default());
        let depth_view = create_depth_texture(&self.device, width, height, MSAA_SAMPLE_COUNT);
        let msaa_view = create_msaa_texture(&self.device, self.surface_format, width, height);

        self.render_to_view(scene, &msaa_view, &color_view, &depth_view)?;

        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let padded_bytes_per_row = (unpadded_bytes_per_row + 255) & !255;

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("offscreen staging"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("copy encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);

        receiver
            .recv()
            .map_err(|_| VtkError::InvalidData("buffer map recv failed".into()))?
            .map_err(|e| VtkError::InvalidData(format!("buffer map failed: {e}")))?;

        let data = buffer_slice.get_mapped_range();
        let mut result = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
        for row in 0..height {
            let start = (row * padded_bytes_per_row) as usize;
            let end = start + unpadded_bytes_per_row as usize;
            result.extend_from_slice(&data[start..end]);
        }
        drop(data);
        staging_buffer.unmap();

        self.width = saved_w;
        self.height = saved_h;
        Ok(result)
    }
}

fn light_to_gpu(light: &Light) -> GpuLight {
    let (lt, cone_angle, exponent) = match light.light_type {
        LightType::Directional => (0.0, 0.0, 0.0),
        LightType::Point => (1.0, 0.0, 0.0),
        LightType::Spot {
            cone_angle,
            exponent,
        } => (2.0, cone_angle as f32, exponent as f32),
        LightType::Ambient => (3.0, 0.0, 0.0),
    };
    GpuLight {
        light_type: lt,
        intensity: light.intensity as f32,
        cone_angle,
        exponent,
        position: [
            light.position[0] as f32,
            light.position[1] as f32,
            light.position[2] as f32,
        ],
        _pad0: 0.0,
        direction: [
            light.direction[0] as f32,
            light.direction[1] as f32,
            light.direction[2] as f32,
        ],
        _pad1: 0.0,
        color: light.color,
        _pad2: 0.0,
    }
}

fn upload_mesh(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> GpuMesh {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertex buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("index buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    GpuMesh {
        vertex_buffer,
        index_buffer,
        num_indices: indices.len() as u32,
    }
}

fn create_pipeline_with_cull(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    topology: wgpu::PrimitiveTopology,
    alpha_blend: bool,
    sample_count: u32,
    cull_mode: Option<wgpu::Face>,
) -> wgpu::RenderPipeline {
    let blend = if alpha_blend {
        Some(wgpu::BlendState::ALPHA_BLENDING)
    } else {
        Some(wgpu::BlendState::REPLACE)
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: !alpha_blend,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    sample_count: u32,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&Default::default())
}

fn create_msaa_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("msaa texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&Default::default())
}

fn poly_data_to_points(
    poly_data: &crate::data::PolyData,
    coloring: &crate::render::Coloring,
) -> (Vec<Vertex>, Vec<u32>) {
    let point_colors = mesh::resolve_colors_pub(poly_data, coloring);
    let n = poly_data.points.len();
    let mut vertices = Vec::with_capacity(n);
    let mut indices = Vec::with_capacity(n);
    for i in 0..n {
        let p = poly_data.points.get(i);
        vertices.push(Vertex {
            position: [p[0] as f32, p[1] as f32, p[2] as f32],
            normal: [0.0, 0.0, 1.0],
            color: point_colors[i],
        });
        indices.push(i as u32);
    }
    (vertices, indices)
}

/// Generate camera-facing quads for point sprite rendering.
/// Each point becomes a quad of `size` world units, facing the camera.
fn poly_data_to_point_sprites(
    poly_data: &crate::data::PolyData,
    coloring: &crate::render::Coloring,
    size: f32,
    camera_right: [f32; 3],
    camera_up: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let point_colors = mesh::resolve_colors_pub(poly_data, coloring);
    let n = poly_data.points.len();
    let half = size * 0.5;
    let mut vertices = Vec::with_capacity(n * 4);
    let mut indices = Vec::with_capacity(n * 6);

    let r = [
        camera_right[0] * half,
        camera_right[1] * half,
        camera_right[2] * half,
    ];
    let u = [
        camera_up[0] * half,
        camera_up[1] * half,
        camera_up[2] * half,
    ];
    // Normal facing camera = cross(right, up)
    let norm = [
        camera_right[1] * camera_up[2] - camera_right[2] * camera_up[1],
        camera_right[2] * camera_up[0] - camera_right[0] * camera_up[2],
        camera_right[0] * camera_up[1] - camera_right[1] * camera_up[0],
    ];

    for i in 0..n {
        let p = poly_data.points.get(i);
        let px = p[0] as f32;
        let py = p[1] as f32;
        let pz = p[2] as f32;
        let color = point_colors[i];
        let base = vertices.len() as u32;

        // 4 corners: -r-u, +r-u, +r+u, -r+u
        vertices.push(Vertex {
            position: [px - r[0] - u[0], py - r[1] - u[1], pz - r[2] - u[2]],
            normal: norm,
            color,
        });
        vertices.push(Vertex {
            position: [px + r[0] - u[0], py + r[1] - u[1], pz + r[2] - u[2]],
            normal: norm,
            color,
        });
        vertices.push(Vertex {
            position: [px + r[0] + u[0], py + r[1] + u[1], pz + r[2] + u[2]],
            normal: norm,
            color,
        });
        vertices.push(Vertex {
            position: [px - r[0] + u[0], py - r[1] + u[1], pz - r[2] + u[2]],
            normal: norm,
            color,
        });

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    (vertices, indices)
}

/// Generate wide lines as screen-aligned quads.
/// Each line segment becomes a quad of `width` world units.
fn poly_data_to_wide_lines(
    poly_data: &crate::data::PolyData,
    coloring: &crate::render::Coloring,
    width: f32,
    camera_dir: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let point_colors = mesh::resolve_colors_pub(poly_data, coloring);
    let half = width * 0.5;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for cell in poly_data.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let j = (i + 1) % nc;
            let id0 = cell[i] as usize;
            let id1 = cell[j] as usize;
            let p0 = poly_data.points.get(id0);
            let p1 = poly_data.points.get(id1);
            let p0f = [p0[0] as f32, p0[1] as f32, p0[2] as f32];
            let p1f = [p1[0] as f32, p1[1] as f32, p1[2] as f32];

            // Line direction
            let dx = p1f[0] - p0f[0];
            let dy = p1f[1] - p0f[1];
            let dz = p1f[2] - p0f[2];

            // Offset = normalize(cross(line_dir, camera_dir)) * half
            let cx = dy * camera_dir[2] - dz * camera_dir[1];
            let cy = dz * camera_dir[0] - dx * camera_dir[2];
            let cz = dx * camera_dir[1] - dy * camera_dir[0];
            let len = (cx * cx + cy * cy + cz * cz).sqrt();
            if len < 1e-10 {
                continue;
            }
            let ox = cx / len * half;
            let oy = cy / len * half;
            let oz = cz / len * half;

            let base = vertices.len() as u32;
            let c0 = point_colors[id0];
            let c1 = point_colors[id1];
            let norm = [camera_dir[0], camera_dir[1], camera_dir[2]];

            vertices.push(Vertex {
                position: [p0f[0] - ox, p0f[1] - oy, p0f[2] - oz],
                normal: norm,
                color: c0,
            });
            vertices.push(Vertex {
                position: [p0f[0] + ox, p0f[1] + oy, p0f[2] + oz],
                normal: norm,
                color: c0,
            });
            vertices.push(Vertex {
                position: [p1f[0] + ox, p1f[1] + oy, p1f[2] + oz],
                normal: norm,
                color: c1,
            });
            vertices.push(Vertex {
                position: [p1f[0] - ox, p1f[1] - oy, p1f[2] - oz],
                normal: norm,
                color: c1,
            });

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    (vertices, indices)
}

/// Project a 3D world position to NDC [0,1] screen coordinates.
/// Returns None if behind camera.
fn project_to_ndc(vp: &glam::DMat4, pos: [f64; 3]) -> Option<(f32, f32)> {
    let p = *vp * glam::DVec4::new(pos[0], pos[1], pos[2], 1.0);
    if p.w.abs() < 1e-10 || p.z / p.w < -1.0 {
        return None;
    }
    let nx = ((p.x / p.w + 1.0) * 0.5) as f32;
    let ny = ((p.y / p.w + 1.0) * 0.5) as f32;
    Some((nx, ny))
}
