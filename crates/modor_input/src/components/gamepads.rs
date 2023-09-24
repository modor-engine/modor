use crate::{utils, InputState};
use fxhash::FxHashMap;
use modor_math::Vec2;
use std::ops::{Index, IndexMut};
use std::sync::OnceLock;

/// Direction of a gamepad stick.
///
/// X coordinate corresponds to the horizontal position of the stick between `-1.0` and `1.0`.<br>
/// Y coordinate corresponds to the vertical position of the stick between `-1.0` and `1.0`.
pub type GamepadStickDirection = Vec2;

/// The state of the gamepads.
///
/// # Examples
///
/// State access:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn access_gamepads(gamepads: SingleRef<'_, '_, Gamepads>) {
///     let gamepads = gamepads.get();
///     println!("Left stick direction of gamepad 0: {:?}", gamepads[0][GamepadStick::LeftStick]);
///     for gamepad_id in gamepads.iter() {
///         let gamepad = &gamepads[gamepad_id];
///         println!("=== Gamepad {gamepad_id} ===");
///         println!("Start button pressed: {}", gamepad[GamepadButton::Start].state.is_pressed());
///         println!("Left stick direction: {:?}", gamepad[GamepadStick::LeftStick]);
///     }
/// }
/// ```
///
/// State update:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// #[derive(Component)]
/// struct StartButtonPresser;
///
/// #[systems]
/// impl StartButtonPresser {
///     #[run_as(component(Gamepads))]
///     fn run(mut gamepads: SingleMut<'_, '_, Gamepads>) {
///         let gamepads = gamepads.get_mut();
///         gamepads.refresh();
///         let gamepad_id = 0;
///         gamepads[gamepad_id][GamepadButton::Start].state.press();
///     }
/// }
/// ```
#[derive(SingletonComponent, NoSystem, Debug, Default)]
pub struct Gamepads {
    gamepads: FxHashMap<u64, Gamepad>,
}

impl Gamepads {
    /// Refreshes gamepads state.
    ///
    /// This should be called at the beginning of [`App`](modor::App) update, before updating the
    /// gamepads state.
    pub fn refresh(&mut self) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.refresh();
        }
    }

    /// Synchronizes direction pad buttons with stick.
    ///
    /// This should be called after all other gamepad updates.
    pub fn sync_d_pad(&mut self) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.sync_d_pad();
        }
    }

    /// Returns an iterator on gamepad IDs.
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.gamepads.keys().copied()
    }
}

impl Index<u64> for Gamepads {
    type Output = Gamepad;

    fn index(&self, index: u64) -> &Self::Output {
        self.gamepads
            .get(&index)
            .unwrap_or_else(|| DEFAULT_GAMEPAD.get_or_init(Gamepad::default))
    }
}

impl IndexMut<u64> for Gamepads {
    fn index_mut(&mut self, index: u64) -> &mut Self::Output {
        self.gamepads.entry(index).or_default()
    }
}

/// The state of a gamepad.
///
/// # Examples
///
/// See [`Gamepads`].
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Gamepad {
    /// Whether the gamepad is connected.
    pub is_connected: bool,
    buttons: FxHashMap<GamepadButton, GamepadButtonState>,
    stick_directions: FxHashMap<GamepadStick, GamepadStickDirection>,
    has_d_pad_button: bool,
}

static DEFAULT_GAMEPAD: OnceLock<Gamepad> = OnceLock::new();

impl Gamepad {
    /// Return an iterator on all pressed buttons.
    pub fn pressed_iter(&self) -> impl Iterator<Item = GamepadButton> + '_ {
        self.buttons
            .iter()
            .filter(|(_, s)| s.state.is_pressed())
            .map(|(&b, _)| b)
    }

    fn refresh(&mut self) {
        for button in self.buttons.values_mut() {
            button.refresh();
        }
    }

    fn sync_d_pad(&mut self) {
        if self[GamepadButton::DPadLeft].state.is_pressed()
            || self[GamepadButton::DPadRight].state.is_pressed()
            || self[GamepadButton::DPadUp].state.is_pressed()
            || self[GamepadButton::DPadDown].state.is_pressed()
        {
            self.has_d_pad_button = true;
        }
        if self.has_d_pad_button {
            self[GamepadStick::DPad] = utils::normalized_direction(
                self[GamepadButton::DPadLeft].state.is_pressed(),
                self[GamepadButton::DPadRight].state.is_pressed(),
                self[GamepadButton::DPadUp].state.is_pressed(),
                self[GamepadButton::DPadDown].state.is_pressed(),
            );
        }
    }
}

impl Index<GamepadButton> for Gamepad {
    type Output = GamepadButtonState;

    fn index(&self, index: GamepadButton) -> &Self::Output {
        self.buttons
            .get(&index)
            .unwrap_or(&GamepadButtonState::DEFAULT)
    }
}

impl IndexMut<GamepadButton> for Gamepad {
    fn index_mut(&mut self, index: GamepadButton) -> &mut Self::Output {
        self.buttons.entry(index).or_default()
    }
}

impl Index<GamepadStick> for Gamepad {
    type Output = GamepadStickDirection;

    fn index(&self, index: GamepadStick) -> &Self::Output {
        self.stick_directions
            .get(&index)
            .unwrap_or(&GamepadStickDirection::ZERO)
    }
}

impl IndexMut<GamepadStick> for Gamepad {
    fn index_mut(&mut self, index: GamepadStick) -> &mut Self::Output {
        self.stick_directions.entry(index).or_default()
    }
}

/// The state of the gamepad button.
///
/// # Examples
///
/// See [`Gamepads`].
#[derive(Default, Debug, Clone, Copy)]
pub struct GamepadButtonState {
    /// State of the button.
    pub state: InputState,
    /// Value between `0.0` and `1.0` of the button state.
    pub value: f32,
}

impl GamepadButtonState {
    const DEFAULT: Self = Self {
        state: InputState::new(),
        value: 0.,
    };

    fn refresh(&mut self) {
        self.state.refresh();
    }
}

/// A gamepad button.
///
/// # Examples
///
/// See [`Gamepads`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
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

/// A gamepad stick.
///
/// # Examples
///
/// See [`Gamepads`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum GamepadStick {
    /// The left stick.
    LeftStick,
    /// The right stick.
    RightStick,
    /// The directional pad.
    DPad,
}
