use modor::{App, BuiltEntity, EntityAssertions, EntityFilter, With};
use modor_graphics::testing::has_component_diff;
use modor_graphics::{
    model_2d, texture_target, Material, Model2DMaterial, Size, Sprite, Texture, TextureAnimation,
    TextureBuffer, TEXTURE_CAMERAS_2D,
};
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;
use std::thread::sleep;
use std::time::Duration;

const TEXTURE_PATH: &str = "../tests/assets/spritesheet.png";

#[modor_test(disabled(macos, android, wasm))]
fn run_texture_animation() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(25, 25), true))
        .with_entity(spritesheet_texture())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(sprite(vec![Sprite::new(0, 2), Sprite::new(1, 2)], 2))
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(0))
        .updated()
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(0))
        .assert::<With<TextureBuffer>>(1, has_component_diff("animation#frame0", 50, 2))
        .updated_until_all::<(), TextureAnimation>(Some(1), sleep_one_frame)
        .updated()
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(1))
        .assert::<With<TextureBuffer>>(1, has_component_diff("animation#frame1", 50, 2))
        .updated_until_all::<(), TextureAnimation>(Some(1), sleep_one_frame)
        .updated()
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(0))
        .assert::<With<TextureBuffer>>(1, has_component_diff("animation#frame0", 50, 2));
}

#[modor_test(disabled(macos, android, wasm))]
fn run_texture_animation_without_frame() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(25, 25), true))
        .with_entity(spritesheet_texture())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(sprite(vec![], 2))
        .updated()
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(0))
        .assert::<With<TextureBuffer>>(1, has_component_diff("animation#frame0", 150, 2));
}

#[modor_test(disabled(macos, android, wasm))]
fn run_texture_animation_at_zero_fps() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(25, 25), true))
        .with_entity(spritesheet_texture())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(sprite(vec![Sprite::new(0, 2), Sprite::new(1, 2)], 0))
        .updated()
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(0))
        .assert::<With<TextureBuffer>>(1, has_component_diff("animation#frame0", 50, 2))
        .updated()
        .assert::<With<TextureAnimation>>(1, assert_sprite_index(0))
        .assert::<With<TextureBuffer>>(1, has_component_diff("animation#frame0", 50, 2));
}

fn spritesheet_texture() -> Texture {
    let mut texture = Texture::from_path(SPRITESHEET_TEXTURE, TEXTURE_PATH);
    texture.is_smooth = false;
    texture
}

fn sprite(sprites: Vec<Sprite>, fps: u16) -> impl BuiltEntity {
    model_2d(TEXTURE_CAMERAS_2D.get(0), Model2DMaterial::Rectangle)
        .updated(|m: &mut Material| m.texture_key = Some(SPRITESHEET_TEXTURE))
        .component(TextureAnimation::new(5, 9, sprites))
        .with(|a| a.frames_per_second = fps)
}

fn sleep_one_frame<C>(_: &C) -> bool {
    sleep(Duration::from_secs_f32(0.5));
    true
}

fn assert_sprite_index<F>(
    index: usize,
) -> impl FnOnce(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| e.has(|a: &TextureAnimation| assert_eq!(a.current_sprite_index(), index))
}

const SPRITESHEET_TEXTURE: ResKey<Texture> = ResKey::new("spritesheet");
