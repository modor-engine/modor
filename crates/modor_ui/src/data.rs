#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum StyleProperty<T> {
    Fixed(T),
    Manual,
}
