use modor_input::{GamepadButton, Key, MouseButton};
use winit::event;
use winit::keyboard::KeyCode;

// coverage: off (inputs cannot be tested)

pub(crate) fn to_mouse_button(button: event::MouseButton) -> MouseButton {
    match button {
        event::MouseButton::Left => MouseButton::Left,
        event::MouseButton::Right => MouseButton::Right,
        event::MouseButton::Middle => MouseButton::Middle,
        event::MouseButton::Back => MouseButton::Back,
        event::MouseButton::Forward => MouseButton::Forward,
        event::MouseButton::Other(id) => MouseButton::Other(id),
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn to_keyboard_key(code: KeyCode) -> Option<Key> {
    match code {
        KeyCode::Backquote => Some(Key::Backquote),
        KeyCode::Backslash => Some(Key::Backslash),
        KeyCode::BracketLeft => Some(Key::BracketLeft),
        KeyCode::BracketRight => Some(Key::BracketRight),
        KeyCode::Comma => Some(Key::Comma),
        KeyCode::Digit0 => Some(Key::Digit0),
        KeyCode::Digit1 => Some(Key::Digit1),
        KeyCode::Digit2 => Some(Key::Digit2),
        KeyCode::Digit3 => Some(Key::Digit3),
        KeyCode::Digit4 => Some(Key::Digit4),
        KeyCode::Digit5 => Some(Key::Digit5),
        KeyCode::Digit6 => Some(Key::Digit6),
        KeyCode::Digit7 => Some(Key::Digit7),
        KeyCode::Digit8 => Some(Key::Digit8),
        KeyCode::Digit9 => Some(Key::Digit9),
        KeyCode::Equal => Some(Key::Equal),
        KeyCode::IntlBackslash => Some(Key::IntlBackslash),
        KeyCode::IntlRo => Some(Key::IntlRo),
        KeyCode::IntlYen => Some(Key::IntlYen),
        KeyCode::KeyA => Some(Key::KeyA),
        KeyCode::KeyB => Some(Key::KeyB),
        KeyCode::KeyC => Some(Key::KeyC),
        KeyCode::KeyD => Some(Key::KeyD),
        KeyCode::KeyE => Some(Key::KeyE),
        KeyCode::KeyF => Some(Key::KeyF),
        KeyCode::KeyG => Some(Key::KeyG),
        KeyCode::KeyH => Some(Key::KeyH),
        KeyCode::KeyI => Some(Key::KeyI),
        KeyCode::KeyJ => Some(Key::KeyJ),
        KeyCode::KeyK => Some(Key::KeyK),
        KeyCode::KeyL => Some(Key::KeyL),
        KeyCode::KeyM => Some(Key::KeyM),
        KeyCode::KeyN => Some(Key::KeyN),
        KeyCode::KeyO => Some(Key::KeyO),
        KeyCode::KeyP => Some(Key::KeyP),
        KeyCode::KeyQ => Some(Key::KeyQ),
        KeyCode::KeyR => Some(Key::KeyR),
        KeyCode::KeyS => Some(Key::KeyS),
        KeyCode::KeyT => Some(Key::KeyT),
        KeyCode::KeyU => Some(Key::KeyU),
        KeyCode::KeyV => Some(Key::KeyV),
        KeyCode::KeyW => Some(Key::KeyW),
        KeyCode::KeyX => Some(Key::KeyX),
        KeyCode::KeyY => Some(Key::KeyY),
        KeyCode::KeyZ => Some(Key::KeyZ),
        KeyCode::Minus => Some(Key::Minus),
        KeyCode::Period => Some(Key::Period),
        KeyCode::Quote => Some(Key::Quote),
        KeyCode::Semicolon => Some(Key::Semicolon),
        KeyCode::Slash => Some(Key::Slash),
        KeyCode::AltLeft => Some(Key::AltLeft),
        KeyCode::AltRight => Some(Key::AltRight),
        KeyCode::Backspace => Some(Key::Backspace),
        KeyCode::CapsLock => Some(Key::CapsLock),
        KeyCode::ContextMenu => Some(Key::ContextMenu),
        KeyCode::ControlLeft => Some(Key::ControlLeft),
        KeyCode::ControlRight => Some(Key::ControlRight),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::SuperLeft => Some(Key::MetaLeft),
        KeyCode::SuperRight => Some(Key::MetaRight),
        KeyCode::ShiftLeft => Some(Key::ShiftLeft),
        KeyCode::ShiftRight => Some(Key::ShiftRight),
        KeyCode::Space => Some(Key::Space),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Convert => Some(Key::Convert),
        KeyCode::KanaMode => Some(Key::KanaMode),
        KeyCode::Lang1 => Some(Key::Lang1),
        KeyCode::Lang2 => Some(Key::Lang2),
        KeyCode::Lang3 => Some(Key::Lang3),
        KeyCode::Lang4 => Some(Key::Lang4),
        KeyCode::Lang5 => Some(Key::Lang5),
        KeyCode::NonConvert => Some(Key::NonConvert),
        KeyCode::Delete => Some(Key::Delete),
        KeyCode::End => Some(Key::End),
        KeyCode::Help => Some(Key::Help),
        KeyCode::Home => Some(Key::Home),
        KeyCode::Insert => Some(Key::Insert),
        KeyCode::PageDown => Some(Key::PageDown),
        KeyCode::PageUp => Some(Key::PageUp),
        KeyCode::ArrowDown => Some(Key::ArrowDown),
        KeyCode::ArrowLeft => Some(Key::ArrowLeft),
        KeyCode::ArrowRight => Some(Key::ArrowRight),
        KeyCode::ArrowUp => Some(Key::ArrowUp),
        KeyCode::NumLock => Some(Key::NumLock),
        KeyCode::Numpad0 => Some(Key::Numpad0),
        KeyCode::Numpad1 => Some(Key::Numpad1),
        KeyCode::Numpad2 => Some(Key::Numpad2),
        KeyCode::Numpad3 => Some(Key::Numpad3),
        KeyCode::Numpad4 => Some(Key::Numpad4),
        KeyCode::Numpad5 => Some(Key::Numpad5),
        KeyCode::Numpad6 => Some(Key::Numpad6),
        KeyCode::Numpad7 => Some(Key::Numpad7),
        KeyCode::Numpad8 => Some(Key::Numpad8),
        KeyCode::Numpad9 => Some(Key::Numpad9),
        KeyCode::NumpadAdd => Some(Key::NumpadAdd),
        KeyCode::NumpadBackspace => Some(Key::NumpadBackspace),
        KeyCode::NumpadClear => Some(Key::NumpadClear),
        KeyCode::NumpadClearEntry => Some(Key::NumpadClearEntry),
        KeyCode::NumpadComma => Some(Key::NumpadComma),
        KeyCode::NumpadDecimal => Some(Key::NumpadDecimal),
        KeyCode::NumpadDivide => Some(Key::NumpadDivide),
        KeyCode::NumpadEnter => Some(Key::NumpadEnter),
        KeyCode::NumpadEqual => Some(Key::NumpadEqual),
        KeyCode::NumpadHash => Some(Key::NumpadHash),
        KeyCode::NumpadMemoryAdd => Some(Key::NumpadMemoryAdd),
        KeyCode::NumpadMemoryClear => Some(Key::NumpadMemoryClear),
        KeyCode::NumpadMemoryRecall => Some(Key::NumpadMemoryRecall),
        KeyCode::NumpadMemoryStore => Some(Key::NumpadMemoryStore),
        KeyCode::NumpadMemorySubtract => Some(Key::NumpadMemorySubtract),
        KeyCode::NumpadMultiply => Some(Key::NumpadMultiply),
        KeyCode::NumpadParenLeft => Some(Key::NumpadParenLeft),
        KeyCode::NumpadParenRight => Some(Key::NumpadParenRight),
        KeyCode::NumpadStar => Some(Key::NumpadStar),
        KeyCode::NumpadSubtract => Some(Key::NumpadSubtract),
        KeyCode::Escape => Some(Key::Escape),
        KeyCode::Fn => Some(Key::Fn),
        KeyCode::FnLock => Some(Key::FnLock),
        KeyCode::PrintScreen => Some(Key::PrintScreen),
        KeyCode::ScrollLock => Some(Key::ScrollLock),
        KeyCode::Pause => Some(Key::Pause),
        KeyCode::BrowserBack => Some(Key::BrowserBack),
        KeyCode::BrowserFavorites => Some(Key::BrowserFavorites),
        KeyCode::BrowserForward => Some(Key::BrowserForward),
        KeyCode::BrowserHome => Some(Key::BrowserHome),
        KeyCode::BrowserRefresh => Some(Key::BrowserRefresh),
        KeyCode::BrowserSearch => Some(Key::BrowserSearch),
        KeyCode::BrowserStop => Some(Key::BrowserStop),
        KeyCode::Eject => Some(Key::Eject),
        KeyCode::LaunchApp1 => Some(Key::LaunchApp1),
        KeyCode::LaunchApp2 => Some(Key::LaunchApp2),
        KeyCode::LaunchMail => Some(Key::LaunchMail),
        KeyCode::MediaPlayPause => Some(Key::MediaPlayPause),
        KeyCode::MediaSelect => Some(Key::MediaSelect),
        KeyCode::MediaStop => Some(Key::MediaStop),
        KeyCode::MediaTrackNext => Some(Key::MediaTrackNext),
        KeyCode::MediaTrackPrevious => Some(Key::MediaTrackPrevious),
        KeyCode::Power => Some(Key::Power),
        KeyCode::Sleep => Some(Key::Sleep),
        KeyCode::AudioVolumeDown => Some(Key::AudioVolumeDown),
        KeyCode::AudioVolumeMute => Some(Key::AudioVolumeMute),
        KeyCode::AudioVolumeUp => Some(Key::AudioVolumeUp),
        KeyCode::WakeUp => Some(Key::WakeUp),
        KeyCode::Meta => Some(Key::Super),
        KeyCode::Hyper => Some(Key::Hyper),
        KeyCode::Turbo => Some(Key::Turbo),
        KeyCode::Abort => Some(Key::Abort),
        KeyCode::Resume => Some(Key::Resume),
        KeyCode::Suspend => Some(Key::Suspend),
        KeyCode::Again => Some(Key::Again),
        KeyCode::Copy => Some(Key::Copy),
        KeyCode::Cut => Some(Key::Cut),
        KeyCode::Find => Some(Key::Find),
        KeyCode::Open => Some(Key::Open),
        KeyCode::Paste => Some(Key::Paste),
        KeyCode::Props => Some(Key::Props),
        KeyCode::Select => Some(Key::Select),
        KeyCode::Undo => Some(Key::Undo),
        KeyCode::Hiragana => Some(Key::Hiragana),
        KeyCode::Katakana => Some(Key::Katakana),
        KeyCode::F1 => Some(Key::F1),
        KeyCode::F2 => Some(Key::F2),
        KeyCode::F3 => Some(Key::F3),
        KeyCode::F4 => Some(Key::F4),
        KeyCode::F5 => Some(Key::F5),
        KeyCode::F6 => Some(Key::F6),
        KeyCode::F7 => Some(Key::F7),
        KeyCode::F8 => Some(Key::F8),
        KeyCode::F9 => Some(Key::F9),
        KeyCode::F10 => Some(Key::F10),
        KeyCode::F11 => Some(Key::F11),
        KeyCode::F12 => Some(Key::F12),
        KeyCode::F13 => Some(Key::F13),
        KeyCode::F14 => Some(Key::F14),
        KeyCode::F15 => Some(Key::F15),
        KeyCode::F16 => Some(Key::F16),
        KeyCode::F17 => Some(Key::F17),
        KeyCode::F18 => Some(Key::F18),
        KeyCode::F19 => Some(Key::F19),
        KeyCode::F20 => Some(Key::F20),
        KeyCode::F21 => Some(Key::F21),
        KeyCode::F22 => Some(Key::F22),
        KeyCode::F23 => Some(Key::F23),
        KeyCode::F24 => Some(Key::F24),
        KeyCode::F25 => Some(Key::F25),
        KeyCode::F26 => Some(Key::F26),
        KeyCode::F27 => Some(Key::F27),
        KeyCode::F28 => Some(Key::F28),
        KeyCode::F29 => Some(Key::F29),
        KeyCode::F30 => Some(Key::F30),
        KeyCode::F31 => Some(Key::F31),
        KeyCode::F32 => Some(Key::F32),
        KeyCode::F33 => Some(Key::F33),
        KeyCode::F34 => Some(Key::F34),
        KeyCode::F35 => Some(Key::F35),
        _ => None,
    }
}

pub(crate) fn to_gamepad_button(button: gilrs::Button) -> Option<GamepadButton> {
    match button {
        gilrs::Button::South => Some(GamepadButton::South),
        gilrs::Button::East => Some(GamepadButton::East),
        gilrs::Button::North => Some(GamepadButton::North),
        gilrs::Button::West => Some(GamepadButton::West),
        gilrs::Button::C => Some(GamepadButton::C),
        gilrs::Button::Z => Some(GamepadButton::Z),
        gilrs::Button::LeftTrigger => Some(GamepadButton::FrontLeftTrigger),
        gilrs::Button::LeftTrigger2 => Some(GamepadButton::BackLeftTrigger),
        gilrs::Button::RightTrigger => Some(GamepadButton::FrontRightTrigger),
        gilrs::Button::RightTrigger2 => Some(GamepadButton::BackRightTrigger),
        gilrs::Button::Select => Some(GamepadButton::Select),
        gilrs::Button::Start => Some(GamepadButton::Start),
        gilrs::Button::Mode => Some(GamepadButton::Mode),
        gilrs::Button::LeftThumb => Some(GamepadButton::LeftStick),
        gilrs::Button::RightThumb => Some(GamepadButton::RightStick),
        gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
        gilrs::Button::Unknown => None,
    }
}