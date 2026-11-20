use super::TextSystem;
use super::source::{DecodedPicture, ImageSource, SlotTarget};
use crate::render::{AtlasKind, AtlasRegion};

impl TextSystem {
    pub fn hover_image(
        &mut self,
        job: u64,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Option<AtlasRegion> {
        let expected = usize::try_from(width)
            .ok()
            .and_then(|w| w.checked_mul(usize::try_from(height).ok()?))
            .and_then(|px| px.checked_mul(4))?;
        if width == 0 || height == 0 || pixels.len() != expected {
            return None;
        }
        if let Some((cached, region)) = self.images.hover
            && cached == job
            && (region.width, region.height) == (width, height)
        {
            return Some(region);
        }
        let region = self
            .atlas
            .insert(AtlasKind::Color, width, height, pixels.to_vec())?;
        self.images.hover = Some((job, region));
        Some(region)
    }

    pub fn request_picture(&mut self, job: u64, limit: (u32, u32), source: impl Into<ImageSource>) {
        self.images
            .request_slot(SlotTarget::Picture, job, limit, source.into());
    }

    pub fn cancel_picture(&mut self) {
        self.images.picture = None;
        self.images.worker.cancel_slot(SlotTarget::Picture);
    }

    pub fn has_picture_job(&self, job: u64) -> bool {
        self.images.worker.has_slot_job(SlotTarget::Picture, job)
    }

    pub fn take_picture(&mut self, job: u64) -> Option<Option<DecodedPicture>> {
        self.receive_images();
        let answer = self.images.picture.take()?;
        (answer.job == job).then_some(answer.image)
    }

    pub fn request_preview(&mut self, job: u64, limit: (u32, u32), source: impl Into<ImageSource>) {
        self.images
            .request_slot(SlotTarget::Preview, job, limit, source.into());
    }

    pub fn cancel_preview(&mut self) {
        self.images.preview = None;
        self.images.worker.cancel_slot(SlotTarget::Preview);
    }

    pub fn take_preview(&mut self, job: u64) -> Option<Option<DecodedPicture>> {
        self.receive_images();
        let answer = self.images.preview.take()?;
        (answer.job == job).then_some(answer.image)
    }
}
