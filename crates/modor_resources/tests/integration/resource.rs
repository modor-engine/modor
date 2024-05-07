use modor::log::Level;
use modor::{App, Context, Node, RootNode, Visit};
use modor_jobs::AssetLoadingError;
use modor_resources::{Res, Resource, ResourceError, Source};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[modor::test(disabled(wasm))]
fn access_inner() {
    let mut app = App::new::<Root>(Level::Info);
    let mut res = Res::<ContentSize>::from_path(&mut app.ctx(), "res", "not_empty.txt");
    res.size = Some(1);
    assert_eq!(res.size, Some(1));
}

#[modor::test(disabled(wasm))]
fn load_valid_resource_from_path() {
    let mut app = App::new::<Root>(Level::Info);
    create_resource_from_path(&mut app, "not_empty.txt");
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), None);
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, Some(12));
    assert_eq!(res(&mut app).err(), None);
    app.update();
    assert_eq!(res(&mut app).size, Some(12));
    assert_eq!(res(&mut app).err(), None);
}

#[modor::test(disabled(wasm))]
fn load_invalid_resource_from_path() {
    let mut app = App::new::<Root>(Level::Info);
    create_resource_from_path(&mut app, "empty.txt");
    let error = ResourceError::Other("empty resource".into());
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), Some(&error));
    app.update();
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), Some(&error));
}

#[modor::test(disabled(wasm))]
fn load_resource_from_invalid_path() {
    let mut app = App::new::<Root>(Level::Info);
    create_resource_from_path(&mut app, "missing.txt");
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, None);
    assert!(matches!(
        res(&mut app).err(),
        Some(ResourceError::Loading(AssetLoadingError::IoError(_)))
    ));
    app.update();
    assert_eq!(res(&mut app).size, None);
    assert!(matches!(
        res(&mut app).err(),
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
    create_resource_from_source(&mut app, source);
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), None);
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, Some(7));
    assert_eq!(res(&mut app).err(), None);
    app.update();
    assert_eq!(res(&mut app).size, Some(7));
    assert_eq!(res(&mut app).err(), None);
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
    create_resource_from_source(&mut app, source);
    let error = ResourceError::Other("empty resource".into());
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), Some(&error));
    app.update();
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), Some(&error));
}

#[modor::test(disabled(wasm))]
fn load_resource_from_panicking_source() {
    let mut app = App::new::<Root>(Level::Info);
    create_resource_from_source(&mut app, ContentSizeSource::Panicking);
    let error = ResourceError::Other("job has panicked".into());
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), Some(&error));
    app.update();
    assert_eq!(res(&mut app).size, None);
    assert_eq!(res(&mut app).err(), Some(&error));
}

#[modor::test(disabled(wasm))]
fn reload_with_source() {
    let mut app = App::new::<Root>(Level::Info);
    create_resource_from_source(&mut app, ContentSizeSource::SyncStr("content"));
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, Some(7));
    assert_eq!(res(&mut app).err(), None);
    res(&mut app).reload_with_source(ContentSizeSource::SyncStr("other content"));
    app.update();
    assert_eq!(res(&mut app).size, Some(13));
    assert_eq!(res(&mut app).err(), None);
}

#[modor::test(disabled(wasm))]
fn reload_with_path() {
    let mut app = App::new::<Root>(Level::Info);
    create_resource_from_source(&mut app, ContentSizeSource::SyncStr("content"));
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, Some(7));
    assert_eq!(res(&mut app).err(), None);
    res(&mut app).reload_with_path("not_empty.txt");
    wait_resource_loaded(&mut app);
    assert_eq!(res(&mut app).size, Some(7));
    assert_eq!(res(&mut app).err(), None);
    app.update();
    wait_resource_reloaded(&mut app, 12);
    assert_eq!(res(&mut app).size, Some(12));
    assert_eq!(res(&mut app).err(), None);
}

fn create_resource_from_path(app: &mut App, path: &str) {
    app.root::<Root>().content_size = Some(Res::from_path(&mut app.ctx(), "res", path));
}

fn create_resource_from_source(app: &mut App, source: ContentSizeSource) {
    app.root::<Root>().content_size = Some(Res::from_source(&mut app.ctx(), "res", source));
}

fn wait_resource_loaded(app: &mut App) {
    const MAX_RETRIES: u32 = 100;
    for _ in 0..MAX_RETRIES {
        app.update();
        thread::sleep(Duration::from_millis(10));
        if res(app).err().is_some() || res(app).size.is_some() {
            return;
        }
    }
    panic!("max retries reached");
}

fn wait_resource_reloaded(app: &mut App, target_size: usize) {
    const MAX_RETRIES: u32 = 100;
    for _ in 0..MAX_RETRIES {
        app.update();
        thread::sleep(Duration::from_millis(10));
        if target_size == res(app).size.unwrap() {
            return;
        }
    }
    panic!("max retries reached");
}

fn res(app: &mut App) -> &mut Res<ContentSize> {
    app.root::<Root>().content_size.as_mut().unwrap()
}

#[derive(Default, RootNode, Node, Visit)]
struct Root {
    content_size: Option<Res<ContentSize>>,
}

#[derive(Default)]
struct ContentSize {
    size: Option<usize>,
}

impl Resource for ContentSize {
    type Source = ContentSizeSource;
    type Loaded = ContentSizeLoaded;

    fn create(_ctx: &mut Context<'_>) -> Self {
        Self { size: None }
    }

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

    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
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

    fn update(&mut self, _ctx: &mut Context<'_>, loaded: Option<Self::Loaded>, _label: &str) {
        if let Some(loaded) = loaded {
            self.size = Some(loaded.size);
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
enum ContentSizeSource {
    AsyncStr(Arc<Mutex<&'static str>>),
    SyncStr(&'static str),
    Panicking,
}

impl Source for ContentSizeSource {
    fn is_async(&self) -> bool {
        match self {
            Self::AsyncStr(_) | Self::Panicking => true,
            Self::SyncStr(_) => false,
        }
    }
}

struct ContentSizeLoaded {
    size: usize,
}
