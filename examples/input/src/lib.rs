#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, singleton, App, Built, Entity, EntityBuilder, Query, Single, With, World};
use modor_graphics::{
    Camera2D, Color, FrameRate, FrameRateLimit, GraphicsModule, ShapeColor, SurfaceSize,
    WindowSettings,
};
use modor_input::{Finger, Key, Keyboard, Mouse, MouseButton};
use modor_math::Vector3D;
use modor_physics::{Position, Size, Velocity};
use std::io;
use std::io::Write;

// TODO: support game pads
// TODO: add tests + doc

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default()
                .size(SurfaceSize::new(800, 600))
                .title("Modor - input")
                .has_visible_cursor(false),
        ))
        .with_entity(FrameRateLimit::build(FrameRate::VSync))
        .with_entity(Camera2D::build(Position::xy(0.5, 0.5), Size::xy(1.5, 1.5)))
        .with_entity(MouseState::build())
        .with_entity(KeyboardState::build())
        .with_entity(TouchState::build())
        .run(modor_graphics::runner);
}

struct MouseState;

#[singleton]
impl MouseState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., 0.))
            .with(Size::xy(0.25, 0.25))
            .with(ShapeColor(Color::DARK_GRAY))
    }

    #[run]
    fn update_position(position: &mut Position, camera: Single<'_, Camera2D>) {
        position.x = camera.mouse_position().x;
        position.y = camera.mouse_position().y;
    }

    #[run]
    fn update_color(color: &mut ShapeColor, mouse: Single<'_, Mouse>) {
        color.0.r += mouse.scroll_delta_in_lines(30., 30.).x / 50.;
        color.0.g += mouse.scroll_delta_in_lines(30., 30.).y / 50.;
        if mouse.button(MouseButton::Left).is_pressed() {
            color.0 = Color::BLUE;
        } else if mouse.button(MouseButton::Right).is_pressed() {
            color.0 = Color::DARK_GREEN;
        } else if mouse.button(MouseButton::Middle).is_pressed() {
            color.0 = Color::RED;
        }
    }
}

struct KeyboardState;

#[singleton]
impl KeyboardState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., 0.))
            .with(Size::xy(0.25, 0.25))
            .with(Velocity::xy(0., 0.))
            .with(ShapeColor(Color::DARK_GRAY))
    }

    #[run]
    fn update(velocity: &mut Velocity, color: &mut ShapeColor, keyboard: Single<'_, Keyboard>) {
        let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
        velocity.x = direction.x * 3.;
        velocity.y = direction.y * 3.;
        color.0 = if velocity.magnitude() > 0. {
            Color::RED
        } else {
            Color::DARK_GRAY
        };
    }

    #[run]
    fn print_entered_text(keyboard: Single<'_, Keyboard>) {
        if !keyboard.text().is_empty() {
            print!("{}", keyboard.text());
            io::stdout()
                .flush()
                .expect("error when displaying text in terminal");
        }
        if keyboard.key(Key::Return).is_just_released() {
            println!();
        }
    }
}

struct TouchState;

#[entity]
impl TouchState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn update_fingers(
        entity: Entity<'_>,
        fingers: Query<'_, &Finger>,
        mut state_positions: Query<'_, &mut Position, With<FingerState>>,
        camera: Single<'_, Camera2D>,
        mut world: World<'_>,
    ) {
        for child in entity.children().skip(fingers.iter().len()) {
            world.delete_entity(child.id());
        }
        for _ in entity.children().len()..fingers.iter().len() {
            world.create_child_entity(entity.id(), FingerState::build());
        }
        for (finger, state_position) in fingers.iter().zip(state_positions.iter_mut()) {
            if let Some(position) = camera.finger_position(finger.id()) {
                *state_position = position;
            }
        }
    }
}

struct FingerState;

#[entity]
impl FingerState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0.5, 0.5))
            .with(Size::xy(0.25, 0.25))
            .with(ShapeColor(Color::DARK_GRAY))
    }
}
