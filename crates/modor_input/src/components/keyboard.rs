use crate::{utils, InputState};
use fxhash::FxHashMap;
use modor_math::Vec2;
use std::ops::{Index, IndexMut};

/// The state of the keyboard.
///
/// # Examples
///
/// State access:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn access_keyboard(keyboard: SingleRef<'_, '_, Keyboard>) {
///     let keyboard = keyboard.get();
///     println!("Enter key pressed: {}", keyboard[Key::Return].is_pressed());
///     println!("Entered text: {}", keyboard.text);
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
/// struct EscapeKeyPresser;
///
/// #[systems]
/// impl EscapeKeyPresser {
///     #[run_as(component(Keyboard))]
///     fn run(mut keyboard: SingleMut<'_, '_, Keyboard>) {
///         let keyboard = keyboard.get_mut();
///         keyboard.refresh();
///         keyboard[Key::Escape].press();
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem, Debug, Default)]
pub struct Keyboard {
    /// Entered text.
    pub text: String,
    keys: FxHashMap<Key, InputState>,
}

impl Keyboard {
    /// Refreshes keyboard state.
    ///
    /// This should be called at the beginning of [`App`](modor::App) update, before updating the
    /// keyboard state.
    pub fn refresh(&mut self) {
        self.text = String::new();
        for state in self.keys.values_mut() {
            state.refresh();
        }
    }

    /// Return an iterator on all pressed keys.
    pub fn pressed_iter(&self) -> impl Iterator<Item = Key> + '_ {
        self.keys
            .iter()
            .filter(|(_, s)| s.is_pressed())
            .map(|(b, _)| *b)
    }

    /// Returns a normalized delta indicating a direction from left, right, up and down keys.
    ///
    /// If none of the keys are pressed, the returned delta is [`Vec2::ZERO`](Vec2::ZERO).
    pub fn direction(&self, left: Key, right: Key, up: Key, down: Key) -> Vec2 {
        utils::normalized_direction(
            self[left].is_pressed(),
            self[right].is_pressed(),
            self[up].is_pressed(),
            self[down].is_pressed(),
        )
    }

    /// Returns a delta between -1. and 1. from left and right keys.
    ///
    /// If none of the keys are pressed, the returned delta is `0.0`.
    pub fn axis(&self, left: Key, right: Key) -> f32 {
        utils::normalized_axis(self[left].is_pressed(), self[right].is_pressed())
    }
}

impl Index<Key> for Keyboard {
    type Output = InputState;

    fn index(&self, index: Key) -> &Self::Output {
        self.keys.get(&index).unwrap_or(&InputState::DEFAULT)
    }
}

impl IndexMut<Key> for Keyboard {
    fn index_mut(&mut self, index: Key) -> &mut Self::Output {
        self.keys.entry(index).or_default()
    }
}

