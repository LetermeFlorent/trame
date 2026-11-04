use crate::render::gpu::Gpu;

pub struct WindowSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pending: Option<(u32, u32)>,
}

pub enum Frame {
    Ready(wgpu::SurfaceTexture),
    Skip,
}

impl WindowSurface {
    pub fn new(
        gpu: &Gpu,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Option<Self> {
        let capabilities = surface.get_capabilities(&gpu.adapter);
        let mut config = surface.get_default_config(&gpu.adapter, width.max(1), height.max(1))?;
        if let Some(format) = preferred_format(&capabilities.formats) {
            config.format = format;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;
        if capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            config.alpha_mode = wgpu::CompositeAlphaMode::Opaque;
        }
        config.desired_maximum_frame_latency = 1;
        surface.configure(&gpu.device, &config);
        Some(Self {
            surface,
            config,
            pending: None,
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.pending = ((width, height) != self.size()).then_some((width, height));
    }

    #[cfg(feature = "capture")]
    pub fn allow_copy(&mut self, gpu: &Gpu) {
        let usages = self.surface.get_capabilities(&gpu.adapter).usages;
        if usages.contains(wgpu::TextureUsages::COPY_SRC) {
            self.config.usage |= wgpu::TextureUsages::COPY_SRC;
            self.surface.configure(&gpu.device, &self.config);
        }
    }

    pub fn acquire(&mut self, gpu: &Gpu) -> Frame {
        if let Some((width, height)) = self.pending.take() {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&gpu.device, &self.config);
        }
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&gpu.device, &self.config);
                frame_of(self.surface.get_current_texture())
            }
            current => frame_of(current),
        }
    }
}

fn frame_of(current: wgpu::CurrentSurfaceTexture) -> Frame {
    match current {
        wgpu::CurrentSurfaceTexture::Success(texture)
        | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Frame::Ready(texture),
        _ => Frame::Skip,
    }
}

fn preferred_format(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    let wanted = [
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ];
    wanted
        .into_iter()
        .find(|format| formats.contains(format))
        .or_else(|| formats.first().copied())
}
