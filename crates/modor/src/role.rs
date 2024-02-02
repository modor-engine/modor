use std::any;
use std::any::{Any, TypeId};

/// A trait for defining a role that helps to order update of objects.
///
/// # Circular dependencies
///
/// Circular dependencies are checked at runtime. Note that all roles are not necessarily checked
/// during [`App`](crate::App) creation.
///
/// If a circular dependencies is detected, an error is logged and the cycle is arbitrarily broken
/// to ensure correct [`App`](crate::App) execution.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait Role: Any {
    /// Returns the list of constraints of the role.
    fn constraints() -> Vec<RoleConstraint>;
}

/// The default role.
///
/// This role does not have any constraint.
///
/// # Examples
///
/// See [`modor`](crate).
pub struct NoRole;

impl Role for NoRole {
    fn constraints() -> Vec<RoleConstraint> {
        vec![]
    }
}

/// A role constraint.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug)]
pub struct RoleConstraint {
    pub(crate) other_role: RoleType,
    pub(crate) type_: RoleConstraintType,
}

impl RoleConstraint {
    /// Creates a constraint indicating that the objects linked to the role will be run before
    /// objects linked to `T` role.
    pub fn before<T>() -> Self
    where
        T: Role,
    {
        Self {
            other_role: RoleType::new::<T>(),
            type_: RoleConstraintType::Before,
        }
    }

    /// Creates a constraint indicating that the objects linked to the role will be run after
    /// objects linked to `T` role.
    pub fn after<T>() -> Self
    where
        T: Role,
    {
        Self {
            other_role: RoleType::new::<T>(),
            type_: RoleConstraintType::After,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RoleType {
    pub(crate) type_id: TypeId,
    pub(crate) name: &'static str,
}

impl RoleType {
    pub(crate) fn new<T>() -> Self
    where
        T: Role,
    {
        Self {
            type_id: TypeId::of::<T>(),
            name: any::type_name::<T>(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum RoleConstraintType {
    Before,
    After,
}
