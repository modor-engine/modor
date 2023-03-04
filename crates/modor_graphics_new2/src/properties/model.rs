use crate::{IntoResourceKey, ResourceKey};

// TODO: mesh_key should be at model level and not material level (maybe needed to rename to Model2D)

#[must_use]
pub struct Model {
    pub material_key: ResourceKey,
    pub camera_keys: Vec<ResourceKey>,
}

#[component]
impl Model {
    pub fn new(material_key: impl IntoResourceKey) -> Self {
        Self {
            material_key: material_key.into_key(),
            camera_keys: vec![],
        }
    }

    pub fn with_camera_key(mut self, key: impl IntoResourceKey) -> Self {
        self.camera_keys.push(key.into_key());
        self
    }
}
