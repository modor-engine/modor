use crate::{platform, FromApp, State};
use derivative::Derivative;
use fxhash::FxHashMap;
use log::{debug, Level};
use std::any;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// The entrypoint of the engine.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug)]
pub struct App {
    state_indexes: FxHashMap<TypeId, usize>,
    states: Vec<StateData>, // ensures deterministic update order
}

impl App {
    /// Creates a new app with an initial state of type `T`.
    ///
    /// This also configures logging with a minimum `log_level` to display.
    ///
    /// # Platform-specific
    ///
    /// - Web: logging is initialized using the `console_log` crate and panic hook using the
    ///     `console_error_panic_hook` crate.
    /// - Other: logging is initialized using the `pretty_env_logger` crate.
    pub fn new<T>(log_level: Level) -> Self
    where
        T: State,
    {
        platform::init_logging(log_level);
        debug!("Initialize app...");
        let mut app = Self {
            state_indexes: FxHashMap::default(),
            states: vec![],
        };
        app.get_mut::<T>();
        debug!("App initialized");
        app
    }

    /// Update all states registered in the app.
    ///
    /// [`State::update`] method is called for each registered state.
    ///
    /// States are updated in the order in which they are created.
    ///
    /// # Panics
    ///
    /// This will panic if any state is already borrowed.
    pub fn update(&mut self) {
        debug!("Run update app...");
        for state_index in 0..self.states.len() {
            let state = &mut self.states[state_index];
            let mut value = state.value.take().expect("state is already borrowed");
            let update_fn = state.update_fn;
            update_fn(&mut *value, self);
            self.states[state_index].value = Some(value);
        }
        debug!("App updated");
    }

    /// Returns a handle to a state.
    ///
    /// The state is created using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`State::init`] if it doesn't exist.
    pub fn handle<T>(&mut self) -> StateHandle<T>
    where
        T: State,
    {
        StateHandle {
            index: self.state_index_or_create::<T>(),
            phantom: PhantomData,
        }
    }

    /// Creates the state of type `T` using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`State::init`] if it doesn't exist.
    pub fn create<T>(&mut self)
    where
        T: State,
    {
        self.handle::<T>();
    }

    /// Returns a mutable reference to a state.
    ///
    /// The state is created using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`State::init`] if it doesn't exist.
    ///
    /// # Panics
    ///
    /// This will panic if state `T` is already borrowed.
    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: State,
    {
        let state_index = self.state_index_or_create::<T>();
        self.state_mut(state_index)
    }

    /// Borrows a state without borrowing the app.
    ///
    /// The method returns the output of `f`.
    ///
    /// The state is created using [`FromApp::from_app`](crate::FromApp::from_app)
    /// and [`State::init`] if it doesn't exist.
    ///
    /// This method is useful when it is needed to have a mutable reference to multiple states.
    ///
    /// # Panics
    ///
    /// This will panic if state `T` is already borrowed.
    pub fn take<T, O>(&mut self, f: impl FnOnce(&mut T, &mut Self) -> O) -> O
    where
        T: State,
    {
        let state_index = self.state_index_or_create::<T>();
        self.take_state(state_index, f)
    }

    #[allow(clippy::map_entry)]
    fn state_index_or_create<T>(&mut self) -> usize
    where
        T: State,
    {
        let type_id = TypeId::of::<T>();
        if self.state_indexes.contains_key(&type_id) {
            self.state_indexes[&type_id]
        } else {
            debug!("Create state `{}`...", any::type_name::<T>());
            let state = StateData::new(T::from_app_with(self, T::init));
            debug!("State `{}` created", any::type_name::<T>());
            let index = self.states.len();
            self.state_indexes.insert(type_id, index);
            self.states.push(state);
            index
        }
    }

    fn state_mut<T>(&mut self, state_index: usize) -> &mut T
    where
        T: State,
    {
        self.states[state_index]
            .value
            .as_mut()
            .unwrap_or_else(|| panic!("state `{}` already borrowed", any::type_name::<T>()))
            .downcast_mut::<T>()
            .expect("internal error: misconfigured state")
    }

    fn take_state<T, O>(&mut self, state_index: usize, f: impl FnOnce(&mut T, &mut Self) -> O) -> O
    where
        T: State,
    {
        let state = &mut self.states[state_index];
        let mut value = state
            .value
            .take()
            .unwrap_or_else(|| panic!("state `{}` already borrowed", any::type_name::<T>()));
        let value_ref = value
            .downcast_mut()
            .expect("internal error: misconfigured state");
        let result = f(value_ref, self);
        self.states[state_index].value = Some(value);
        result
    }
}

/// A handle to access a [`State`].
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = "")
)]
pub struct StateHandle<T> {
    index: usize,
    phantom: PhantomData<fn(T)>,
}

impl<T> FromApp for StateHandle<T>
where
    T: State,
{
    fn from_app(app: &mut App) -> Self {
        app.handle()
    }
}

impl<T> StateHandle<T>
where
    T: State,
{
    /// Returns an immutable reference to the state.
    ///
    /// # Panics
    ///
    /// This will panic if the state is already borrowed.
    pub fn get(self, app: &App) -> &T {
        app.states[self.index]
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("state `{}` already borrowed", any::type_name::<T>()))
            .downcast_ref::<T>()
            .expect("internal error: misconfigured state")
    }

    /// Returns a mutable reference to the state.
    ///
    /// # Panics
    ///
    /// This will panic if the state is already borrowed.
    pub fn get_mut(self, app: &mut App) -> &mut T {
        app.state_mut(self.index)
    }

    /// Borrows a state without borrowing the app.
    ///
    /// The method returns the output of `f`.
    ///
    /// This method is useful when it is needed to have a mutable reference to multiple states.
    ///
    /// # Panics
    ///
    /// This will panic if the state is already borrowed.
    pub fn take<O>(self, app: &mut App, f: impl FnOnce(&mut T, &mut App) -> O) -> O {
        app.take_state(self.index, f)
    }
}

#[derive(Debug)]
struct StateData {
    value: Option<Box<dyn Any>>,
    update_fn: fn(&mut dyn Any, &mut App),
}

impl StateData {
    fn new<T>(value: T) -> Self
    where
        T: State,
    {
        Self {
            value: Some(Box::new(value)),
            update_fn: |value, app| {
                let value = value
                    .downcast_mut::<T>()
                    .expect("internal error: misconfigured state");
                T::update(value, app);
            },
        }
    }
}
