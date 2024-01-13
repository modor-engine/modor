#![allow(clippy::unwrap_used)]

#[macro_use]
extern crate modor;

use modor::Custom;
use modor_graphics::{Pixel, RenderTarget, TextureBuffer, TextureBufferPartUpdate};
use modor_picking::PickingBuffer;
use modor_resources::ResKey;

// TODO: add buffer.rs for simple tests with PickingBuffer
// TODO: add cameras.rs to test managed camera creation/update/deletion
// TODO: add materials.rs to test managed material creation/update/deletion (only with Default2DMaterial) + NoPicking
// TODO: add renderings.rs to test managed renderings creation/update/deletion + NoPicking
// TODO: add targets.rs to test managed targets creation/update/deletion
// TODO: add textures.rs to test managed textures creation/update/deletion + usage by material

#[derive(Component)]
struct EntityPicker {
    pixel: Pixel,
    target_key: ResKey<RenderTarget>,
    entity_id: Option<usize>,
}

#[systems]
impl EntityPicker {
    fn new(pixel: Pixel, target_key: ResKey<RenderTarget>) -> Self {
        Self {
            pixel,
            target_key,
            entity_id: None,
        }
    }

    #[run_as(action(TextureBufferPartUpdate))]
    fn register_pixel(&self, mut picking_buffer: Custom<PickingBuffer<'_>>) {
        picking_buffer.register(self.pixel, self.target_key);
    }

    #[run_after(component(TextureBuffer))]
    fn retrieve_entity(&mut self, picking_buffer: Custom<PickingBuffer<'_>>) {
        self.entity_id = picking_buffer.entity_id(self.pixel, self.target_key);
    }
}

pub mod buffer;
