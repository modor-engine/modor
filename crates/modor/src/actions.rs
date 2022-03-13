use std::any::{Any, TypeId};

/// A trait for defining an action.
///
/// Actions are used to constrain systems.
///
/// The [`action`](macro@crate::action) proc macro is a convenient way to define an action.
///
/// ## Examples
///
/// See [`SystemRunner`](crate::SystemRunner).
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

#[cfg(test)]
mod depends_on_tests {
    use crate::{Action, ActionConstraint, DependsOn};
    use std::any::TypeId;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    struct TestAction;

    impl Action for TestAction {
        type Constraint = ();
    }

    assert_impl_all!(DependsOn<TestAction>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);

    #[test]
    fn retrieve_dependency_types() {
        let dependency_types = DependsOn::<TestAction>::dependency_types();
        assert_eq!(dependency_types, vec![TypeId::of::<TestAction>()]);
    }
}

#[cfg(test)]
mod tuple_action_constraint_tests {
    use crate::{Action, ActionConstraint, DependsOn};
    use std::any::TypeId;

    macro_rules! define_actions {
        ($($types:ident),*) => {
            $(
                struct $types;

                impl Action for $types {
                    type Constraint = ();
                }
            )*
        };
    }

    define_actions!(A, B, C, D, E, F, G, H, I, J);

    macro_rules! test_tuple_dependency_types {
        ($(($params:ident, $indexes:tt)),*) => {{
            let dependency_types = <($(DependsOn<$params>,)*)>::dependency_types();
            assert_eq!(dependency_types, vec![$(TypeId::of::<$params>()),*]);
        }};
    }

    #[test]
    fn retrieve_dependency_types() {
        test_tuple_dependency_types!();
        run_for_tuples_with_idxs!(test_tuple_dependency_types);
    }
}
