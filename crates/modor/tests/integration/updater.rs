use modor::{FromApp, State, Updater};
use std::marker::PhantomData;

#[modor::test]
fn run_update() {
    let mut value = Value::default();
    ValueUpdater::default()
        .integer(10) // is overwritten by next line
        .for_integer(|i| *i += 1)
        .for_string(String::pop)
        .additional_integer(20_u16)
        .apply(&mut value);
    assert_eq!(value.integer, 26);
    assert_eq!(value.string, "abc");
}

#[modor::test]
fn run_generic_update() {
    let mut value = GenericValue::<2, &str>::default();
    GenericValueUpdater::default()
        .integer(10)
        .dynamic("abc")
        .for_array(|array| array[0] = 10)
        .apply(&mut value);
    assert_eq!(value.integer, 10);
    assert_eq!(value.dynamic, "abc");
    assert_eq!(value.array[0], 10);
    assert_eq!(value.array[1], 0);
}

#[derive(FromApp, State)]
struct Root;

#[derive(Updater)]
struct Value {
    #[updater(field, for_field)]
    integer: u8,
    #[updater(for_field)]
    string: String,
    #[updater(inner_type, field)]
    additional_integer: PhantomData<u16>,
}

impl Default for Value {
    fn default() -> Self {
        Self {
            integer: 5,
            string: "abcd".into(),
            additional_integer: PhantomData,
        }
    }
}

impl ValueUpdater<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn apply(mut self, value: &mut Value) {
        self.integer.apply(&mut value.integer);
        self.string.apply(&mut value.string);
        if let Some(additional_integer) = self.additional_integer.take_value(|| 0) {
            value.integer += additional_integer as u8;
        }
    }
}

#[derive(Updater)]
struct GenericValue<const N: usize, T: 'static> {
    #[updater(field)]
    integer: u8,
    #[updater(field)]
    dynamic: T,
    #[updater(for_field)]
    array: [u16; N],
}

impl<const N: usize, T> Default for GenericValue<N, T>
where
    T: 'static + Default,
{
    fn default() -> Self {
        Self {
            integer: 5,
            dynamic: T::default(),
            array: [0; N],
        }
    }
}

impl<const N: usize, T> GenericValueUpdater<'_, N, T>
where
    T: 'static + Default + PartialEq,
{
    fn apply(mut self, value: &mut GenericValue<N, T>) {
        self.integer.apply(&mut value.integer);
        self.dynamic.apply(&mut value.dynamic);
        self.array.apply(&mut value.array);
    }
}
