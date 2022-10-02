use crate::{utils, InputState};
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder};
use modor_math::Vec2;

/// The state of a gamepad.
///
/// The entity only exists if the gamepad is plugged.
///
/// # Modor
///
/// - **Type**: entity
/// - **Lifetime**: same as [`InputModule`](crate::InputModule)
/// - **Updated during**: [`UpdateInputAction`](crate::UpdateInputAction)
///
/// # Examples
///
/// ```rust
/// # use modor::{Single, Query};
/// # use modor_input::{Gamepad, GamepadButton, GamepadStick};
/// #
/// fn access_gamepads(gamepads: Query<'_, &Gamepad>) {
///     for gamepad in gamepads.iter() {
///         let button_pressed = gamepad.button(GamepadButton::Start).state().is_pressed;
///         let stick_direction = gamepad.stick_direction(GamepadStick::LeftStick);
///         println!("Button of gamepad {} is pressed: {:?}", gamepad.id(), button_pressed);
///         println!("Left stick direction of gamepad {}: {:?}", gamepad.id(), stick_direction);
///     }
/// }
/// ```
pub struct Gamepad {
    id: u64,
    buttons: FxHashMap<GamepadButton, GamepadButtonState>,
    stick_directions: FxHashMap<GamepadStick, Vec2>,
    left_z_axis_value: f32,
    right_z_axis_value: f32,
    has_d_pad_buttons: bool,
}

#[entity]
impl Gamepad {
    /// Unique identifier of the gamepad.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Return all pressed buttons.
    pub fn pressed_buttons(&self) -> impl Iterator<Item = GamepadButton> + '_ {
        self.buttons
            .iter()
            .filter(|(_, s)| s.state().is_pressed)
            .map(|(b, _)| *b)
    }

    /// Returns the state of a button.
    #[must_use]
    pub fn button(&self, button: GamepadButton) -> GamepadButtonState {
        self.buttons.get(&button).copied().unwrap_or_default()
    }

    /// Returns the normalized direction of a stick.
    ///
    /// If the stick is not used, the returned direction has all components equal to `0.0`.
    #[must_use]
    pub fn stick_direction(&self, stick: GamepadStick) -> Vec2 {
        self.stick_directions
            .get(&stick)
            .copied()
            .unwrap_or_default()
    }

    /// Returns the value between `-1.0` and `1.0` of the left Z axis.
    #[must_use]
    pub fn left_z_axis_value(&self) -> f32 {
        self.left_z_axis_value
    }

    /// Returns the value between `-1.0` and `1.0` of the right Z axis.
    #[must_use]
    pub fn right_z_axis_value(&self) -> f32 {
        self.right_z_axis_value
    }

    pub(crate) fn build(id: u64) -> impl Built<Self> {
        EntityBuilder::new(Self {
            id,
            buttons: FxHashMap::default(),
            stick_directions: FxHashMap::default(),
            left_z_axis_value: 0.0,
            right_z_axis_value: 0.0,
            has_d_pad_buttons: false,
        })
    }

    pub(crate) fn reset(&mut self) {
        for button in self.buttons.values_mut() {
            button.reset();
        }
    }

    pub(crate) fn apply_event(&mut self, event: &GamepadEvent) {
        match event {
            GamepadEvent::Plugged(_) | GamepadEvent::Unplugged(_) => {
                unreachable!("internal error: unreachable gamepad event to apply")
            }
            GamepadEvent::PressedButton(_, button) => {
                if matches!(
                    button,
                    GamepadButton::DPadUp
                        | GamepadButton::DPadDown
                        | GamepadButton::DPadLeft
                        | GamepadButton::DPadRight
                ) {
                    self.has_d_pad_buttons = true;
                }
                self.buttons.entry(*button).or_default().state.press();
            }
            GamepadEvent::ReleasedButton(_, button) => {
                self.buttons.entry(*button).or_default().state.release();
            }
            GamepadEvent::UpdatedButtonValue(_, button, value) => {
                self.buttons.entry(*button).or_default().value = *value;
            }
            GamepadEvent::UpdatedAxisValue(_, axis, value) => match axis {
                GamepadAxis::LeftStickX => {
                    self.stick_directions
                        .entry(GamepadStick::LeftStick)
                        .or_default()
                        .x = *value;
                }
                GamepadAxis::LeftStickY => {
                    self.stick_directions
                        .entry(GamepadStick::LeftStick)
                        .or_default()
                        .y = *value;
                }
                GamepadAxis::RightStickX => {
                    self.stick_directions
                        .entry(GamepadStick::RightStick)
                        .or_default()
                        .x = *value;
                }
                GamepadAxis::RightStickY => {
                    self.stick_directions
                        .entry(GamepadStick::RightStick)
                        .or_default()
                        .y = *value;
                }
                GamepadAxis::DPadX => {
                    self.stick_directions
                        .entry(GamepadStick::DPad)
                        .or_default()
                        .x = *value;
                }
                GamepadAxis::DPadY => {
                    self.stick_directions
                        .entry(GamepadStick::DPad)
                        .or_default()
                        .y = *value;
                }
                GamepadAxis::LeftZ => self.left_z_axis_value = *value,
                GamepadAxis::RightZ => self.right_z_axis_value = *value,
            },
        }
    }

    pub(crate) fn normalize(&mut self) {
        let d_pad_direction = utils::normalized_direction(
            self.buttons
                .get(&GamepadButton::DPadLeft)
                .map_or(false, |b| b.state().is_pressed),
            self.buttons
                .get(&GamepadButton::DPadRight)
                .map_or(false, |b| b.state().is_pressed),
            self.buttons
                .get(&GamepadButton::DPadUp)
                .map_or(false, |b| b.state().is_pressed),
            self.buttons
                .get(&GamepadButton::DPadDown)
                .map_or(false, |b| b.state().is_pressed),
        );
        if self.has_d_pad_buttons {
            *self.stick_directions.entry(GamepadStick::DPad).or_default() = d_pad_direction;
        }
    }
}

