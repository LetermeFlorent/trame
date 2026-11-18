use super::atlas::{Atlas, AtlasKind};
use super::batches::batches;
use super::clear::clear_color;
use super::draw_pipeline::{create_quad_buffer, render_pipeline};
use super::gpu::Gpu;
use super::image_bindings::ImageBindings;
use super::image_textures::ImageTextures;
use super::pipeline::{Bindings, Globals};
use super::scene::{INITIAL_QUAD_CAPACITY, Scene};
use super::slack::Slack;
use crate::theme::Rgba;

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bindings: Bindings,
    bind_group: wgpu::BindGroup,
    image_bindings: ImageBindings,
    textures: ImageTextures,
    quads: wgpu::Buffer,
    quad_capacity: usize,
    slack: Slack,
    srgb_output: bool,
}

impl Renderer {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat, atlas: &mut Atlas) -> Self {
        let device = &gpu.device;
        atlas.limit_to(device.limits().max_texture_dimension_2d);
        let bindings = Bindings::new(device, atlas);
        let image_bindings = ImageBindings::new(device, &gpu.queue);
        let layouts = [bindings.layout(), &image_bindings.layout];
        Self {
            pipeline: render_pipeline(device, layouts, format),
            bind_group: bindings.group(device),
            bindings,
            image_bindings,
            textures: ImageTextures::default(),
            quads: create_quad_buffer(device, INITIAL_QUAD_CAPACITY),
            quad_capacity: INITIAL_QUAD_CAPACITY,
            slack: Slack::default(),
            srgb_output: format.is_srgb(),
        }
    }

    pub fn sync_atlas(&mut self, gpu: &Gpu, atlas: &mut Atlas) {
        let bindings = &mut self.bindings;
        let new_mask = bindings
            .mask
            .fit(&gpu.device, &gpu.queue, atlas.size(AtlasKind::Mask));
        let new_color = bindings
            .color
            .fit(&gpu.device, &gpu.queue, atlas.size(AtlasKind::Color));
        if new_mask || new_color {
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
        self.textures
            .sync(gpu, &self.image_bindings, scene.image_uploads());
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
        self.draw(
            &mut encoder,
            target,
            scene,
            clear_color(clear, self.srgb_output),
        );
        gpu.queue.submit([encoder.finish()]);
        self.textures.end_frame();
    }

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        scene: &Scene,
        clear: wgpu::Color,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        if scene.quads().is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_bind_group(1, &self.image_bindings.empty, &[]);
        pass.set_vertex_buffer(0, self.quads.slice(..));
        for (range, serial) in batches(scene.quads().len(), scene.images()) {
            match serial.map(|serial| self.textures.group(serial)) {
                None => pass.draw(0..4, range),
                Some(Some(group)) => {
                    pass.set_bind_group(1, group, &[]);
                    pass.draw(0..4, range);
                }
                Some(None) => {}
            }
        }
    }

    fn reserve(&mut self, device: &wgpu::Device, count: usize) {
        let fits = count <= self.quad_capacity;
        if fits
            && !self
                .slack
                .shrinks(count, self.quad_capacity, INITIAL_QUAD_CAPACITY)
        {
            return;
        }
        self.quad_capacity = count.next_power_of_two().max(INITIAL_QUAD_CAPACITY);
        self.quads = create_quad_buffer(device, self.quad_capacity);
    }
}
