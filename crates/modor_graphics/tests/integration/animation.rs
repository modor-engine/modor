use log::Level;
use modor::{App, Node, RootNode, Visit};
use modor_graphics::{TextureAnimation, TexturePart};
use modor_input::modor_math::Vec2;
use modor_internal::assert_approx_eq;
use std::time::Duration;

#[modor::test(disabled(wasm))]
fn create_with_some_parts() {
    let mut app = App::new::<Root>(Level::Info);
    let mut texture_animation = TextureAnimation::new(2, 4)
        .with_fps(2)
        .with_parts(|p| *p = vec![TexturePart::new(1, 2), TexturePart::new(0, 0)]);
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::ZERO);
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::new(0.5, 0.5));
    spin_sleep::sleep(Duration::from_millis(510));
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::ZERO);
    spin_sleep::sleep(Duration::from_millis(500));
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::new(0.5, 0.5));
}

#[modor::test(disabled(wasm))]
fn create_with_no_part() {
    let mut app = App::new::<Root>(Level::Info);
    let mut texture_animation = TextureAnimation::new(2, 4);
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::ZERO);
    spin_sleep::sleep(Duration::from_millis(110));
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::ZERO);
}

#[modor::test(disabled(wasm))]
fn create_with_zero_fps() {
    let mut app = App::new::<Root>(Level::Info);
    let mut texture_animation = TextureAnimation::new(2, 4)
        .with_fps(0)
        .with_parts(|p| *p = vec![TexturePart::new(1, 2), TexturePart::new(0, 0)]);
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::new(0.5, 0.5));
    spin_sleep::sleep(Duration::from_millis(110));
    texture_animation.update(&mut app.ctx());
    assert_approx_eq!(texture_animation.part_size(), Vec2::new(0.5, 0.25));
    assert_approx_eq!(texture_animation.part_position(), Vec2::new(0.5, 0.5));
}

#[derive(Default, RootNode, Node, Visit)]
struct Root;
