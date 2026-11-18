use super::image_textures::create_texture;

const LABEL: &str = "image";
const TEXTURE_BINDING: u32 = 0;
const SAMPLER_BINDING: u32 = 1;

pub(super) struct ImageBindings {
    pub(super) layout: wgpu::BindGroupLayout,
    pub(super) sampler: wgpu::Sampler,
    pub(super) empty: wgpu::BindGroup,
}

impl ImageBindings {
    pub(super) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let layout = layout(device);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(LABEL),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texture = create_texture(device, queue, (1, 1), &[0; 4]);
        let empty = group(device, &layout, &sampler, &texture);
        Self {
            layout,
            sampler,
            empty,
        }
    }

    pub(super) fn group(&self, device: &wgpu::Device, texture: &wgpu::Texture) -> wgpu::BindGroup {
        group(device, &self.layout, &self.sampler, texture)
    }
}

fn group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(LABEL),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: TEXTURE_BINDING,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(LABEL),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: TEXTURE_BINDING,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: SAMPLER_BINDING,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}
