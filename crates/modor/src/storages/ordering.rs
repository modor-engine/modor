use crate::role::{RoleConstraintType, RoleType};
use crate::{Object, Role};
use fxhash::{FxHashMap, FxHashSet};
use log::error;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{Dfs, Topo};
use std::any::TypeId;

#[derive(Default, Debug)]
pub(crate) struct OrderingStorage {
    registered_object_types: FxHashSet<TypeId>,
    role_order: DiGraphMap<RoleType, ()>,
    role_object_types: FxHashMap<TypeId, Vec<TypeId>>,
}

impl OrderingStorage {
    pub(crate) fn register<T>(&mut self)
    where
        T: Object,
    {
        if self.registered_object_types.insert(TypeId::of::<T>()) {
            self.role_object_types
                .entry(TypeId::of::<T::Role>())
                .or_default()
                .push(TypeId::of::<T>());
            self.register_role::<T::Role>();
        }
    }

    pub(crate) fn sorted_types(&self) -> Vec<TypeId> {
        let mut sorted_object_types = Vec::new();
        let mut walker = Topo::new(&self.role_order);
        while let Some(role_type) = walker.next(&self.role_order) {
            sorted_object_types.extend_from_slice(&self.role_object_types[&role_type.type_id]);
        }
        sorted_object_types
    }

    fn register_role<T>(&mut self)
    where
        T: Role,
    {
        let type_ = RoleType::new::<T>();
        self.role_order.add_node(type_);
        for constraint in T::constraints() {
            match constraint.type_ {
                RoleConstraintType::Before => self.add_dependency(type_, constraint.other_role),
                RoleConstraintType::After => self.add_dependency(constraint.other_role, type_),
            };
        }
    }

    fn add_dependency(&mut self, previous_type: RoleType, next_type: RoleType) {
        let mut walker = Dfs::new(&self.role_order, next_type);
        let mut cycle_types = vec![previous_type.name];
        while let Some(type_) = walker.next(&self.role_order) {
            cycle_types.push(type_.name);
            if type_ == previous_type {
                Self::log_circular_dependency(previous_type, next_type, &cycle_types);
                return;
            }
        }
        self.role_order.add_edge(previous_type, next_type, ());
    }

    fn log_circular_dependency(previous_type: RoleType, next_type: RoleType, cycle_types: &[&str]) {
        let cycle = cycle_types
            .iter()
            .map(|t| format!("`{t}`"))
            .collect::<Vec<_>>()
            .join(" before ");
        error!("Circular dependency detected between roles ({})", cycle);
        error!(
            "`{}` role will not be run before `{}` to avoid circular dependency",
            previous_type.name, next_type.name
        );
    }
}
