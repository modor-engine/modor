use crate::components::material_converter::MaterialRegistration;
use crate::data::ManagedResources;
use modor::{SingleRef, World};
use modor_graphics::Material;
use modor_resources::{ResKey, ResourceRegistry};

#[derive(Debug, Default, SingletonComponent)]
pub(crate) struct ManagedMaterials {
    pub(crate) resources: ManagedResources<ResKey<Material>, Material>,
}

#[systems]
impl ManagedMaterials {
    #[run_as(action(MaterialReset))]
    fn reset(&mut self) {
        self.resources.reset();
    }

    #[run_as(action(MaterialNotRegisteredDeletion))]
    fn delete_not_registered(
        &mut self,
        material_registry: SingleRef<'_, '_, ResourceRegistry<Material>>,
        mut world: World<'_>,
    ) {
        self.resources
            .delete_not_registered(material_registry.get(), &mut world);
    }
}

#[derive(Action)]
pub(crate) struct MaterialReset;

#[derive(Action)]
pub(crate) struct MaterialNotRegisteredDeletion(MaterialRegistration);
