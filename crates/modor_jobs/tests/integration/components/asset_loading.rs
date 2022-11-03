use modor::{App, Built, EntityBuilder, With};
use modor_jobs::{AssetLoadingError, AssetLoadingJob};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

struct FileSize {
    size: Result<Option<usize>, AssetLoadingError>,
    should_poll: bool,
}

#[entity]
impl FileSize {
    fn build(path: impl AsRef<str>) -> impl Built<Self> {
        EntityBuilder::new(Self {
            size: Ok(None),
            should_poll: true,
        })
        .with(AssetLoadingJob::new(path, |b| async move { b.len() }))
    }

    #[run]
    fn poll(&mut self, job: &mut AssetLoadingJob<usize>) {
        if !self.should_poll {
            return;
        }
        self.size = job.try_poll();
        self.should_poll = false;
    }
}

#[test]
fn load_valid_texture_with_cargo() {
    // Multiple retries allowed as `load_valid_texture_without_cargo` updates `CARGO_MANIFEST_DIR`.
    modor_internal::retry!(3, {
        App::new()
            .with_entity(FileSize::build("../tests/assets/test.txt"))
            .updated()
            .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))))
            .with_update::<(), _>(|_: &mut FileSize| thread::sleep(Duration::from_millis(100)))
            .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
            .updated()
            .assert::<With<FileSize>>(1, |e| {
                e.has(|s: &FileSize| assert_eq!(s.size, Ok(Some(12))))
            })
            .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
            .updated()
            .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
    });
}

#[test]
fn load_valid_texture_without_cargo() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let asset_path = PathBuf::from(cargo_manifest_dir).join("tests/assets/test.txt");
    let executable_path: PathBuf = std::env::current_exe().unwrap().parent().unwrap().into();
    fs::create_dir_all(executable_path.join("assets")).unwrap();
    fs::copy(asset_path, executable_path.join("assets/copied_test.txt")).unwrap();
    std::env::remove_var("CARGO_MANIFEST_DIR");
    App::new()
        .with_entity(FileSize::build("copied_test.txt"))
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))))
        .with_update::<(), _>(|_: &mut FileSize| thread::sleep(Duration::from_millis(100)))
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert_eq!(s.size, Ok(Some(12))))
        })
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
    std::env::set_var("CARGO_MANIFEST_DIR", cargo_manifest_dir);
}

#[test]
fn load_missing_texture() {
    App::new()
        .with_entity(FileSize::build("invalid.txt"))
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))))
        .with_update::<(), _>(|_: &mut FileSize| thread::sleep(Duration::from_millis(100)))
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert!(matches!(s.size, Err(AssetLoadingError::IoError(_)))))
        })
        .with_update::<(), _>(|s: &mut FileSize| s.should_poll = true)
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
}
