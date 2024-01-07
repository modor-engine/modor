use crate::components::managed_targets::ManagedTargets;
use modor::{Query, SingleRef};
use modor_graphics::{Pixel, RenderTarget, TextureBuffer, TextureBufferPart};
use modor_resources::{ResKey, ResourceRegistry};

/// A system parameter to access picking buffer of a `RenderTarget` of
/// [`MAIN_RENDERING`](modor_graphics::MAIN_RENDERING) category.
///
/// Note that when a target has both a [`Window`] and a [`Texture`], the [`Window`] is tracked in priority
/// by the picking module.
///
/// # Requirements
///
/// The buffer is effective only if:
/// - picking [`module`](crate::module) is initialized
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// # use modor_picking::*;
/// #
/// #[derive(Component)]
/// struct EntityPicker {
///     pixel: Pixel,
///     entity_id: Option<usize>,
/// }
///
/// #[systems]
/// impl EntityPicker {
///     #[run_as(action(TextureBufferPartUpdate))]
///     fn register_pixel(&mut self, mut picking_buffer: Custom<PickingBuffer<'_>>) {
///         picking_buffer.register(self.pixel, WINDOW_TARGET);
///     }
///
///     #[run_after(component(TextureBuffer))]
///     fn retrieve_entity(&mut self, mut picking_buffer: Custom<PickingBuffer<'_>>) {
///         self.entity_id = picking_buffer.entity_id(self.pixel, WINDOW_TARGET);
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct PickingBuffer<'a> {
    managed_targets: SingleRef<'a, 'static, ManagedTargets>,
    target_registry: SingleRef<'a, 'static, ResourceRegistry<RenderTarget>>,
    target_buffers: Query<'a, &'static mut TextureBuffer>,
}

impl PickingBuffer<'_> {
    /// Register pixel coordinates of `target_key` to track for the current app update.
    ///
    /// This method should be called during [`TextureBufferPartUpdate`](modor_graphics::TextureBufferPartUpdate) action.
    pub fn register(&mut self, pixel: Pixel, target_key: ResKey<RenderTarget>) {
        if let Some(entity_id) = self.target_id(target_key) {
            if let Some(buffer) = self.target_buffers.get_mut(entity_id) {
                if let TextureBufferPart::Pixels(pixels) = &mut buffer.part {
                    pixels.push(pixel);
                }
            }
        }
    }

    /// Returns the entity ID rendered at the provided `pixel` coordinates of `target_key`.
    ///
    /// The entity ID is returned on if:
    /// - The target is tracked by the picking module.
    /// - The provided `pixel` coordinates have been registered with [`PickingBuffer::register`].
    /// - An entity is rendered at provided `pixel` coordinates.
    ///
    /// This method should be called after [`TextureBuffer`] systems have been run.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn entity_id(&self, pixel: Pixel, target_key: ResKey<RenderTarget>) -> Option<usize> {
        let entity_id = self.target_id(target_key)?;
        let buffer = self.target_buffers.get(entity_id)?;
        let color = buffer.pixel(pixel)?;
        let color_array: [u8; 4] = [
            (Self::srgb_to_rgb(color.r) * 255.).round() as u8,
            (Self::srgb_to_rgb(color.g) * 255.).round() as u8,
            (Self::srgb_to_rgb(color.b) * 255.).round() as u8,
            (color.a * 255.) as u8,
        ];
        let entity_id: &[u32] = bytemuck::cast_slice(&color_array);
        if entity_id[0] < u32::MAX {
            Some(entity_id[0] as usize)
        } else {
            None
        }
    }

    fn target_id(&self, target_key: ResKey<RenderTarget>) -> Option<usize> {
        let managed_target_key = self
            .managed_targets
            .get()
            .resources
            .managed_key(target_key)?;
        self.target_registry.get().entity_id(managed_target_key)
    }

    fn srgb_to_rgb(component: f32) -> f32 {
        if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    }
}
