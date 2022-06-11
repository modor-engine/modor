#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, singleton, App, Built, Entity, EntityBuilder, Query, Single, World};
use modor_graphics::{
    Camera2D, Color, FrameRate, FrameRateLimit, GraphicsModule, ShapeColor, SurfaceSize,
    WindowSettings,
};
use modor_input::{
    Finger, Gamepad, GamepadButton, GamepadStick, Key, Keyboard, Mouse, MouseButton,
};
use modor_math::{Vector2D, Vector3D};
use modor_physics::{Position, Size, Velocity};
use std::io;
use std::io::Write;

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
        .with_entity(GamepadsState::build())
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
    fn create_fingers(
        entity: Entity<'_>,
        finger_states: Query<'_, &FingerState>,
        fingers: Query<'_, &Finger>,
        mut world: World<'_>,
    ) {
        for finger in fingers.iter() {
            if !finger_states.iter().any(|s| s.id == finger.id()) {
                world.create_child_entity(entity.id(), FingerState::build(finger.id()));
            }
        }
    }
}

struct FingerState {
    id: u64,
}

#[entity]
impl FingerState {
    fn build(id: u64) -> impl Built<Self> {
        EntityBuilder::new(Self { id })
            .with(Position::xy(0.5, 0.5))
            .with(Size::xy(0.25, 0.25))
            .with(ShapeColor(Color::DARK_GRAY))
    }

    #[run]
    fn update(
        &self,
        position: &mut Position,
        fingers: Query<'_, &Finger>,
        camera: Single<'_, Camera2D>,
    ) {
        if let Some(finger) = fingers.iter().find(|f| f.id() == self.id) {
            if let Some(finger_position) = camera.finger_position(finger.id()) {
                *position = finger_position;
            }
        }
    }

    #[run]
    fn delete(&self, entity: Entity<'_>, fingers: Query<'_, &Finger>, mut world: World<'_>) {
        if !fingers.iter().any(|f| f.id() == self.id) {
            world.delete_entity(entity.id());
        }
    }
}

struct GamepadsState;

#[entity]
impl GamepadsState {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn create_fingers(
        entity: Entity<'_>,
        gamepad_states: Query<'_, &GamepadState>,
        gamepads: Query<'_, &Gamepad>,
        mut world: World<'_>,
    ) {
        for gamepad in gamepads.iter() {
            if !gamepad_states.iter().any(|s| s.id == gamepad.id()) {
                world.create_child_entity(entity.id(), GamepadState::build(gamepad.id()));
            }
        }
    }
}

struct GamepadState {
    id: u64,
}

#[entity]
impl GamepadState {
    fn build(id: u64) -> impl Built<Self> {
        EntityBuilder::new(Self { id })
            .with(Position::xy(0.5, 0.5))
            .with(Size::xy(0.25, 0.25))
            .with(Velocity::ZERO)
            .with(ShapeColor(Color::MAROON))
    }

    #[run]
    fn update(
        &self,
        color: &mut ShapeColor,
        velocity: &mut Velocity,
        gamepads: Query<'_, &Gamepad>,
    ) {
        if let Some(gamepad) = gamepads.iter().find(|f| f.id() == self.id) {
            let red = 1. - gamepad.button(GamepadButton::BackLeftTrigger).value();
            let green = 1. - gamepad.button(GamepadButton::BackRightTrigger).value();
            let blue = gamepad
                .button(GamepadButton::South)
                .state()
                .is_pressed()
                .then(|| 0.)
                .unwrap_or(1.);
            color.0 = Color::rgb(red, green, blue);
            let velocity1: Velocity = gamepad.stick_direction(GamepadStick::LeftStick).into_vec3();
            let velocity2 = gamepad
                .stick_direction(GamepadStick::RightStick)
                .into_vec3();
            let velocity3 = gamepad.stick_direction(GamepadStick::DPad).into_vec3();
            let velocity4 = Velocity::xy(gamepad.left_z_axis_value(), gamepad.right_z_axis_value());
            *velocity = velocity1 + velocity2 + velocity3 + velocity4;
        }
    }

    #[run]
    fn delete(&self, entity: Entity<'_>, gamepads: Query<'_, &Gamepad>, mut world: World<'_>) {
        if !gamepads.iter().any(|f| f.id() == self.id) {
            world.delete_entity(entity.id());
        }
    }
}
