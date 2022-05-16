use std::any::{Any, TypeId};

/// A trait for defining an action.
///
/// Actions are used to constrain systems.
///
/// The [`action`](macro@crate::action) proc macro is a convenient way to define an action.
///
/// ## Examples
///
/// See [`entity`](macro@crate::entity).
pub trait Action: Any {
    /// The constraint definition of the action.
    type Constraint: ActionConstraint;
}

/// A trait implemented for all types representing an action constraint.
pub trait ActionConstraint {
    #[doc(hidden)]
    fn dependency_types() -> Vec<TypeId>;
}

macro_rules! impl_tuple_action_constraint {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<$($params),*> ActionConstraint for ($($params,)*)
        where
            $($params: ActionConstraint,)*
        {
            #[allow(unused_mut)]
            fn dependency_types() -> Vec<TypeId> {
                let mut dependency_types = Vec::new();
                $(dependency_types.extend($params::dependency_types());)*
                dependency_types
            }
        }
    };
}
impl_tuple_action_constraint!();
run_for_tuples_with_idxs!(impl_tuple_action_constraint);

/// A type defining a dependency on an action.
pub struct DependsOn<A>(A::Constraint)
where
    A: Action;

impl<A> ActionConstraint for DependsOn<A>
where
    A: Action,
{
    fn dependency_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }
}
