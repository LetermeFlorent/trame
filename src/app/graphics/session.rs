use std::sync::Arc;

use winit::window::Window;

use super::capture::Capture;
use trame::render::{
    Atlas, Frame, Gpu, ReadbackError, Renderer, Scene, WindowSurface, render_in_format,
};
use trame::theme::Rgba;

pub(super) struct Session<S> {
    pub(super) gpu: Gpu,
    pub(super) surface: S,
    pub(super) format: wgpu::TextureFormat,
    pub(super) renderer: Renderer,
    pub(super) software: bool,
}

pub(super) struct RenderFrame<'a> {
    pub(super) scene: &'a Scene,
    pub(super) atlas: &'a mut Atlas,
    pub(super) clear: Rgba,
}

pub(super) enum FrameOutcome {
    Presented,
    Skipped,
    Lost,
}

pub(super) trait FrameTexture {
    fn texture(&self) -> &wgpu::Texture;
}

impl FrameTexture for wgpu::SurfaceTexture {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

impl<S> Session<S> {
    pub(super) fn needs_recovery(&self) -> bool {
        self.gpu.is_lost() || self.gpu.take_out_of_memory()
    }

    pub(super) fn render_offscreen(
        &mut self,
        scene: &Scene,
        atlas: &mut Atlas,
    ) -> Result<Vec<u8>, ReadbackError> {
        self.render_offscreen_with(scene, atlas, render_in_format)
    }

    pub(super) fn render_offscreen_with(
        &mut self,
        scene: &Scene,
        atlas: &mut Atlas,
        render: impl FnOnce(
            &Gpu,
            &mut Renderer,
            &Scene,
            Rgba,
            wgpu::TextureFormat,
        ) -> Result<Vec<u8>, ReadbackError>,
    ) -> Result<Vec<u8>, ReadbackError> {
        self.renderer.sync_atlas(&self.gpu, atlas);
        if self.needs_recovery() {
            return Err(ReadbackError::Lost);
        }
        let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
        let result = render(&self.gpu, &mut self.renderer, scene, clear, self.format);
        if self.needs_recovery() {
            return Err(ReadbackError::Lost);
        }
        result
    }

    pub(super) fn present_with<T: FrameTexture>(
        &mut self,
        atlas: &mut Atlas,
        texture: T,
        render: impl FnOnce(&Gpu, &mut Renderer, &wgpu::TextureView),
        present: impl FnOnce(&Gpu, T),
    ) -> FrameOutcome {
        self.renderer.sync_atlas(&self.gpu, atlas);
        if self.needs_recovery() {
            drop(texture);
            return FrameOutcome::Lost;
        }
        let view = texture.texture().create_view(&Default::default());
        render(&self.gpu, &mut self.renderer, &view);
        if self.needs_recovery() {
            drop(view);
            drop(texture);
            return FrameOutcome::Lost;
        }
        present(&self.gpu, texture);
        if self.needs_recovery() {
            FrameOutcome::Lost
        } else {
            FrameOutcome::Presented
        }
    }
}

impl Session<WindowSurface> {
    pub(super) fn acquire_and_present(
        &mut self,
        window: &Arc<Window>,
        frame: RenderFrame<'_>,
        capture: Option<&Capture>,
    ) -> FrameOutcome {
        let texture = match self.surface.acquire(&self.gpu, |instance| {
            instance.create_surface(window.clone())
        }) {
            Frame::Ready(texture) => texture,
            Frame::Skip => return FrameOutcome::Skipped,
            Frame::DeviceLost => return FrameOutcome::Lost,
        };
        let (scene, clear) = (frame.scene, frame.clear);
        self.present_with(
            frame.atlas,
            texture,
            |gpu, renderer, view| renderer.render(gpu, view, scene, clear),
            |gpu, texture| {
                if let Some(capture) = capture {
                    capture.save(gpu, &texture.texture);
                }
                window.pre_present_notify();
                gpu.queue.present(texture);
            },
        )
    }
}
