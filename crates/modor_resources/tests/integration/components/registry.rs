use modor::{App, Custom, With};
use modor_jobs::AssetLoadingError;
use modor_resources::{
    ResKey, Resource, ResourceAccessor, ResourceAccessorMut, ResourceLoadingError,
    ResourceRegistry, ResourceState,
};

const VALUE1: ResKey<Value> = ResKey::new("val1");
const VALUE2: ResKey<Value> = ResKey::new("val2");
const VALUE3: ResKey<Value> = ResKey::new("val3");

#[modor_test]
fn retrieve_not_loaded_resource() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new(VALUE1, ResourceState::NotLoaded, 10))
        .with_entity(Value::new(VALUE2, ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new(VALUE1))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, None))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        });
}

#[modor_test]
fn retrieve_loading_resource() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new(VALUE1, ResourceState::Loading, 10))
        .with_entity(Value::new(VALUE2, ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new(VALUE1))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, None))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        });
}

#[modor_test]
fn retrieve_loaded_resource() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new(VALUE1, ResourceState::Loaded, 10))
        .with_entity(Value::new(VALUE2, ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new(VALUE1))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(10)))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, Some(10)))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        });
}

#[modor_test]
fn retrieve_error_resource() {
    const ERROR: ResourceLoadingError =
        ResourceLoadingError::AssetLoadingError(AssetLoadingError::InvalidAssetPath);
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new(VALUE1, ResourceState::Error(&ERROR), 10))
        .with_entity(Value::new(VALUE2, ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new(VALUE1))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, None))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        });
}

#[modor_test]
fn retrieve_resource_with_duplicated_key() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new(VALUE1, ResourceState::Loaded, 10))
        .with_entity(Value::new(VALUE1, ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new(VALUE1))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(20)))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, Some(20)))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        });
}

#[modor_test]
fn retrieve_resource_with_replaced_key() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(RetrievedValue::new(VALUE1))
        .with_entity(Value::new(VALUE1, ResourceState::Loaded, 10))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(10)))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, Some(10)))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        })
        .with_update::<(), _>(|v: &mut Value| *v = Value::new(VALUE2, ResourceState::Loaded, 20))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
                .has(|v: &RetrievedValue| assert!(!v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, None))
                .has(|v: &RetrievedValue| assert!(!v.exists_mut))
        })
        .with_update::<(), _>(|v: &mut Value| *v = Value::new(VALUE1, ResourceState::Loaded, 30))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, Some(30)))
                .has(|v: &RetrievedValue| assert!(v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, Some(30)))
                .has(|v: &RetrievedValue| assert!(v.exists_mut))
        });
}

#[modor_test]
fn retrieve_resource_with_missing_key() {
    App::new()
        .with_entity(ValueRegistry::default())
        .with_entity(Value::new(VALUE1, ResourceState::Loaded, 10))
        .with_entity(Value::new(VALUE2, ResourceState::Loaded, 20))
        .with_entity(RetrievedValue::new(VALUE3))
        .updated()
        .assert::<With<RetrievedValue>>(1, |e| {
            e.has(|v: &RetrievedValue| assert_eq!(v.value, None))
                .has(|v: &RetrievedValue| assert!(!v.exists))
                .has(|v: &RetrievedValue| assert_eq!(v.value_mut, None))
                .has(|v: &RetrievedValue| assert!(!v.exists_mut))
        });
}

type ValueRegistry = ResourceRegistry<Value>;

#[derive(Component, NoSystem)]
struct Value {
    key: ResKey<Self>,
    state: ResourceState<'static>,
    value: u32,
}

impl Value {
    fn new(key: ResKey<Self>, state: ResourceState<'static>, value: u32) -> Self {
        Self { key, state, value }
    }
}

impl Resource for Value {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        self.state.clone()
    }
}

#[derive(Component)]
struct RetrievedValue {
    key: ResKey<Value>,
    value: Option<u32>,
    exists: bool,
    value_mut: Option<u32>,
    exists_mut: bool,
}

#[systems]
impl RetrievedValue {
    fn new(key: ResKey<Value>) -> Self {
        Self {
            key,
            value: None,
            exists: false,
            value_mut: None,
            exists_mut: false,
        }
    }

    #[run_after(component(ValueRegistry), component(Value))]
    fn update(&mut self, values: Custom<ResourceAccessor<'_, Value>>) {
        self.value = values.get(self.key).map(|v| v.value);
        self.exists = values.registry.as_ref().unwrap().get().exists(self.key);
    }

    #[run_after(component(ValueRegistry), component(Value))]
    fn update_mut(&mut self, mut values: Custom<ResourceAccessorMut<'_, Value>>) {
        self.value_mut = values.get_mut(self.key).map(|v| v.value);
        self.exists_mut = values.registry.as_mut().unwrap().get_mut().exists(self.key);
    }
}
