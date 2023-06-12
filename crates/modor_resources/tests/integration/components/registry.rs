use log::LevelFilter;
use modor::{App, Query, SingleMut, With};
use modor_jobs::AssetLoadingError;
use modor_resources::{
    IntoResourceKey, Resource, ResourceKey, ResourceLoadingError, ResourceRegistry, ResourceState,
};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_not_loaded_resource() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new("val1", ResourceState::NotLoaded, 10))
        .with_entity(Value::new("val2", ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new("val1"))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_loading_resource() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new("val1", ResourceState::Loading, 10))
        .with_entity(Value::new("val2", ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new("val1"))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_loaded_resource() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new("val1", ResourceState::Loaded, 10))
        .with_entity(Value::new("val2", ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new("val1"))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(10)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_error_resource() {
    const ERROR: ResourceLoadingError =
        ResourceLoadingError::AssetLoadingError(AssetLoadingError::InvalidAssetPath);
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new("val1", ResourceState::Error(&ERROR), 10))
        .with_entity(Value::new("val2", ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new("val1"))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_resource_with_duplicated_key() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new("val1", ResourceState::Loaded, 10))
        .with_entity(Value::new("val1", ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new("val1"))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(20)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_resource_with_replaced_key() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(RetrievedValue::new("val1"))
        .with_entity(Value::new("val1", ResourceState::Loaded, 10))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(10)))
        })
        .with_update::<(), _>(|v: &mut Value| *v = Value::new("val2", ResourceState::Loaded, 20))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
        })
        .with_update::<(), _>(|v: &mut Value| *v = Value::new("val1", ResourceState::Loaded, 30))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(30)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_resource_with_missing_key() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new("val1", ResourceState::Loaded, 10))
        .with_entity(Value::new("val2", ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new("val3"))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
        });
}

type ValueRegistry = ResourceRegistry<Value>;

#[derive(Component, NoSystem)]
struct Value {
    key: ResourceKey,
    state: ResourceState<'static>,
    value: u32,
}

impl Value {
    fn new(key: impl IntoResourceKey, state: ResourceState<'static>, value: u32) -> Self {
        Self {
            key: key.into_key(),
            state,
            value,
        }
    }
}

impl Resource for Value {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        self.state.clone()
    }
}

#[derive(Component)]
struct RetrievedValue {
    key: ResourceKey,
    value: Option<u32>,
}

#[systems]
impl RetrievedValue {
    fn new(key: impl IntoResourceKey) -> Self {
        Self {
            key: key.into_key(),
            value: None,
        }
    }

    #[run_after(component(ValueRegistry), component(Value))]
    fn update(&mut self, mut registry: SingleMut<'_, ValueRegistry>, values: Query<'_, &Value>) {
        self.value = registry.get(&self.key, &values).map(|v| v.value);
    }
}
