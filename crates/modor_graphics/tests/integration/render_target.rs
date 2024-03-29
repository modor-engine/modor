use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::is_same;
use modor_graphics::{
    instance_2d, material, texture_target, window_target, Camera2D, Color, Default2DMaterial,
    InstanceRendering2D, Material, RenderTarget, Size, Texture, TextureBuffer, Window,
    TARGET_TEXTURES, TEXTURE_CAMERAS_2D, TEXTURE_TARGETS, WINDOW_CAMERA_2D, WINDOW_TARGET,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::{ResKey, Resource, ResourceState};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(main_texture().component(RenderTarget::new(MAIN_TARGET)))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#black"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_invalid_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(
            EntityBuilder::new()
                .component(Texture::from_path(MAIN_TARGET_TEXTURE, "invalid.png"))
                .component(RenderTarget::new(MAIN_TARGET)),
        )
        .updated()
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert!(matches!(t.state(), ResourceState::Loading)))
        })
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert!(matches!(t.state(), ResourceState::Error(_))))
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_background_color() {
    let mut target = RenderTarget::new(MAIN_TARGET);
    target.background_color = Color::RED;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(main_texture().component(target))
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
    let mut target = RenderTarget::new(MAIN_TARGET);
    target.background_color = Color::RED;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(main_texture().component(target))
        .updated()
        .with_component::<With<RenderTarget>, _>(|| {
            Texture::from_size(MAIN_TARGET_TEXTURE, Size::new(20, 30))
        })
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#red2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn recreate_texture() {
    let mut target = RenderTarget::new(MAIN_TARGET);
    target.background_color = Color::RED;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(main_texture().component(target))
        .updated()
        .with_deleted_components::<With<MainTarget>, Texture>()
        .updated()
        .with_component::<With<RenderTarget>, _>(|| {
            Texture::from_size(MAIN_TARGET_TEXTURE, Size::new(20, 30))
        })
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#red2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn render_target_in_target() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resource())
        .with_entity(main_texture().component(RenderTarget::new(MAIN_TARGET)))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#target_in_target"))
        .with_entity(blue_rectangle_with_material(TARGET_MATERIAL))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#target_in_use"))
        .updated()
        .assert::<With<MainTarget>>(1, is_same("render_target#target_in_use"));
}

#[modor_test]
fn create_window_target_entity() {
    App::new()
        .with_entity(window_target())
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert_eq!(t.key(), WINDOW_TARGET))
                .has(|_: &Window| ())
                .has(|c: &Camera2D| assert_eq!(c.key(), WINDOW_CAMERA_2D))
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn create_texture_target_entity_without_buffer() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(5, Size::new(30, 20), false))
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert_eq!(t.key(), TEXTURE_TARGETS.get(5)))
                .has(|t: &Texture| assert_eq!(t.size(), Some(Size::new(30, 20))))
                .has(|t: &Texture| assert_eq!(t.key(), TARGET_TEXTURES.get(5)))
                .has(|c: &Camera2D| assert_eq!(c.key(), TEXTURE_CAMERAS_2D.get(5)))
                .has_not::<TextureBuffer>()
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn create_texture_target_entity_with_buffer() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(5, Size::new(30, 20), true))
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .assert::<With<RenderTarget>>(1, |e| {
            e.has(|t: &RenderTarget| assert_eq!(t.key(), TEXTURE_TARGETS.get(5)))
                .has(|t: &Texture| assert_eq!(t.size(), Some(Size::new(30, 20))))
                .has(|t: &Texture| assert_eq!(t.key(), TARGET_TEXTURES.get(5)))
                .has(|c: &Camera2D| assert_eq!(c.key(), TEXTURE_CAMERAS_2D.get(5)))
                .has(|_: &TextureBuffer| ())
        });
}

fn main_texture() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Texture::from_size(MAIN_TARGET_TEXTURE, Size::new(30, 20)))
        .component(TextureBuffer::default())
        .component(MainTarget)
}

fn resource() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Camera2D::new(MAIN_CAMERA, MAIN_TARGET))
        .child_component(Camera2D::new(SECONDARY_CAMERA, SECONDARY_TARGET))
        .child_entity(material::<Default2DMaterial>(TARGET_MATERIAL).updated(
            |m: &mut Default2DMaterial| {
                m.texture_key = Some(SECONDARY_TARGET_TEXTURE);
            },
        ))
        .child_entity(secondary_target())
        .child_entity(blue_rectangle())
        .child_entity(
            instance_2d(MAIN_CAMERA, Default2DMaterial::new())
                .updated(|r: &mut InstanceRendering2D| r.material_key = TARGET_MATERIAL),
        )
}

fn secondary_target() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Texture::from_size(
            SECONDARY_TARGET_TEXTURE,
            Size::new(20, 50),
        ))
        .component(TextureBuffer::default())
        .component(RenderTarget::new(SECONDARY_TARGET))
        .with(|r| r.background_color = Color::GREEN)
        .component(SecondaryTarget)
}

fn blue_rectangle() -> impl BuiltEntity {
    instance_2d(SECONDARY_CAMERA, Default2DMaterial::new())
        .updated(|t: &mut Transform2D| t.position = Vec2::ONE * 0.25)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .updated(|m: &mut Default2DMaterial| m.color = Color::BLUE)
        .component(BlueRectangle)
}

fn blue_rectangle_with_material(material_key: ResKey<Material>) -> impl BuiltEntity {
    instance_2d(SECONDARY_CAMERA, material_key)
        .updated(|t: &mut Transform2D| t.position = Vec2::ONE * 0.25)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .updated(|m: &mut Default2DMaterial| m.color = Color::BLUE)
        .component(BlueRectangle)
}

#[derive(SingletonComponent, NoSystem)]
struct MainTarget;

#[derive(SingletonComponent, NoSystem)]
struct SecondaryTarget;

#[derive(SingletonComponent, NoSystem)]
struct BlueRectangle;

const MAIN_TARGET: ResKey<RenderTarget> = ResKey::new("main");
const SECONDARY_TARGET: ResKey<RenderTarget> = ResKey::new("secondary");
const MAIN_TARGET_TEXTURE: ResKey<Texture> = ResKey::new("main");
const SECONDARY_TARGET_TEXTURE: ResKey<Texture> = ResKey::new("secondary");
const MAIN_CAMERA: ResKey<Camera2D> = ResKey::new("main");
const SECONDARY_CAMERA: ResKey<Camera2D> = ResKey::new("secondary");
const TARGET_MATERIAL: ResKey<Material> = ResKey::new("target");
