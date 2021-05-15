use crate::{SystemData, SystemInfo};
use std::any::TypeId;

pub type SystemWrapper = fn(&SystemData<'_>, SystemInfo);

pub struct SystemBuilder {
    pub(crate) wrapper: SystemWrapper,
    pub(crate) component_types: Vec<TypeAccess>,
    pub(crate) group_actions: bool,
}

impl SystemBuilder {
    pub fn new(
        wrapper: SystemWrapper,
        component_types: Vec<TypeAccess>,
        group_actions: bool,
    ) -> Self {
        Self {
            wrapper,
            component_types,
            group_actions,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeAccess {
    Read(TypeId),
    Write(TypeId),
}

impl TypeAccess {
    pub(crate) fn to_inner(&self) -> TypeId {
        match self {
            Self::Read(type_id) | Self::Write(type_id) => *type_id,
        }
    }
}

#[macro_export]
macro_rules! system {
    ($($system:expr),+) => {{
        let mut types = Vec::new();
        $(types.extend(::modor::System::component_types(&$system).into_iter());)+
        let mut group_actions = $(::modor::System::has_group_actions(&$system))&&+;
        ::modor::SystemBuilder::new(::modor::_system_wrapper!($($system),+), types, group_actions)
    }};
}

// TODO: move query type check from for_each(_mut) to system!() 
#[macro_export]
macro_rules! for_each {
    ($query:expr, $system:expr) => {{
        let query: &::modor::Query<_> = &$query;
        let mut system = $system;
        let mut query_run = query.run(system);
        let mut system = query_run.system;
        let info =
            ::modor::SystemInfo::new(query_run.filtered_component_types, query_run.group_idx);
        (::modor::_system_wrapper!(system))(&query_run.data, info);
    }};
}

#[macro_export]
macro_rules! for_each_mut {
    ($query:expr, $system:expr) => {{
        let query: &mut ::modor::QueryMut<_> = &mut $query;
        let mut system = $system;
        let mut query_run = query.run(system);
        let mut system = query_run.system;
        let info =
            ::modor::SystemInfo::new(query_run.filtered_component_types, query_run.group_idx);
        (::modor::_system_wrapper!(system))(&query_run.data, info);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _system_wrapper {
    ($($system:expr),+) => {
        |data: &::modor::SystemData<'_>, info: ::modor::SystemInfo| {
            use ::modor::SystemWithCorrectParams as _SystemWithCorrectParams;
            use ::modor::SystemWithMissingComponentParam as _SystemWithMissingComponentParam;
            use ::modor::SystemWithIncompatibleParams as _SystemWithIncompatibleParams;
            $(let mut system = ::modor::SystemStaticChecker::new($system).check_statically();
            let mut locks = ::modor::System::lock(&system, data);
            if ::modor::System::has_mandatory_component(&system) {
                for archetype in ::modor::System::archetypes(&system, data, &info) {
                    ::modor::System::run(&mut system, data, &info, &mut locks, archetype);
                }
            } else {
                ::modor::System::run_once(&mut system, &info, &mut locks);
            })+
        }
    };
}
