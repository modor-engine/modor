use crate::Picking;
use modor::{Query, SingleRef};
use modor_graphics::{Pixel, RenderTarget, TextureBuffer, WINDOW_TARGET};
use modor_resources::{ResKey, ResourceRegistry};

#[derive(SystemParam)]
pub struct PickingBuffer<'a> {
    picking: SingleRef<'a, 'static, Picking>,
    target_registry: SingleRef<'a, 'static, ResourceRegistry<RenderTarget>>,
    target_buffers: Query<'a, &'static mut TextureBuffer>,
}

impl PickingBuffer<'_> {
    pub fn generated_buffer(
        &mut self,
        target_key: ResKey<RenderTarget>,
    ) -> Option<&mut TextureBuffer> {
        self.picking.get().generated_buffer(
            target_key,
            self.target_registry.get(),
            &mut self.target_buffers,
        )
    }

    pub fn picked_entity_id(
        &mut self,
        pixel: Pixel,
        target_key: ResKey<RenderTarget>,
    ) -> Option<usize> {
        self.picking.get().picked_entity_id(
            pixel,
            target_key,
            self.target_registry.get(),
            &mut self.target_buffers,
        )
    }
}
