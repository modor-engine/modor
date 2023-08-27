use modor::{App, BuiltEntity, EntityAssertions, EntityBuilder, EntityFilter, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    texture_target, Material, Model, Size, Texture, TextureBuffer, TextureSource,
    TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::{ResKey, Resource, ResourceLoadingError, ResourceState};

const TEXTURE_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/opaque-texture.png"
));

#[modor_test(disabled(macos, android, wasm))]
fn create_from_size() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_size(RECTANGLE_TEXTURE, Size::new(40, 20))))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("texture#size"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(40, 20)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_zero_size() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_size(RECTANGLE_TEXTURE, Size::ZERO)))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("texture#zero"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(1, 1)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_buffer() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_buffer(
            RECTANGLE_TEXTURE,
            Size::new(3, 1),
            vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255],
        )))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("texture#buffer"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(3, 1)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_empty_buffer() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_buffer(
            RECTANGLE_TEXTURE,
            Size::ZERO,
            vec![],
        )))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("texture#zero"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(1, 1)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_file() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_file(RECTANGLE_TEXTURE, TEXTURE_DATA)))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_loading())
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("texture#file"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(4, 4)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_path() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_path(
            RECTANGLE_TEXTURE,
            "../tests/assets/opaque-texture.png",
        )))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_loading())
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("texture#file"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(4, 4)));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_unsupported_format() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_path(
            RECTANGLE_TEXTURE,
            "../tests/assets/text.txt",
        )))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, assert_invalid_format());
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_corrupted_file() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_path(
            RECTANGLE_TEXTURE,
            "../tests/assets/corrupted-texture.png",
        )))
        .assert::<With<TextureBuffer>>(1, assert_not_loaded())
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, assert_invalid_format());
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_buffer_with_too_big_size() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_buffer(
            RECTANGLE_TEXTURE,
            Size::new(4, 1),
            vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255],
        )))
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_invalid_format());
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_default_params() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(rectangle())
        .with_entity(Texture::from_file(RECTANGLE_TEXTURE, TEXTURE_DATA))
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_component_diff("texture#render_default", 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_smooth() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(rectangle())
        .with_entity(
            EntityBuilder::new()
                .component(Texture::from_file(RECTANGLE_TEXTURE, TEXTURE_DATA))
                .with(|t| t.is_smooth = false)
                .component(TestedTexture),
        )
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("texture#render_not_smooth"))
        .with_update::<With<TestedTexture>, _>(|t: &mut Texture| t.is_smooth = true)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("texture#render_default", 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_repeated() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(rectangle())
        .with_entity(
            EntityBuilder::new()
                .component(Texture::from_file(RECTANGLE_TEXTURE, TEXTURE_DATA))
                .with(|t| t.is_repeated = true)
                .component(TestedTexture),
        )
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_component_diff("texture#render_repeated", 1))
        .with_update::<With<TestedTexture>, _>(|t: &mut Texture| t.is_repeated = false)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("texture#render_default", 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn set_source() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(buffer().component(Texture::from_buffer(
            RECTANGLE_TEXTURE,
            Size::new(3, 1),
            vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255],
        )))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("texture#buffer"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(3, 1)))
        .with_update::<With<TextureBuffer>, _>(|t: &mut Texture| {
            t.set_source(TextureSource::File(TEXTURE_DATA));
        })
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(3, 1)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("texture#buffer"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(3, 1)))
        .updated_until_any::<With<TextureBuffer>, _>(Some(100), |t: &Texture| {
            t.size() == Some(Size::new(4, 4))
        })
        .assert::<With<TextureBuffer>>(1, is_same("texture#file"))
        .assert::<With<TextureBuffer>>(1, assert_loaded(Size::new(4, 4)));
}

fn assert_not_loaded<F>() -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| {
        e.has(|t: &Texture| {
            assert_eq!(t.size(), None);
            assert_eq!(t.state(), ResourceState::NotLoaded);
        })
    }
}

fn assert_loaded<F>(size: Size) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| {
        e.has(|t: &Texture| {
            assert_eq!(t.size(), Some(size));
            assert_eq!(t.state(), ResourceState::Loaded);
        })
    }
}

fn assert_loading<F>() -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| {
        e.has(|t: &Texture| {
            assert_eq!(t.size(), None);
            assert_eq!(t.state(), ResourceState::Loading);
        })
    }
}

fn assert_invalid_format<F>() -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| {
        e.has(|t: &Texture| {
            assert_eq!(t.size(), None);
            assert!(matches!(
                t.state(),
                ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
            ));
        })
    }
}

fn buffer() -> impl BuiltEntity {
    EntityBuilder::new().component(TextureBuffer::default())
}

fn rectangle() -> impl BuiltEntity {
    let material_key = ResKey::unique("rectangle");
    EntityBuilder::new()
        .component(Transform2D::new())
        .component(Model::rectangle(material_key, TEXTURE_CAMERAS_2D.get(0)))
        .component(Material::new(material_key))
        .with(|m| m.texture_key = Some(RECTANGLE_TEXTURE))
        .with(|m| m.texture_size = Vec2::ONE * 2.)
}

#[derive(Component, NoSystem)]
struct TestedTexture;

const RECTANGLE_TEXTURE: ResKey<Texture> = ResKey::new("rectangle");
