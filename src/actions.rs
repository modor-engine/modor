use std::any::{Any, TypeId};

/// A trait for defining an action.
///
/// Actions are used to constrain systems.
///
/// ## Examples
///
/// See [`EntityRunner`](crate::EntityRunner).
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
    use super::*;
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
    use super::*;

    struct A1;
    struct A2;
    struct A3;
    struct A4;
    struct A5;
    struct A6;
    struct A7;
    struct A8;
    struct A9;
    struct A10;

    macro_rules! impl_actions {
        ($($types:ty),*) => {
            $(impl Action for $types {
                type Constraint = ();
            })*
        };
    }

    impl_actions!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);

    macro_rules! test_tuple_dependency_types {
        ($($params:ident),*) => {{
            let dependency_types = <($(DependsOn<$params>,)*)>::dependency_types();

            assert_eq!(dependency_types, vec![$(TypeId::of::<$params>()),*]);
        }};
    }

    #[test]
    fn retrieve_dependency_types_for_empty_tuple() {
        test_tuple_dependency_types!();
    }

    #[test]
    fn retrieve_dependency_types_for_1_item_tuple() {
        test_tuple_dependency_types!(A1);
    }

    #[test]
    fn retrieve_dependency_types_for_2_item_tuple() {
        test_tuple_dependency_types!(A1, A2);
    }

    #[test]
    fn retrieve_dependency_types_for_3_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3);
    }

    #[test]
    fn retrieve_dependency_types_for_4_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4);
    }

    #[test]
    fn retrieve_dependency_types_for_5_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4, A5);
    }

    #[test]
    fn retrieve_dependency_types_for_6_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4, A5, A6);
    }

    #[test]
    fn retrieve_dependency_types_for_7_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4, A5, A6, A7);
    }

    #[test]
    fn retrieve_dependency_types_for_8_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4, A5, A6, A7, A8);
    }

    #[test]
    fn retrieve_dependency_types_for_9_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
    }

    #[test]
    fn retrieve_dependency_types_for_10_item_tuple() {
        test_tuple_dependency_types!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
    }
}
