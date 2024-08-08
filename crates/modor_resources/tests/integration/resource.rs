use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_jobs::AssetLoadingError;
use modor_resources::{testing, Res, ResSource, Resource, ResourceError, ResourceState, Source};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[modor::test(disabled(wasm))]
fn update_inner() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    assert_eq!(res.get(&app).size, None);
    res.updater()
        .for_inner(&mut app, |res, _| res.size = Some(1))
        .apply(&mut app);
    assert_eq!(res.get(&app).size, Some(1));
}

#[modor::test(disabled(wasm))]
fn load_valid_resource_from_path() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater().path("not_empty.txt").apply(&mut app);
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &ResourceState::Loading);
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, Some(12));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
    app.update();
    assert_eq!(res.get(&app).size, Some(12));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
}

#[modor::test(disabled(wasm))]
fn load_invalid_resource_from_path() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater().path("empty.txt").apply(&mut app);
    let error = ResourceState::Error(ResourceError::Other("empty resource".into()));
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &error);
    app.update();
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &error);
}

#[modor::test(disabled(wasm))]
fn load_resource_from_invalid_path() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater().path("missing.txt").apply(&mut app);
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, None);
    assert!(matches!(
        res.get(&app).state().error(),
        Some(ResourceError::Loading(AssetLoadingError::IoError(_)))
    ));
    app.update();
    assert_eq!(res.get(&app).size, None);
    assert!(matches!(
        res.get(&app).state().error(),
        Some(ResourceError::Loading(AssetLoadingError::IoError(_)))
    ));
}

#[modor::test(
    disabled(wasm),
    cases(
        async_ = "ContentSizeSource::AsyncStr(Arc::new(Mutex::new(\"content\")))",
        sync = "ContentSizeSource::SyncStr(\"content\")"
    )
)]
fn load_valid_resource_from_source(source: ContentSizeSource) {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater().source(source).apply(&mut app);
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, Some(7));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
    app.update();
    assert_eq!(res.get(&app).size, Some(7));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
}

#[modor::test(
    disabled(wasm),
    cases(
        async_ = "ContentSizeSource::AsyncStr(Arc::new(Mutex::new(\"\")))",
        sync = "ContentSizeSource::SyncStr(\"\")"
    )
)]
fn load_invalid_resource_from_source(source: ContentSizeSource) {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater().source(source).apply(&mut app);
    let error = ResourceState::Error(ResourceError::Other("empty resource".into()));
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &error);
    app.update();
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &error);
}

#[modor::test(disabled(wasm))]
fn load_resource_from_panicking_source() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater()
        .source(ContentSizeSource::Panicking)
        .apply(&mut app);
    let error = ResourceState::Error(ResourceError::Other("job has panicked".into()));
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &error);
    app.update();
    assert_eq!(res.get(&app).size, None);
    assert_eq!(res.get(&app).state(), &error);
}

#[modor::test(disabled(wasm))]
fn set_source() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater()
        .source(ContentSizeSource::SyncStr("content"))
        .apply(&mut app);
    assert_eq!(res.get(&app).size, Some(7));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
    res.updater()
        .source(ContentSizeSource::SyncStr("other content"))
        .apply(&mut app);
    assert_eq!(res.get(&app).size, Some(13));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
}

#[modor::test(disabled(wasm))]
fn set_path() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater()
        .source(ContentSizeSource::SyncStr("content"))
        .apply(&mut app);
    assert_eq!(res.get(&app).size, Some(7));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
    res.updater().path("not_empty.txt").apply(&mut app);
    assert_eq!(res.get(&app).size, Some(7));
    assert_eq!(res.get(&app).state(), &ResourceState::Loading);
    testing::wait_resources(&mut app);
    assert_eq!(res.get(&app).size, Some(12));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
}

#[modor::test(disabled(wasm))]
fn reload_default() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.get_mut(&mut app).size = Some(42);
    res.updater().reload().apply(&mut app);
    assert_eq!(res.get(&app).size, Some(42));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
}

#[modor::test(disabled(wasm))]
fn reload_not_default() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Glob::<Res<ContentSize>>::from_app(&mut app);
    res.updater()
        .source(ContentSizeSource::SyncStr("content"))
        .apply(&mut app);
    assert_eq!(res.get(&app).size, Some(7));
    res.get_mut(&mut app).size = None;
    res.updater().reload().apply(&mut app);
    assert_eq!(res.get(&app).size, Some(7));
    assert_eq!(res.get(&app).state(), &ResourceState::Loaded);
}

#[derive(FromApp, State)]
struct Root;

#[derive(Default)]
struct ContentSize {
    size: Option<usize>,
}

impl Resource for ContentSize {
    type Source = ContentSizeSource;
    type Loaded = ContentSizeLoaded;

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        thread::sleep(Duration::from_millis(10));
        if file_bytes.is_empty() {
            Err(ResourceError::Other("empty resource".into()))
        } else {
            Ok(ContentSizeLoaded {
                size: file_bytes.len(),
            })
        }
    }

    fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        thread::sleep(Duration::from_millis(10));
        let size = match source {
            ContentSizeSource::AsyncStr(str) => str.lock().unwrap().len(),
            ContentSizeSource::SyncStr(str) => str.len(),
            ContentSizeSource::Panicking => panic!(),
        };
        if size == 0 {
            Err(ResourceError::Other("empty resource".into()))
        } else {
            Ok(ContentSizeLoaded { size })
        }
    }

    fn on_load(&mut self, _app: &mut App, loaded: Self::Loaded, _source: &ResSource<Self>) {
        self.size = Some(loaded.size);
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
enum ContentSizeSource {
    AsyncStr(Arc<Mutex<&'static str>>),
    SyncStr(&'static str),
    Panicking,
}

impl Default for ContentSizeSource {
    fn default() -> Self {
        Self::SyncStr("")
    }
}

impl Source for ContentSizeSource {
    fn is_async(&self) -> bool {
        match self {
            Self::AsyncStr(_) | Self::Panicking => true,
            Self::SyncStr(_) => false,
        }
    }
}

#[derive(Debug)]
struct ContentSizeLoaded {
    size: usize,
}
