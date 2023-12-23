use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{
    Color, GraphicsModule, Pixel, RenderTarget, Size, Texture, TextureBuffer, TextureBufferPart,
};
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
        .with_entity(white_buffer(Size::new(3, 2)))
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
fn create_buffer_with_specific_pixels() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer(
            Size::new(3, 2),
            vec![
                255, 0, 0, 255, // 0, 0
                0, 255, 0, 255, // 1, 0
                0, 0, 255, 255, // 2, 0
                255, 255, 0, 255, // 0, 1
                0, 255, 255, 255, // 1, 1
                255, 0, 255, 255, // 2, 1
            ],
        ))
        .with_update::<(), _>(|b: &mut TextureBuffer| {
            b.part = TextureBufferPart::Pixels(vec![
                Pixel::new(1, 0),
                Pixel::new(2, 1),
                Pixel::new(3, 0),
                Pixel::new(0, 2),
            ]);
        })
        .updated()
        .assert::<BufferEntity>(1, has_pixel(0, 0, None))
        .assert::<BufferEntity>(1, has_pixel(1, 0, Some(Color::rgba(0., 1., 0., 1.))))
        .assert::<BufferEntity>(1, has_pixel(2, 0, None))
        .assert::<BufferEntity>(1, has_pixel(0, 1, None))
        .assert::<BufferEntity>(1, has_pixel(1, 1, None))
        .assert::<BufferEntity>(1, has_pixel(2, 1, Some(Color::rgba(1., 0., 1., 1.))))
        .assert::<BufferEntity>(1, has_pixel(3, 0, None))
        .assert::<BufferEntity>(1, has_pixel(0, 2, None));
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
        .with_entity(white_buffer(Size::new(3, 2)))
        .updated()
        .with_deleted_components::<BufferEntity, Texture>()
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_associated_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(white_buffer(Size::new(3, 2)))
        .updated()
        .with_component::<BufferEntity, _>(|| Texture::from_size(TEXTURE, Size::new(4, 5)))
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(4, 5)));
}

#[modor_test]
fn create_without_graphics_module() {
    App::new()
        .with_entity(white_buffer(Size::new(3, 2)))
        .updated()
        .assert::<BufferEntity>(1, is_buffer_empty());
}

#[modor_test(disabled(macos, android, wasm))]
fn create_graphics_module() {
    App::new()
        .with_entity(white_buffer(Size::new(3, 2)))
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_graphics_module() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(white_buffer(Size::new(3, 2)))
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<BufferEntity>(1, has_buffer_pixels([255, 255, 255, 255], Size::new(3, 2)));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(white_buffer(Size::new(3, 2)))
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
        assert_eq!(buffer.pixel(Pixel::new(0, 0)), None);
    }

    fn has_buffer_pixels(buffer: &TextureBuffer, color: [u8; 4], size: Size) {
        let expected_data = iter::repeat(color)
            .take((size.width * size.height) as usize)
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(buffer.get(), expected_data);
        assert_eq!(buffer.size(), size);
        assert_eq!(
            buffer.pixel(Pixel::new(0, 0)),
            Some(Color::rgba(
                f32::from(color[0]) / 255.,
                f32::from(color[1]) / 255.,
                f32::from(color[2]) / 255.,
                f32::from(color[3]) / 255.
            ))
        );
        assert_eq!(buffer.pixel(Pixel::new(size.width, 0)), None);
        assert_eq!(buffer.pixel(Pixel::new(0, size.height)), None);
    }

    fn has_pixel(buffer: &TextureBuffer, x: u32, y: u32, color: Option<Color>) {
        assert_eq!(buffer.pixel(Pixel::new(x, y)), color);
    }
);

fn white_buffer(size: Size) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Texture::from_size(TEXTURE, size))
        .component(TextureBuffer::default())
}

fn buffer(size: Size, data: Vec<u8>) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Texture::from_buffer(TEXTURE, size, data))
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
