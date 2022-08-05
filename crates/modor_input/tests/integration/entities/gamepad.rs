use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::App;
use modor_input::{
    Gamepad, GamepadAxis, GamepadButton, GamepadEvent, GamepadStick, InputEventCollector,
    InputModule,
};
use modor_math::Vec2;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn handle_gamepads() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::Plugged(0).into());
        c.push(GamepadEvent::Plugged(1).into());
    });
    app.update();
    app.assert_entity(4)
        .has(|g: &Gamepad| assert_eq!(g.id(), 0));
    app.assert_entity(5)
        .has(|g: &Gamepad| assert_eq!(g.id(), 1));
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::Unplugged(1).into());
        c.push(GamepadEvent::Plugged(2).into());
    });
    app.update();
    app.assert_entity(4)
        .has(|g: &Gamepad| assert_eq!(g.id(), 0));
    app.assert_entity(5).does_not_exist();
    app.assert_entity(6)
        .has(|g: &Gamepad| assert_eq!(g.id(), 2));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_button_state() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(GamepadEvent::Plugged(0).into()));
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_eq!(g.pressed_buttons().collect::<Vec<_>>(), []);
        assert!(!g.button(GamepadButton::Start).state().is_just_pressed());
        assert!(!g.button(GamepadButton::Start).state().is_just_released());
        assert!(!g.button(GamepadButton::Start).state().is_pressed());
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::PressedButton(0, GamepadButton::Start).into());
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_eq!(
            g.pressed_buttons().collect::<Vec<_>>(),
            [GamepadButton::Start]
        );
        assert!(g.button(GamepadButton::Start).state().is_just_pressed());
        assert!(!g.button(GamepadButton::Start).state().is_just_released());
        assert!(g.button(GamepadButton::Start).state().is_pressed());
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_eq!(
            g.pressed_buttons().collect::<Vec<_>>(),
            [GamepadButton::Start]
        );
        assert!(!g.button(GamepadButton::Start).state().is_just_pressed());
        assert!(!g.button(GamepadButton::Start).state().is_just_released());
        assert!(g.button(GamepadButton::Start).state().is_pressed());
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::ReleasedButton(0, GamepadButton::Start).into());
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_eq!(g.pressed_buttons().collect::<Vec<_>>(), []);
        assert!(!g.button(GamepadButton::Start).state().is_just_pressed());
        assert!(g.button(GamepadButton::Start).state().is_just_released());
        assert!(!g.button(GamepadButton::Start).state().is_pressed());
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_eq!(g.pressed_buttons().collect::<Vec<_>>(), []);
        assert!(!g.button(GamepadButton::Start).state().is_just_pressed());
        assert!(!g.button(GamepadButton::Start).state().is_just_released());
        assert!(!g.button(GamepadButton::Start).state().is_pressed());
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_button_value() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(GamepadEvent::Plugged(0).into()));
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_abs_diff_eq!(g.button(GamepadButton::BackLeftTrigger).value(), 0.);
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::UpdatedButtonValue(0, GamepadButton::BackLeftTrigger, 0.5).into());
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_abs_diff_eq!(g.button(GamepadButton::BackLeftTrigger).value(), 0.5);
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_abs_diff_eq!(g.button(GamepadButton::BackLeftTrigger).value(), 0.5);
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_axis() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| c.push(GamepadEvent::Plugged(0).into()));
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_abs_diff_eq!(g.stick_direction(GamepadStick::LeftStick), Vec2::ZERO);
        assert_abs_diff_eq!(g.stick_direction(GamepadStick::RightStick), Vec2::ZERO);
        assert_abs_diff_eq!(g.stick_direction(GamepadStick::DPad), Vec2::ZERO);
        assert_abs_diff_eq!(g.left_z_axis_value(), 0.);
        assert_abs_diff_eq!(g.right_z_axis_value(), 0.);
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::LeftStickX, 0.1).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::LeftStickY, 0.2).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::RightStickX, 0.3).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::RightStickY, 0.4).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::DPadX, 0.5).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::DPadY, 0.6).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::LeftZ, 0.7).into());
        c.push(GamepadEvent::UpdatedAxisValue(0, GamepadAxis::RightZ, 0.8).into());
    });
    app.update();
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_abs_diff_eq!(
            g.stick_direction(GamepadStick::LeftStick),
            Vec2::new(0.1, 0.2)
        );
        assert_abs_diff_eq!(
            g.stick_direction(GamepadStick::RightStick),
            Vec2::new(0.3, 0.4)
        );
        assert_abs_diff_eq!(g.stick_direction(GamepadStick::DPad), Vec2::new(0.5, 0.6));
        assert_abs_diff_eq!(g.left_z_axis_value(), 0.7);
        assert_abs_diff_eq!(g.right_z_axis_value(), 0.8);
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(GamepadEvent::PressedButton(0, GamepadButton::DPadRight).into());
    });
    app.update();
    app.assert_entity(4).has(|g: &Gamepad| {
        assert_abs_diff_eq!(g.stick_direction(GamepadStick::DPad), Vec2::new(1., 0.));
    });
}
