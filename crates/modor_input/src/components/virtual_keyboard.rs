use crate::platform;

/// The state of the virtual keyboard.
///
/// For key state, use [`Keyboard`](crate::Keyboard) instead.
///
/// # Platform-specific
///
/// The actual virtual keyboard can only be controlled on Android.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// #[derive(Component)]
/// struct Button;
///
/// #[systems]
/// impl Button {
///     #[run_after(component(VirtualKeyboard))] // ensure opening precedence
///     fn on_click(mut virtual_keyboard: SingleMut<'_, '_, VirtualKeyboard>) {
///         virtual_keyboard.get_mut().open();
///     }
/// }
/// ```
#[derive(SingletonComponent, Debug, Default)]
pub struct VirtualKeyboard {
    is_open_requested: bool,
    is_close_requested: bool,
    is_open: bool,
}

#[systems]
impl VirtualKeyboard {
    /// Tell whether the keyboard is open.
    ///
    /// Note that the state can be incorrect in case the keyboard is opened or closed by an event
    /// generated outside [`VirtualKeyboard`].
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Request to open the virtual keyboard.
    ///
    /// It is highly recommended to mark a system running this method as depending on the
    /// [`VirtualKeyboard`] component to ensure that opening has higher priority than closing
    /// (e.g. with `#[run_after(component(VirtualKeyboard))]`).
    ///
    /// # Platform-specific
    ///
    /// The actual virtual keyboard can only be controlled on Android.
    pub fn open(&mut self) {
        self.is_open_requested = true;
    }

    /// Request to close the virtual keyboard.
    ///
    /// It is highly recommended to mark a system running this method as depending on the
    /// [`VirtualKeyboard`] component to ensure that opening has higher priority than closing
    /// (e.g. with `#[run_after(component(VirtualKeyboard))]`).
    ///
    /// # Platform-specific
    ///
    /// The actual virtual keyboard can only be controlled on Android.
    pub fn close(&mut self) {
        self.is_close_requested = true;
    }

    #[run]
    fn update(&mut self) {
        if self.is_open_requested {
            platform::open_virtual_keyboard();
            self.is_open = true;
        } else if self.is_close_requested {
            platform::close_virtual_keyboard();
            self.is_open = false;
        }
        self.is_open_requested = false;
        self.is_close_requested = false;
    }
}
