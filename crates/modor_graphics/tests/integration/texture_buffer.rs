use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{Color, GraphicsModule, RenderTarget, Size, Texture, TextureBuffer};
use modor_resources::ResKey;
use std::iter;

#[modor_test(disabled(macos, android, wasm))]
fn create_without_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(TextureBuffer::default())
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_loaded_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_target_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(target_buffer(Size::new(3, 2), Color::RED))
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 0, 0, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn add_associated_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(TextureBuffer::default())
        .updated()
        .with_component::<BufferEntity, _>(|| Texture::from_size(TEXTURE, Size::new(3, 2)))
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_associated_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .with_deleted_components::<BufferEntity, Texture>()
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_associated_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .with_component::<BufferEntity, _>(|| Texture::from_size(TEXTURE, Size::new(4, 5)))
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(4, 5)));
}

#[modor_test]
fn create_without_graphics_module() {
    App::new()
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test(disabled(macos, android, wasm))]
fn create_graphics_module() {
    App::new()
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_graphics_module() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer(Size::new(3, 2)))
        .updated()
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty())
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

assertion_functions!(
    fn is_buffer_empty(buffer: &TextureBuffer) {
        assert!(buffer.get().is_empty());
        assert_eq!(buffer.size(), Size::ZERO);
    }

    fn has_buffer_pixels(buffer: &TextureBuffer, color: [u8; 4], size: Size) {
        let expected_data = iter::repeat(color)
            .take((size.width * size.height) as usize)
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(buffer.get(), expected_data);
        assert_eq!(buffer.size(), size);
    }
);

fn buffer(size: Size) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Texture::from_size(TEXTURE, size))
        .component(TextureBuffer::default())
}

fn target_buffer(size: Size, color: Color) -> impl BuiltEntity {
    let target_key = ResKey::unique("main");
    EntityBuilder::new()
        .component(Texture::from_size(TEXTURE, size))
        .component(TextureBuffer::default())
        .component(RenderTarget::new(target_key))
        .with(|t| t.background_color = color)
}

type BufferEntity = With<TextureBuffer>;

const TEXTURE: ResKey<Texture> = ResKey::new("main");
