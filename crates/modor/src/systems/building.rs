use super::context::SystemContext;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use std::any::TypeId;

pub(crate) type SystemWrapper = fn(SystemContext<'_>);
pub(crate) type ArchetypeFilterFn = fn(&[TypeId]) -> bool;

#[doc(hidden)]
#[macro_export]
macro_rules! system {
    ($system:expr) => {{
        #[allow(unused_imports)] // traits are imported to perform compile time checks
        use $crate::{SystemWithParamMutabilityIssue, SystemWithParams};

        #[allow(clippy::semicolon_if_nothing_returned)]
        $crate::SystemBuilder {
            properties_fn: $crate::System::properties_fn(&$system),
            archetype_filter_fn: $crate::System::archetype_filter_fn(&$system),
            wrapper: |context| {
                let checker = $crate::SystemParamMutabilityChecker::new($system);
                let mut system = checker.check_param_mutability().into_inner();
                let mut guard = $crate::System::lock(&system, context);
                let mut guard_borrow = $crate::System::borrow_guard(&system, &mut guard);
                let mut stream = $crate::System::stream(&system, &mut guard_borrow);
                while let Some(item) = $crate::System::stream_next(&system, &mut stream) {
                    $crate::System::apply(&mut system, item);
                }
            },
        }
    }};
}

#[doc(hidden)]
pub struct SystemBuilder<S>
where
    S: FnMut(SystemContext<'_>),
{
    #[doc(hidden)]
    pub properties_fn: fn(&mut CoreStorage) -> SystemProperties,
    pub archetype_filter_fn: ArchetypeFilterFn,
    #[doc(hidden)]
    pub wrapper: S,
}
