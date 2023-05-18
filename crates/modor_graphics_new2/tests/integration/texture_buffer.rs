use modor::{App, BuiltEntity, EntityAssertions, EntityBuilder, EntityFilter, With};
use modor_graphics_new2::testing::wait_texture_loading;
use modor_graphics_new2::{
    Color, GraphicsModule, RenderTarget, Size, Texture, TextureBuffer, TextureSource,
};
use std::iter;

#[modor_test]
fn create_without_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(TextureBuffer::default())
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test]
fn create_without_graphics_module() {
    App::new()
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test]
fn create_with_loaded_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test]
fn create_with_target_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(target_buffer(Size::new(3, 2), Color::RED))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 0, 0, 255], Size::new(3, 2)));
}

#[modor_test]
fn add_associated_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(TextureBuffer::default())
        .updated()
        .with_component::<BufferEntity, _>(|| texture(Size::new(3, 2)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test]
fn delete_associated_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .with_deleted_components::<BufferEntity, Texture>()
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test]
fn replace_associated_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .with_component::<BufferEntity, _>(|| texture(Size::new(4, 5)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(4, 5)));
}

#[modor_test]
fn create_graphics_module() {
    App::new()
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .with_entity(modor_graphics_new2::module())
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test]
fn replace_graphics_module() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .with_entity(modor_graphics_new2::module())
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test]
fn delete_graphics_module() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated_until_all::<BufferEntity, _>(Some(100), wait_texture_loading)
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

type BufferEntity = With<TextureBuffer>;

fn buffer(size: Size) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(texture(size))
        .with(TextureBuffer::default())
}

fn target_buffer(size: Size, color: Color) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(texture(size))
        .with(TextureBuffer::default())
        .with(RenderTarget::new("TargetKey").with_background_color(color))
}

fn texture(size: Size) -> Texture {
    Texture::new("TextureKey", TextureSource::Size(size))
}

fn is_buffer_empty<F>() -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    |e| {
        e.has(|b: &TextureBuffer| {
            assert!(b.get().is_empty());
            assert_eq!(b.size(), Size::ZERO);
        })
    }
}

fn has_buffer_pixels<F>(
    color: [u8; 4],
    size: Size,
) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| {
        e.has(|b: &TextureBuffer| {
            let expected_data = iter::repeat(color)
                .take((size.width * size.height) as usize)
                .flatten()
                .collect::<Vec<_>>();
            assert_eq!(b.get(), expected_data);
            assert_eq!(b.size(), size);
        })
    }
}
