use modor::log::Level;
use modor::{App, Context, Glob, GlobRef, Node, RootNode, Visit};
use modor_jobs::AssetLoadingError;
use modor_resources::{Res, Resource, ResourceError, Source};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[modor::test]
fn access_inner() {
    let mut app = App::new::<Root>(Level::Info);
    let mut res = Res::<ContentSize>::from_path(&mut app.ctx(), "not_empty.txt");
    res.should_be_reloaded = true;
    assert!(res.should_be_reloaded);
}

#[modor::test(disabled(wasm))]
fn load_valid_resource_from_path() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_path(&mut app, "not_empty.txt");
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert_eq!(res(&mut app).err(), None);
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(12));
    assert_eq!(res(&mut app).err(), None);
    app.update();
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(12));
    assert_eq!(res(&mut app).err(), None);
}

#[modor::test(disabled(wasm))]
fn load_invalid_resource_from_path() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_path(&mut app, "empty.txt");
    let error = ResourceError::Other("empty resource".into());
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert_eq!(res(&mut app).err(), Some(&error));
    app.update();
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert_eq!(res(&mut app).err(), Some(&error));
}

#[modor::test(disabled(wasm))]
fn load_resource_from_invalid_path() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_path(&mut app, "missing.txt");
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert!(matches!(
        res(&mut app).err(),
        Some(ResourceError::Loading(AssetLoadingError::IoError(_)))
    ));
    app.update();
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert!(matches!(
        res(&mut app).err(),
        Some(ResourceError::Loading(AssetLoadingError::IoError(_)))
    ));
}

#[modor::test(cases(
    async_ = "ContentSizeSource::AsyncStr(Arc::new(Mutex::new(\"content\")))",
    sync = "ContentSizeSource::SyncStr(\"content\")"
))]
fn load_valid_resource_from_source(source: ContentSizeSource) {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_source(&mut app, source);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert_eq!(res(&mut app).err(), None);
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(7));
    assert_eq!(res(&mut app).err(), None);
    app.update();
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(7));
    assert_eq!(res(&mut app).err(), None);
}

#[modor::test(cases(
    async_ = "ContentSizeSource::AsyncStr(Arc::new(Mutex::new(\"\")))",
    sync = "ContentSizeSource::SyncStr(\"\")"
))]
fn load_invalid_resource_from_source(source: ContentSizeSource) {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_source(&mut app, source);
    let error = ResourceError::Other("empty resource".into());
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert_eq!(res(&mut app).err(), Some(&error));
    app.update();
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), None);
    assert_eq!(res(&mut app).err(), Some(&error));
}

#[modor::test]
fn reload_with_source() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_source(&mut app, ContentSizeSource::SyncStr("content"));
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(7));
    assert_eq!(res(&mut app).err(), None);
    res(&mut app).reload_with_source(ContentSizeSource::SyncStr("other content"));
    app.update();
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(13));
    assert_eq!(res(&mut app).err(), None);
}

#[modor::test]
fn reload_with_path() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_source(&mut app, ContentSizeSource::SyncStr("content"));
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(7));
    assert_eq!(res(&mut app).err(), None);
    res(&mut app).reload_with_path("not_empty.txt");
    wait_resource_loaded(&mut app);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(7));
    assert_eq!(res(&mut app).err(), None);
    app.update();
    wait_resource_reloaded(&mut app, 12);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(12));
    assert_eq!(res(&mut app).err(), None);
}

#[modor::test]
fn reload_from_inside() {
    let content = Arc::new(Mutex::new("content"));
    let mut app = App::new::<Root>(Level::Info);
    let glob = create_resource_from_source(&mut app, ContentSizeSource::AsyncStr(content.clone()));
    wait_resource_loaded(&mut app);
    *content.lock().unwrap() = "other content";
    res(&mut app).should_be_reloaded = true;
    app.update();
    res(&mut app).should_be_reloaded = false;
    wait_resource_reloaded(&mut app, 13);
    assert_eq!(glob.get(&app.ctx()).as_ref().map(|g| g.size), Some(13));
    assert_eq!(res(&mut app).err(), None);
}

fn create_resource_from_path(app: &mut App, path: &str) -> GlobRef<Option<ContentSizeGlob>> {
    app.root::<Root>().content_size = Some(Res::from_path(&mut app.ctx(), path));
    res(app).glob().clone()
}

fn create_resource_from_source(
    app: &mut App,
    source: ContentSizeSource,
) -> GlobRef<Option<ContentSizeGlob>> {
    app.root::<Root>().content_size = Some(Res::from_source(&mut app.ctx(), source));
    res(app).glob().clone()
}

fn wait_resource_loaded(app: &mut App) {
    const MAX_RETRIES: u32 = 100;
    for _ in 0..MAX_RETRIES {
        app.update();
        thread::sleep(Duration::from_millis(10));
        if res(app).err().is_some() || res(app).glob().clone().get(&app.ctx()).is_some() {
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
        let size = res(app)
            .glob()
            .clone()
            .get(&app.ctx())
            .as_ref()
            .unwrap()
            .size;
        if target_size == size {
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
    should_be_reloaded: bool,
}

impl Resource for ContentSize {
    type Source = ContentSizeSource;
    type Loaded = ContentSizeLoaded;
    type Glob = ContentSizeGlob;

    fn should_be_reloaded(&self, _glob: &Glob<Option<Self::Glob>>, _ctx: &mut Context<'_>) -> bool {
        self.should_be_reloaded
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
        };
        if size == 0 {
            Err(ResourceError::Other("empty resource".into()))
        } else {
            Ok(ContentSizeLoaded { size })
        }
    }

    fn update(
        &mut self,
        glob: &Glob<Option<Self::Glob>>,
        ctx: &mut Context<'_>,
        loaded: Option<Self::Loaded>,
    ) {
        if let Some(loaded) = loaded {
            *glob.get_mut(ctx) = Some(ContentSizeGlob { size: loaded.size });
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
enum ContentSizeSource {
    AsyncStr(Arc<Mutex<&'static str>>),
    SyncStr(&'static str),
}

impl Source for ContentSizeSource {
    fn is_async(&self) -> bool {
        match self {
            Self::AsyncStr(_) => true,
            Self::SyncStr(_) => false,
        }
    }
}

struct ContentSizeLoaded {
    size: usize,
}

struct ContentSizeGlob {
    size: usize,
}
