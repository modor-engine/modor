use crate::{SystemData, SystemInfo};
use std::any::TypeId;

pub type SystemWrapper = fn(&SystemData<'_>, SystemInfo);

pub struct SystemBuilder {
    pub(crate) wrapper: SystemWrapper,
    pub(crate) component_types: Vec<TypeAccess>,
    pub(crate) actions: bool,
}

impl SystemBuilder {
    pub fn new(wrapper: SystemWrapper, component_types: Vec<TypeAccess>, actions: bool) -> Self {
        Self {
            wrapper,
            component_types,
            actions,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeAccess {
    Read(TypeId),
    Write(TypeId),
}

#[macro_export]
macro_rules! system {
    ($($systems:expr),+) => {{
        let mut types = Vec::new();
        $(types.extend(::modor::System::component_types(&$systems).into_iter());)+
        let mut actions = $(::modor::System::has_actions(&$systems))||+;
        ::modor::SystemBuilder::new(::modor::_system_wrapper!($($systems),+), types, actions)
    }};
}

#[macro_export]
macro_rules! for_each {
    ($query:expr, $system:expr) => {{
        let query: &::modor::Query<_> = &$query;
        let mut system = $system;
        let mut query_run = query.run(system);
        let mut system = query_run.system;
        let info =
            ::modor::SystemInfo::new(query_run.filtered_component_types, query_run.group_idx);
        _run_system!(&query_run.data, info, system);
    }};
}

#[macro_export]
macro_rules! for_each_mut {
    ($query:expr, $system:expr) => {{
        let query: &mut ::modor::Query<_> = &mut $query;
        let mut system = $system;
        let mut query_run = query.run_mut(system);
        let mut system = query_run.system;
        let info =
            ::modor::SystemInfo::new(query_run.filtered_component_types, query_run.group_idx);
        _run_system!(&query_run.data, info, system);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _system_wrapper {
    ($($systems:expr),+) => {
        |data: &::modor::SystemData<'_>, info: ::modor::SystemInfo| {
            use ::modor::SystemWithCorrectParams as _SystemWithCorrectParams;
            use ::modor::SystemWithMissingComponentParam as _SystemWithMissingComponentParam;
            use ::modor::SystemWithIncompatibleParams as _SystemWithIncompatibleParams;
            _run_system!(
                data,
                info,
                $(::modor::SystemStaticChecker::new($systems).check_statically()),+
            );
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _run_system {
    ($data:expr, $info:expr, $($systems:expr),+) => {
        let mut data = $data;
        let mut info = $info;
        $(
            let mut system = $systems;
            let mut locks = ::modor::System::lock(&system, data);
            if ::modor::System::has_mandatory_component(&system) {
                for archetype in ::modor::System::archetypes(&system, data, &info) {
                    ::modor::System::run(&mut system, data, &info, &mut locks, archetype);
                }
            } else {
                ::modor::System::run_once(&mut system, &info, &mut locks);
            }
        )+
    };
}

#[cfg(test)]
mod system_wrapper_tests {
    use super::*;

    assert_impl_all!(SystemWrapper: Sync, Send, Clone);
}

#[cfg(test)]
mod system_builder_tests {
    use super::*;

    assert_impl_all!(SystemBuilder: Sync, Send);
    assert_not_impl_any!(SystemBuilder: Clone);
}

#[cfg(test)]
mod type_access_tests {
    use super::*;
    use std::fmt::Debug;

    assert_impl_all!(TypeAccess: Sync, Send, Copy, Eq, Debug);
}
