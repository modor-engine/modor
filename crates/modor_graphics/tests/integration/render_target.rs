use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::is_same;
use modor_graphics::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::{Resource, ResourceState};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(main_texture().with(RenderTarget::new(TargetKey::Main)))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#black"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_invalid_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(
            EntityBuilder::new()
                .with(Texture::from_path(TargetTextureKey::Main, "invalid.png"))
                .with(RenderTarget::new(TargetKey::Main)),
        )
        .updated()
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert!(matches!(t.state(), ResourceState::Loading)))
        })
        .updated_until_all::<With<Texture>, Texture>(Some(100), wait_resource_loading)
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert!(matches!(t.state(), ResourceState::Error(_))))
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_background_color() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(
            main_texture()
                .with(RenderTarget::new(TargetKey::Main).with_background_color(Color::RED)),
        )
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#red"))
        .with_update::<With<RenderTarget>, _>(|t: &mut RenderTarget| {
            t.background_color = Color::BLACK;
        })
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#black"));
}

#[modor_test(disabled(macos, android, wasm))]
fn resize_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(
            main_texture()
                .with(RenderTarget::new(TargetKey::Main).with_background_color(Color::RED)),
        )
        .updated()
        .with_component::<With<RenderTarget>, _>(|| {
            Texture::from_size(TargetTextureKey::Main, Size::new(20, 30))
        })
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#red2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn recreate_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(
            main_texture()
                .with(RenderTarget::new(TargetKey::Main).with_background_color(Color::RED)),
        )
        .updated()
        .with_deleted_components::<With<MainTarget>, Texture>()
        .updated()
        .with_component::<With<RenderTarget>, _>(|| {
            Texture::from_size(TargetTextureKey::Main, Size::new(20, 30))
        })
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#red2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn render_target_in_target() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resource())
        .with_entity(main_texture().with(RenderTarget::new(TargetKey::Main)))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#target_in_target"))
        .with_component::<With<BlueRectangle>, _>(|| {
            Model::rectangle(MaterialKey::Target, CameraKey::Secondary)
        })
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#target_in_use"))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#target_in_use"));
}

fn main_texture() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::from_size(
            TargetTextureKey::Main,
            Size::new(30, 20),
        ))
        .with(TextureBuffer::default())
        .with(MainTarget)
}

fn resource() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(Camera2D::new(CameraKey::Main, TargetKey::Main))
        .with_child(Camera2D::new(CameraKey::Secondary, TargetKey::Secondary))
        .with_child(Material::new(MaterialKey::Rectangle).with_color(Color::BLUE))
        .with_child(
            Material::new(MaterialKey::Target).with_texture_key(TargetTextureKey::Secondary),
        )
        .with_child(secondary_target())
        .with_child(blue_rectangle())
        .with_child(target_rectangle())
}

fn secondary_target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::from_size(
            TargetTextureKey::Secondary,
            Size::new(20, 50),
        ))
        .with(TextureBuffer::default())
        .with(RenderTarget::new(TargetKey::Secondary).with_background_color(Color::GREEN))
        .with(SecondaryTarget)
}

fn blue_rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::ONE * 0.25)
                .with_size(Vec2::ONE * 0.5),
        )
        .with(Model::rectangle(
            MaterialKey::Rectangle,
            CameraKey::Secondary,
        ))
        .with(BlueRectangle)
}

fn target_rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(Model::rectangle(MaterialKey::Target, CameraKey::Main))
}

#[derive(SingletonComponent, NoSystem)]
struct MainTarget;

#[derive(SingletonComponent, NoSystem)]
struct SecondaryTarget;

#[derive(SingletonComponent, NoSystem)]
struct BlueRectangle;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TargetKey {
    Main,
    Secondary,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TargetTextureKey {
    Main,
    Secondary,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum CameraKey {
    Main,
    Secondary,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MaterialKey {
    Target,
    Rectangle,
}
