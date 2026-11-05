use std::mem::size_of;

use bytemuck::{Pod, Zeroable};

use super::atlas::{Atlas, AtlasKind, AtlasTexture};

pub(super) const SCENE_LABEL: &str = "scene";
const GLOBALS_BINDING: u32 = 0;
const MASK_BINDING: u32 = 1;
const COLOR_BINDING: u32 = 2;

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