/// A keyboard key.
///
/// The keys are virtual: they are not associated to a physical location on the keyboard.
///
/// # Examples
///
/// See [`Keyboard`](Keyboard).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Key {
    /// The `1` key over the letters.
    Key1,
    /// The `2` key over the letters.
    Key2,
    /// The `3` key over the letters.
    Key3,
    /// The `4` key over the letters.
    Key4,
    /// The `5` key over the letters.
    Key5,
    /// The `6` key over the letters.
    Key6,
    /// The `7` key over the letters.
    Key7,
    /// The `8` key over the letters.
    Key8,
    /// The `9` key over the letters.
    Key9,
    /// The `0` key over the letters.
    Key0,
    /// The `A` key.
    A,
    /// The `B` key.
    B,
    /// The `C` key.
    C,
    /// The `D` key.
    D,
    /// The `E` key.
    E,
    /// The `F` key.
    F,
    /// The `G` key.
    G,
    /// The `H` key.
    H,
    /// The `I` key.
    I,
    /// The `J` key.
    J,
    /// The `K` key.
    K,
    /// The `L` key.
    L,
    /// The `M` key.
    M,
    /// The `N` key.
    N,
    /// The `O` key.
    O,
    /// The `N` key.
    P,
    /// The `Q` key.
    Q,
    /// The `R` key.
    R,
    /// The `S` key.
    S,
    /// The `T` key.
    T,
    /// The `U` key.
    U,
    /// The `V` key.
    V,
    /// The `W` key.
    W,
    /// The `X` key.
    X,
    /// The `Y` key.
    Y,
    /// The `Z` key.
    Z,
    /// The `Esc` key, next to `F1`.
    Escape,
    /// The `F1` key.
    F1,
    /// The `F2` key.
    F2,
    /// The `F3` key.
    F3,
    /// The `F4` key.
    F4,
    /// The `F5` key.
    F5,
    /// The `F6` key.
    F6,
    /// The `F7` key.
    F7,
    /// The `F8` key.
    F8,
    /// The `F9` key.
    F9,
    /// The `F10` key.
    F10,
    /// The `F11` key.
    F11,
    /// The `F12` key.
    F12,
    /// The `F13` key.
    F13,
    /// The `F14` key.
    F14,
    /// The `F15` key.
    F15,
    /// The `F16` key.
    F16,
    /// The `F17` key.
    F17,
    /// The `F18` key.
    F18,
    /// The `F19` key.
    F19,
    /// The `F20` key.
    F20,
    /// The `F21` key.
    F21,
    /// The `F22` key.
    F22,
    /// The `F23` key.
    F23,
    /// The `F24` key.
    F24,
    /// The `Print Screen / SysRq` key.
    Snapshot,
    /// The `Scroll Lock` key.
    Scroll,
    /// The `Pause / Break` key, next to `Scroll Lock`.
    Pause,
    /// The `Insert` key.
    Insert,
    /// The `Home` key.
    Home,
    /// The `Delete` key.
    Delete,
    /// The `End` key.
    End,
    /// The `Page Down` key.
    PageDown,
    /// The `Page Up` key.
    PageUp,
    /// The left arrow key.
    Left,
    /// The up arrow key.
    Up,
    /// The right arrow key.
    Right,
    /// The down arrow key.
    Down,
    /// The `Backspace` key, right over `Enter`.
    Back,
    /// The `Enter` key.
    Return,
    /// The space bar.
    Space,
    /// The `Compose` key.
    Compose,
    /// The `^` key.
    Caret,
    /// The `Num Lock` key.
    Numlock,
    /// The `0` key on the numeric keypad.
    Numpad0,
    /// The `1` key on the numeric keypad.
    Numpad1,
    /// The `2` key on the numeric keypad.
    Numpad2,
    /// The `3` key on the numeric keypad.
    Numpad3,
    /// The `4` key on the numeric keypad.
    Numpad4,
    /// The `5` key on the numeric keypad.
    Numpad5,
    /// The `6` key on the numeric keypad.
    Numpad6,
    /// The `7` key on the numeric keypad.
    Numpad7,
    /// The `8` key on the numeric keypad.
    Numpad8,
    /// The `9` key on the numeric keypad.
    Numpad9,
    /// The `+` key on the numeric keypad.
    NumpadAdd,
    /// The `/` key on the numeric keypad.
    NumpadDivide,
    /// The `.` key on the numeric keypad.
    NumpadDecimal,
    /// The `,` key on the numeric keypad.
    NumpadComma,
    /// The `Enter` key on the numeric keypad.
    NumpadEnter,
    /// The `=` key on the numeric keypad.
    NumpadEquals,
    /// The `*` key on the numeric keypad.
    NumpadMultiply,
    /// The `-` key on the numeric keypad.
    NumpadSubtract,
    /// The ABNT_C1 (Brazilian) key.
    AbntC1,
    /// The ABNT_C2 (Brazilian) key.
    AbntC2,
    /// The `'` key.
    Apostrophe,
    /// The Application key, also known as Menu key.
    Apps,
    /// The `*` key.
    Asterisk,
    /// The `@` key.
    At,
    /// The AX key.
    Ax,
    /// The `\\` key.
    Backslash,
    /// The calculator key.
    Calculator,
    /// The `Caps Lock` key.
    Capital,
    /// The `:` key.
    Colon,
    /// The `,` key.
    Comma,
    /// The convert key (Japanese).
    Convert,
    /// The `=` key.
    Equals,
    /// The `\` ` key.
    Grave,
    /// The Kana key.
    Kana,
    /// The Kanji key.
    Kanji,
    /// The left `Alt` key.
    LAlt,
    /// The `[` key.
    LBracket,
    /// The left `Ctrl` key.
    LControl,
    /// The left `Shift` key.
    LShift,
    /// The left Windows key.
    LWin,
    /// The mail key.
    Mail,
    /// The media select key.
    MediaSelect,
    /// The media stop key.
    MediaStop,
    /// The `-` key.
    Minus,
    /// The `🔇` key.
    Mute,
    /// The My Computer key.
    MyComputer,
    /// The Navigate Forward key.
    NavigateForward,
    /// The Navigate Backward key.
    NavigateBackward,
    /// The `⏭` key.
    NextTrack,
    /// The no convert key (Japanese).
    NoConvert,
    /// The OEM 102 key.
    OEM102,
    /// The `.` key.
    Period,
    /// The `⏯` key.
    PlayPause,
    /// The `+ key.
    Plus,
    /// The `⏻` key.
    Power,
    /// The `⏮` key.
    PrevTrack,
    /// The right `Alt` key.
    RAlt,
    /// The `]` key.
    RBracket,
    /// The right `Ctrl` key.
    RControl,
    /// The right `Shift` key.
    RShift,
    /// The right Windows key.
    RWin,
    /// The `;` key.
    Semicolon,
    /// The `/` key.
    Slash,
    /// The `💤` key.
    Sleep,
    /// The `⏹` key.
    Stop,
    /// The `SysRq` key.
    Sysrq,
    /// The `↹` key.
    Tab,
    /// The `_` key.
    Underline,
    /// A blank key.
    Unlabeled,
    /// The `🔉` key.
    VolumeDown,
    /// The `🔊` key.
    VolumeUp,
    /// The wake key.
    Wake,
    /// The web back.
    WebBack,
    /// The web favorites key.
    WebFavorites,
    /// The web forwawrd key.
    WebForward,
    /// The web home key.
    WebHome,
    /// The web refresh key.
    WebRefresh,
    /// The web search key.
    WebSearch,
    /// The web stop key.
    WebStop,
    /// The `¥` key.
    Yen,
    /// The copy key.
    Copy,
    /// The paste key.
    Paste,
    /// The cut key.
    Cut,
}
