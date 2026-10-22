use std::mem::size_of;

use super::atlas::{Atlas, AtlasKind};
use super::gpu::Gpu;
use super::pipeline::{self, Bindings, Globals};
use super::scene::{Quad, Scene};
use crate::color::Rgba;

const INITIAL_QUAD_CAPACITY: usize = 4096;

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bindings: Bindings,
    bind_group: wgpu::BindGroup,
    quads: wgpu::Buffer,
    quad_capacity: usize,
    srgb_output: bool,
}

impl Renderer {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat, atlas: &Atlas) -> Self {
        let device = &gpu.device;
        let bindings = Bindings::new(device, atlas);
        Self {
            pipeline: pipeline::render_pipeline(device, bindings.layout(), format),
            bind_group: bindings.group(device),
            bindings,
            quads: create_quad_buffer(device, INITIAL_QUAD_CAPACITY),
            quad_capacity: INITIAL_QUAD_CAPACITY,
            srgb_output: format.is_srgb(),
        }
    }

    pub fn sync_atlas(&mut self, gpu: &Gpu, atlas: &mut Atlas) {
        let bindings = &mut self.bindings;
        let grew_mask = bindings
            .mask
            .grow(&gpu.device, &gpu.queue, atlas.size(AtlasKind::Mask));
        let grew_color = bindings
            .color
            .grow(&gpu.device, &gpu.queue, atlas.size(AtlasKind::Color));
        if grew_mask || grew_color {
            self.bind_group = bindings.group(&gpu.device);
        }
        for upload in atlas.take_uploads() {
            match upload.region.kind {
                AtlasKind::Mask => bindings.mask.write(&gpu.queue, &upload),
                AtlasKind::Color => bindings.color.write(&gpu.queue, &upload),
            }
        }
    }

    pub fn render(&mut self, gpu: &Gpu, target: &wgpu::TextureView, scene: &Scene, clear: Rgba) {
        let globals = Globals {
            viewport: [scene.width().max(1.0), scene.height().max(1.0)],
            srgb_output: u32::from(self.srgb_output),
            padding: 0,
        };
        gpu.queue
            .write_buffer(&self.bindings.globals, 0, bytemuck::bytes_of(&globals));
        let quads = scene.quads();
        self.reserve(&gpu.device, quads.len());
        if !quads.is_empty() {
            gpu.queue
                .write_buffer(&self.quads, 0, bytemuck::cast_slice(quads));
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color(clear)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            if !quads.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.quads.slice(..));
                pass.draw(0..4, 0..quads.len() as u32);
            }
        }
        gpu.queue.submit([encoder.finish()]);
    }

    fn reserve(&mut self, device: &wgpu::Device, count: usize) {
        if count <= self.quad_capacity {
            return;
        }
        self.quad_capacity = count.next_power_of_two();
        self.quads = create_quad_buffer(device, self.quad_capacity);
    }

    fn clear_color(&self, color: Rgba) -> wgpu::Color {
        let channel = |value: f32| {
            let value = f64::from(value);
            if self.srgb_output {
                srgb_to_linear(value)
            } else {
                value
            }
        };
        wgpu::Color {
            r: channel(color.r),
            g: channel(color.g),
            b: channel(color.b),
            a: f64::from(color.a),
        }
    }
}

fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn create_quad_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quads"),
        size: (capacity * size_of::<Quad>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
