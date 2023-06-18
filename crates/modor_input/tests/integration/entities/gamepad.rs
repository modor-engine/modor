use modor::{App, With};
use modor_input::{
    Gamepad, GamepadAxis, GamepadButton, GamepadEvent, GamepadStick, InputEventCollector,
    InputModule,
};
use modor_math::Vec2;

#[modor_test(disabled(wasm))]
fn handle_gamepads() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::Plugged(0).into());
            c.push(GamepadEvent::Plugged(1).into());
        })
        .updated()
        .assert_any::<With<Gamepad>>(2, |e| {
            e.has(|g: &Gamepad| assert_eq!(g.id(), 0))
                .has(|g: &Gamepad| assert_eq!(g.id(), 1))
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::Unplugged(1).into());
            c.push(GamepadEvent::Plugged(2).into());
        })
        .updated()
        .assert_any::<With<Gamepad>>(2, |e| {
            e.has(|g: &Gamepad| assert_eq!(g.id(), 0))
                .has(|g: &Gamepad| assert_eq!(g.id(), 2))
        });
}

#[modor_test]
fn update_button_state() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| c.push(GamepadEvent::Plugged(0).into()))
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                assert_eq!(g.pressed_buttons().collect::<Vec<_>>(), []);
                assert!(!g.button(GamepadButton::Start).state().is_just_pressed);
                assert!(!g.button(GamepadButton::Start).state().is_just_released);
                assert!(!g.button(GamepadButton::Start).state().is_pressed);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::PressedButton(0, GamepadButton::Start).into());
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                let pressed_buttons = g.pressed_buttons().collect::<Vec<_>>();
                assert_eq!(pressed_buttons, [GamepadButton::Start]);
                assert!(g.button(GamepadButton::Start).state().is_just_pressed);
                assert!(!g.button(GamepadButton::Start).state().is_just_released);
                assert!(g.button(GamepadButton::Start).state().is_pressed);
            })
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                let pressed_buttons = g.pressed_buttons().collect::<Vec<_>>();
                assert_eq!(pressed_buttons, [GamepadButton::Start]);
                assert!(!g.button(GamepadButton::Start).state().is_just_pressed);
                assert!(!g.button(GamepadButton::Start).state().is_just_released);
                assert!(g.button(GamepadButton::Start).state().is_pressed);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::ReleasedButton(0, GamepadButton::Start).into());
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                assert_eq!(g.pressed_buttons().collect::<Vec<_>>(), []);
                assert!(!g.button(GamepadButton::Start).state().is_just_pressed);
                assert!(g.button(GamepadButton::Start).state().is_just_released);
                assert!(!g.button(GamepadButton::Start).state().is_pressed);
            })
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                assert_eq!(g.pressed_buttons().collect::<Vec<_>>(), []);
                assert!(!g.button(GamepadButton::Start).state().is_just_pressed);
                assert!(!g.button(GamepadButton::Start).state().is_just_released);
                assert!(!g.button(GamepadButton::Start).state().is_pressed);
            })
        });
}

#[modor_test]
fn update_button_value() {
    let button = GamepadButton::BackLeftTrigger;
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| c.push(GamepadEvent::Plugged(0).into()))
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| assert_approx_eq!(g.button(button).value(), 0.))
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::UpdatedButtonValue(0, button, 0.5).into());
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| assert_approx_eq!(g.button(button).value(), 0.5))
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| assert_approx_eq!(g.button(button).value(), 0.5))
        });
}

#[modor_test]
fn update_axis() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| c.push(GamepadEvent::Plugged(0).into()))
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                assert_approx_eq!(g.stick_direction(GamepadStick::LeftStick), Vec2::ZERO);
                assert_approx_eq!(g.stick_direction(GamepadStick::RightStick), Vec2::ZERO);
                assert_approx_eq!(g.stick_direction(GamepadStick::DPad), Vec2::ZERO);
                assert_approx_eq!(g.left_z_axis_value(), 0.);
                assert_approx_eq!(g.right_z_axis_value(), 0.);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::LeftStickX, 0.1).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::LeftStickY, 0.2).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::RightStickX, 0.3).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::RightStickY, 0.4).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::DPadX, 0.5).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::DPadY, 0.6).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::LeftZ, 0.7).into());
            c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::RightZ, 0.8).into());
        })
        .updated()
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                let direction = g.stick_direction(GamepadStick::LeftStick);
                assert_approx_eq!(direction, Vec2::new(0.1, 0.2));
                let direction = g.stick_direction(GamepadStick::RightStick);
                assert_approx_eq!(direction, Vec2::new(0.3, 0.4));
                assert_approx_eq!(g.stick_direction(GamepadStick::DPad), Vec2::new(0.5, 0.6));
                assert_approx_eq!(g.left_z_axis_value(), 0.7);
                assert_approx_eq!(g.right_z_axis_value(), 0.8);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(GamepadEvent::PressedButton(0, GamepadButton::DPadRight).into());
        })
        .updated()
        .assert::<With<Gamepad>>(1, |e| {
            e.has(|g: &Gamepad| {
                assert_approx_eq!(g.stick_direction(GamepadStick::DPad), Vec2::new(1., 0.));
            })
        });
}
