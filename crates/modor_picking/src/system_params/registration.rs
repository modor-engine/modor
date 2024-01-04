use crate::NoPicking;
use modor::{Custom, Filter, Not, Query, SingleRef, With};
use modor_graphics::{Camera2D, RenderTarget, Texture};
use modor_resources::{ResourceAccessor, ResourceRegistry};

#[derive(SystemParam)]
pub(crate) struct RegistrationResources<'a> {
    cameras: Custom<ResourceAccessor<'a, Camera2D>>,
    target_registry: SingleRef<'a, 'static, ResourceRegistry<RenderTarget>>,
    targets: Query<
        'a,
        (
            &'static RenderTarget,
            Option<&'static Texture>,
            Filter<Not<With<NoPicking>>>,
        ),
    >,
}

impl<'a> RegistrationResources<'a> {
    pub(crate) fn as_ref<'b>(&'b self) -> RegistrationResourcesRef<'a, 'b> {
        RegistrationResourcesRef {
            cameras: &self.cameras,
            target_registry: self.target_registry.get(),
            targets: &self.targets,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RegistrationResourcesRef<'a, 'b> {
    pub(crate) cameras: &'b ResourceAccessor<'a, Camera2D>,
    pub(crate) target_registry: &'b ResourceRegistry<RenderTarget>,
    pub(crate) targets: &'b Query<
        'a,
        (
            // TODO: create query system param
            &'static RenderTarget,
            Option<&'static Texture>,
            Filter<Not<With<NoPicking>>>,
        ),
    >,
}
