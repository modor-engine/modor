use crate::actions::internal::SealedConstraint;
use std::any::{Any, TypeId};

/// A trait for defining an action.
///
/// Actions are used to constrain systems.
///
/// **Do not implement manually this trait.**<br>
/// The [`action`](macro@crate::action) proc macro can be used instead to define an action.
///
/// # Examples
///
/// See [`entity`](macro@crate::entity).
pub trait Action: Any {
    #[doc(hidden)]
    fn dependency_types() -> Vec<TypeId>;
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

impl<A, C> Constraint for (A, C)
where
    A: Action,
    C: Constraint,
{
    fn action_types() -> Vec<TypeId> {
        let mut types = A::dependency_types();
        types.push(TypeId::of::<A>());
        types.extend(C::action_types());
        types
    }
}

mod internal {
    pub trait SealedConstraint {}

    impl SealedConstraint for () {}

    impl<A> SealedConstraint for (A,) {}

    impl<A, C> SealedConstraint for (A, C) {}
}
