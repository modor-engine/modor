use crate::components::mesh::MeshKey;
use crate::{IntoResourceKey, ResourceKey};

#[must_use]
#[derive(Component, NoSystem)]
pub struct Model {
    pub material_key: ResourceKey,
    pub camera_keys: Vec<ResourceKey>,
    pub(crate) mesh_key: ResourceKey,
}

impl Model {
    pub fn rectangle(material_key: impl IntoResourceKey) -> Self {
        Self {
            material_key: material_key.into_key(),
            camera_keys: vec![],
            mesh_key: MeshKey::Rectangle.into_key(),
        }
    }

    pub fn with_camera_key(mut self, key: impl IntoResourceKey) -> Self {
        self.camera_keys.push(key.into_key());
        self
    }
}
