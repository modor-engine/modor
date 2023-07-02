use modor_resources::{
    Load, ResKey, Resource, ResourceHandler, ResourceLoadingError, ResourceSource, ResourceState,
};
use std::thread;
use std::time::Duration;

const RESOURCE: ResKey<SizeResource> = ResKey::new("size");

#[modor_test(disabled(wasm))]
fn load_valid_resource_from_sync_data() {
    let source = ResourceSource::SyncData("text".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    assert_eq!(handler.resource(), Some(LoadedSize(4)));
    assert_eq!(handler.state(), ResourceState::Loaded);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loaded);
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_invalid_resource_from_sync_data() {
    let source = ResourceSource::SyncData(String::new());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    let error = ResourceLoadingError::LoadingError("empty data".into());
    assert_eq!(handler.state(), ResourceState::Error(&error));
    assert_eq!(handler.resource(), None);
    assert_eq!(handler.state(), ResourceState::Error(&error));
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Error(&error));
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_valid_resource_from_async_data() {
    let source = ResourceSource::AsyncData("text".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    for _ in 0..1000 {
        handler.update(RESOURCE);
        if let Some(resource) = handler.resource() {
            assert_eq!(resource, LoadedSize(4));
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(handler.state(), ResourceState::Loaded);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loaded);
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_invalid_resource_from_async_data() {
    let source = ResourceSource::AsyncData(String::new());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    for _ in 0..1000 {
        handler.update(RESOURCE);
        if handler.state() != ResourceState::Loading {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    let error = ResourceLoadingError::LoadingError("empty data".into());
    assert_eq!(handler.state(), ResourceState::Error(&error));
    assert_eq!(handler.resource(), None);
    assert_eq!(handler.state(), ResourceState::Error(&error));
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Error(&error));
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_resource_from_async_data_that_panics_during_loading() {
    let source = ResourceSource::AsyncData("x".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    for _ in 0..1000 {
        handler.update(RESOURCE);
        if handler.state() != ResourceState::Loading {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(
        handler.state(),
        ResourceState::Error(&ResourceLoadingError::LoadingError(_))
    ));
    assert_eq!(handler.resource(), None);
    assert!(matches!(
        handler.state(),
        ResourceState::Error(&ResourceLoadingError::LoadingError(_))
    ));
    handler.update(RESOURCE);
    assert!(matches!(
        handler.state(),
        ResourceState::Error(&ResourceLoadingError::LoadingError(_))
    ));
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_valid_resource_from_async_path() {
    let source = ResourceSource::AsyncPath("not_empty.txt".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    for _ in 0..1000 {
        handler.update(RESOURCE);
        if let Some(resource) = handler.resource() {
            assert_eq!(resource, LoadedSize(12));
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(handler.state(), ResourceState::Loaded);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loaded);
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_invalid_resource_from_valid_async_path() {
    let source = ResourceSource::AsyncPath("empty.txt".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    for _ in 0..1000 {
        handler.update(RESOURCE);
        if handler.state() != ResourceState::Loading {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    let error = ResourceLoadingError::LoadingError("empty file".into());
    assert_eq!(handler.state(), ResourceState::Error(&error));
    assert_eq!(handler.resource(), None);
    assert_eq!(handler.state(), ResourceState::Error(&error));
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Error(&error));
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn load_invalid_resource_from_invalid_async_path() {
    let source = ResourceSource::AsyncPath("invalid.txt".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    assert_eq!(handler.resource(), None);
    handler.update(RESOURCE);
    assert_eq!(handler.state(), ResourceState::Loading);
    for _ in 0..1000 {
        handler.update(RESOURCE);
        if handler.state() != ResourceState::Loading {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(
        handler.state(),
        ResourceState::Error(ResourceLoadingError::AssetLoadingError(_))
    ));
    assert_eq!(handler.resource(), None);
    assert!(matches!(
        handler.state(),
        ResourceState::Error(ResourceLoadingError::AssetLoadingError(_))
    ));
    handler.update(RESOURCE);
    assert!(matches!(
        handler.state(),
        ResourceState::Error(ResourceLoadingError::AssetLoadingError(_))
    ));
    assert_eq!(handler.resource(), None);
}

#[modor_test(disabled(wasm))]
fn reload_resource() {
    let source = ResourceSource::SyncData("text".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    handler.update(RESOURCE);
    handler.resource().unwrap();
    handler.reload();
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    handler.update(RESOURCE);
    assert_eq!(handler.resource(), Some(LoadedSize(4)));
    assert_eq!(handler.state(), ResourceState::Loaded);
}

#[modor_test(disabled(wasm))]
fn change_resource_source() {
    let source = ResourceSource::SyncData("text".into());
    let mut handler = ResourceHandler::<LoadedSize, _>::new(source);
    handler.update(RESOURCE);
    handler.resource().unwrap();
    handler.set_source(ResourceSource::SyncData("longer text".into()));
    assert_eq!(handler.state(), ResourceState::NotLoaded);
    handler.update(RESOURCE);
    assert_eq!(handler.resource(), Some(LoadedSize(11)));
    assert_eq!(handler.state(), ResourceState::Loaded);
}

struct SizeResource(ResKey<Self>);

impl Resource for SizeResource {
    fn key(&self) -> ResKey<Self> {
        self.0
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::NotLoaded
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LoadedSize(usize);

impl Load<String> for LoadedSize {
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError> {
        thread::sleep(Duration::from_millis(2));
        if data.is_empty() {
            Err(ResourceLoadingError::LoadingError("empty file".into()))
        } else if data.len() == 1 {
            panic!("example of panic during loading");
        } else {
            Ok(Self(data.len()))
        }
    }

    fn load_from_data(data: &String) -> Result<Self, ResourceLoadingError> {
        thread::sleep(Duration::from_millis(2));
        if data.is_empty() {
            Err(ResourceLoadingError::LoadingError("empty data".into()))
        } else if data.len() == 1 {
            panic!("example of panic during loading");
        } else {
            Ok(Self(data.len()))
        }
    }
}
