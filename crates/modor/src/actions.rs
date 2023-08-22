use crate::actions::internal::SealedConstraint;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A trait for defining an action.
///
/// Actions are used to constrain systems.
///
/// **Do not implement manually this trait.**<br>
/// The [`Action`](macro@crate::Action) derive macro can be used instead to define an action.
pub trait Action: Any {
    #[doc(hidden)]
    fn dependency_types() -> Vec<TypeId>;
}

impl<T> Action for PhantomData<T>
where
    T: Any,
{
    fn dependency_types() -> Vec<TypeId> {
        vec![]
    }
}

#[doc(hidden)]
pub trait Constraint: SealedConstraint {
    fn action_types() -> Vec<TypeId>;
}

impl Constraint for () {
    fn action_types() -> Vec<TypeId> {
        vec![]
    }
}

impl<A> Constraint for (A,)
where
    A: Action,
{
    fn action_types() -> Vec<TypeId> {
        let mut types = A::dependency_types();
        types.push(TypeId::of::<A>());
        types
    }
}

impl<C, A> Constraint for (C, A)
where
    C: Constraint,
    A: Action,
{
    fn action_types() -> Vec<TypeId> {
        let mut types = A::dependency_types();
        types.extend(C::action_types());
        types.push(TypeId::of::<A>());
        types
    }
}

mod internal {
    pub trait SealedConstraint {}

    impl SealedConstraint for () {}

    impl<A> SealedConstraint for (A,) {}

    impl<A, C> SealedConstraint for (A, C) {}
}

#[cfg(test)]
mod action_tests {
    use crate::Action;
    use std::marker::PhantomData;

    #[test]
    fn retrieve_phantom_actions() {
        assert!(<PhantomData<()> as Action>::dependency_types().is_empty());
    }
}
