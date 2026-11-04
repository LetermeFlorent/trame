use std::mem::size_of;

use bytemuck::{Pod, Zeroable};

use super::atlas::{Atlas, AtlasKind, AtlasTexture};
use super::scene::Quad;

const SCENE_LABEL: &str = "scene";
const GLOBALS_BINDING: u32 = 0;
const MASK_BINDING: u32 = 1;
const COLOR_BINDING: u32 = 2;

const QUAD_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    0 => Float32x4,
    1 => Float32x4,
    2 => Float32x4,
    3 => Float32x4,
    4 => Float32x4,
];

const QUAD_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: size_of::<Quad>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &QUAD_ATTRIBUTES,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Globals {
    pub(super) viewport: [f32; 2],
    pub(super) srgb_output: u32,
    pub(super) padding: u32,
}

pub(super) struct Bindings {
    layout: wgpu::BindGroupLayout,
    pub(super) globals: wgpu::Buffer,
    pub(super) mask: AtlasTexture,
    pub(super) color: AtlasTexture,
}

impl Bindings {
    pub(super) fn new(device: &wgpu::Device, atlas: &Atlas) -> Self {
        Self {
            layout: bind_group_layout(device),
            globals: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("globals"),
                size: size_of::<Globals>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            mask: AtlasTexture::new(device, AtlasKind::Mask, atlas.size(AtlasKind::Mask)),
            color: AtlasTexture::new(device, AtlasKind::Color, atlas.size(AtlasKind::Color)),
        }
    }

    pub(super) fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub(super) fn group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(SCENE_LABEL),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: GLOBALS_BINDING,
                    resource: self.globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: MASK_BINDING,
                    resource: wgpu::BindingResource::TextureView(self.mask.view()),
                },
                wgpu::BindGroupEntry {
                    binding: COLOR_BINDING,
                    resource: wgpu::BindingResource::TextureView(self.color.view()),
                },
            ],
        })
    }
}

fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(SCENE_LABEL),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: GLOBALS_BINDING,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            texture(MASK_BINDING),
            texture(COLOR_BINDING),
        ],
    })
}

pub(super) fn render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(SCENE_LABEL),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(SCENE_LABEL),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(SCENE_LABEL),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(QUAD_LAYOUT)],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
