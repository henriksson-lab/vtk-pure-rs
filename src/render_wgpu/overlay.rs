use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::render::{AxesWidget, ScalarBar};

use crate::render_wgpu::bitmap_font;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct OverlayVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl OverlayVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OverlayVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct OverlayPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl OverlayPipeline {
    /// Access the underlying render pipeline (for annotation rendering).
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

impl OverlayPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("overlay_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_overlay"),
                buffers: &[OverlayVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_overlay"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self { pipeline }
    }

    /// Render scalar bars as 2D overlays on top of the 3D scene.
    pub fn render_scalar_bars(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        scalar_bars: &[ScalarBar],
    ) {
        if scalar_bars.is_empty() {
            return;
        }

        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for bar in scalar_bars {
            // Background quad
            let bg = bar.background_color;
            let pad = 0.01f32;
            let bx = bar.position[0] - pad;
            let by = bar.position[1] - pad;
            let bw = bar.size[0] + 2.0 * pad + 0.06; // extra space for labels
            let bh = bar.size[1] + 2.0 * pad + 0.04; // extra space for title
            push_quad(&mut vertices, &mut indices, bx, by, bw, bh, bg);

            // Color band quads
            for (pos, size, color) in bar.color_band_quads() {
                push_quad(
                    &mut vertices,
                    &mut indices,
                    pos[0],
                    pos[1],
                    size[0],
                    size[1],
                    color,
                );
            }

            // Tick marks and labels
            let text_color = [bar.text_color[0], bar.text_color[1], bar.text_color[2], 1.0];
            for (label_pos, text) in bar.label_info() {
                // Tick mark
                push_quad(
                    &mut vertices,
                    &mut indices,
                    label_pos[0] - 0.005,
                    label_pos[1] - 0.001,
                    0.004,
                    0.002,
                    text_color,
                );
                // Label text
                bitmap_font::render_text(
                    &mut vertices,
                    &mut indices,
                    &text,
                    label_pos[0] + 0.002,
                    label_pos[1] - 0.006,
                    0.012,
                    text_color,
                );
            }

            // Title text
            let title_pos = bar.title_position();
            bitmap_font::render_text(
                &mut vertices,
                &mut indices,
                &bar.title,
                title_pos[0],
                title_pos[1],
                0.014,
                text_color,
            );
        }

        if vertices.is_empty() {
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("overlay vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("overlay index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve the 3D scene
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    }

    /// Render an orientation axes widget as 2D overlay.
    pub fn render_axes_widget(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        widget: &AxesWidget,
        view_matrix: &[[f64; 4]; 4],
    ) {
        if !widget.enabled {
            return;
        }

        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Background circle (approximate with a filled quad)
        let bg_size = widget.size * 1.3;
        push_quad(
            &mut vertices,
            &mut indices,
            widget.position[0] - bg_size,
            widget.position[1] - bg_size,
            bg_size * 2.0,
            bg_size * 2.0,
            [0.0, 0.0, 0.0, 0.4],
        );

        let axes = widget.projected_axes(view_matrix);

        for (tip, color, label) in &axes {
            let cx = widget.position[0];
            let cy = widget.position[1];
            let rgba = [color[0], color[1], color[2], 1.0];

            // Draw axis line as a thin quad
            let dx = tip[0] - cx;
            let dy = tip[1] - cy;
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-6 {
                continue;
            }
            let nx = -dy / len * 0.003; // perpendicular half-width
            let ny = dx / len * 0.003;

            let base = vertices.len() as u32;
            vertices.push(OverlayVertex {
                position: [cx - nx, cy - ny],
                color: rgba,
            });
            vertices.push(OverlayVertex {
                position: [cx + nx, cy + ny],
                color: rgba,
            });
            vertices.push(OverlayVertex {
                position: [tip[0] + nx, tip[1] + ny],
                color: rgba,
            });
            vertices.push(OverlayVertex {
                position: [tip[0] - nx, tip[1] - ny],
                color: rgba,
            });
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

            // Arrowhead
            let arrow_size = 0.012;
            push_quad(
                &mut vertices,
                &mut indices,
                tip[0] - arrow_size * 0.5,
                tip[1] - arrow_size * 0.5,
                arrow_size,
                arrow_size,
                rgba,
            );

            // Label
            if widget.show_labels {
                let label_str = label.to_string();
                let label_x = tip[0] + dx / len * 0.015;
                let label_y = tip[1] + dy / len * 0.015 - 0.005;
                bitmap_font::render_text(
                    &mut vertices,
                    &mut indices,
                    &label_str,
                    label_x,
                    label_y,
                    0.012,
                    rgba,
                );
            }
        }

        if vertices.is_empty() {
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("axes vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("axes index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("axes overlay pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    }
}

pub fn push_quad(
    vertices: &mut Vec<OverlayVertex>,
    indices: &mut Vec<u32>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
) {
    let base = vertices.len() as u32;
    vertices.push(OverlayVertex {
        position: [x, y],
        color,
    });
    vertices.push(OverlayVertex {
        position: [x + w, y],
        color,
    });
    vertices.push(OverlayVertex {
        position: [x + w, y + h],
        color,
    });
    vertices.push(OverlayVertex {
        position: [x, y + h],
        color,
    });
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