/// A gamepad event.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Debug, Clone)]
pub enum GamepadEvent {
    /// Gamepad plugged.
    Plugged(u64),
    /// Gamepad unplugged.
    Unplugged(u64),
    /// Gamepad button pressed.
    PressedButton(u64, GamepadButton),
    /// Gamepad button pressed.
    ReleasedButton(u64, GamepadButton),
    /// Gamepad button value between `0.0` and `1.0` updated.
    UpdatedButtonValue(u64, GamepadButton, f32),
    /// Gamepad axis value between `-1.0` and `1.0` updated.
    UpdatedAxisValue(u64, GamepadAxis, f32),
}

impl GamepadEvent {
    pub(crate) fn id(&self) -> u64 {
        *match self {
            Self::PressedButton(id, _)
            | Self::ReleasedButton(id, _)
            | Self::UpdatedButtonValue(id, _, _)
            | Self::UpdatedAxisValue(id, _, _) => id,
            Self::Plugged(_) | Self::Unplugged(_) => {
                unreachable!("internal error: unreachable gamepad event without ID")
            }
        }
    }
}

/// The state of the gamepad button.
///
/// # Examples
///
/// See [`Gamepad`](crate::Gamepad).
#[derive(Default, Clone, Copy)]
pub struct GamepadButtonState {
    state: InputState,
    value: f32,
}

impl GamepadButtonState {
    /// Returns the state of the button.
    #[must_use]
    pub fn state(&self) -> InputState {
        self.state
    }

    /// Returns the value between `0.0` and `1.0` of the button.
    #[must_use]
    pub fn value(&self) -> f32 {
        self.value
    }

    pub(crate) fn reset(&mut self) {
        self.state.refresh();
    }
}

/// A gamepad button.
///
/// # Examples
///
/// See [`Gamepad`](crate::Gamepad).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GamepadButton {
    /// Up button on the right of the gamepad.
    ///
    /// For example: `Y` button for Xbox gamepads, `△` for Playstation gamepads.
    North,
    /// Down button on the right of the gamepad.
    ///
    /// For example: `A` button for Xbox gamepads, `X` for Playstation gamepads.
    South,
    /// Left button on the right of the gamepad.
    ///
    /// For example: `X` button for Xbox gamepads, `□` for Playstation gamepads.
    West,
    /// Right button on the right of the gamepad.
    ///
    /// For example: `B` button for Xbox gamepads, `◯` for Playstation gamepads.
    East,
    /// The `C` button.
    C,
    /// The `Z` button.
    Z,
    /// Left trigger in front of the gamepad.
    ///
    /// For example, `LB` for Xbox gamepads, `L1` for Playstation gamepads.
    FrontLeftTrigger,
    /// Left trigger at back of the gamepad.
    ///
    /// For example, `LT` for Xbox gamepads, `L2` for Playstation gamepads.
    BackLeftTrigger,
    /// Right trigger in front of the gamepad.
    ///
    /// For example, `RB` for Xbox gamepads, `R1` for Playstation gamepads.
    FrontRightTrigger,
    /// Right trigger at back of the gamepad.
    ///
    /// For example, `RT` for Xbox gamepads, `R2` for Playstation gamepads.
    BackRightTrigger,
    /// The `Select` button.
    Select,
    /// The `Start` button.
    Start,
    /// The `Mode` button.
    Mode,
    /// The button corresponding to the left joystick.
    LeftStick,
    /// The button corresponding to the right joystick.
    RightStick,
    /// The up button of the directional pad.
    DPadUp,
    /// The down button of the directional pad.
    DPadDown,
    /// The left button of the directional pad.
    DPadLeft,
    /// The right button of the directional pad.
    DPadRight,
}

/// A gamepad axis.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GamepadAxis {
    /// The X axis of the left stick.
    LeftStickX,
    /// The Y axis of the left stick.
    LeftStickY,
    /// The X axis of the right stick.
    RightStickX,
    /// The Y axis of the right stick.
    RightStickY,
    /// The X axis of the directional pad.
    DPadX,
    /// The Y axis of the directional pad.
    DPadY,
    /// The Z axis on the left part of the gamepad.
    LeftZ,
    /// The Z axis on the right part of the gamepad.
    RightZ,
}

/// A gamepad stick.
///
/// # Examples
///
/// See [`Gamepad`](crate::Gamepad).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GamepadStick {
    /// The left stick.
    LeftStick,
    /// The right stick.
    RightStick,
    /// The directional pad.
    DPad,
}
