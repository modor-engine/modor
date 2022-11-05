use modor::{App, Built, EntityBuilder, With};
use modor_jobs::{AssetLoadingError, AssetLoadingJob};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

struct FileSize {
    size: Result<Option<usize>, AssetLoadingError>,
}

#[entity]
impl FileSize {
    fn build(path: impl AsRef<str>) -> impl Built<Self> {
        EntityBuilder::new(Self { size: Ok(None) })
            .with(AssetLoadingJob::new(path, |b| async move { b.len() }))
    }

    #[run]
    fn poll(&mut self, job: &mut AssetLoadingJob<usize>) {
        self.size = job.try_poll();
    }
}

#[test]
fn load_valid_file_with_cargo() {
    // Multiple retries allowed as `load_valid_file_without_cargo` updates `CARGO_MANIFEST_DIR`.
    modor_internal::retry!(3, {
        App::new()
            .with_entity(FileSize::build("../tests/assets/test.txt"))
            .updated_until_all::<(), _>(Some(100), |s: &FileSize| {
                thread::sleep(Duration::from_millis(10));
                s.size != Ok(None)
            })
            .assert::<With<FileSize>>(1, |e| {
                e.has(|s: &FileSize| assert_eq!(s.size, Ok(Some(12))))
            })
            .updated()
            .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
    });
}

#[test]
fn load_valid_file_without_cargo() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let asset_path = PathBuf::from(cargo_manifest_dir).join("tests/assets/test.txt");
    let executable_path: PathBuf = std::env::current_exe().unwrap().parent().unwrap().into();
    fs::create_dir_all(executable_path.join("assets")).unwrap();
    fs::copy(asset_path, executable_path.join("assets/copied_test.txt")).unwrap();
    std::env::remove_var("CARGO_MANIFEST_DIR");
    App::new()
        .with_entity(FileSize::build("copied_test.txt"))
        .updated_until_all::<(), _>(Some(100), |s: &FileSize| {
            thread::sleep(Duration::from_millis(10));
            s.size != Ok(None)
        })
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert_eq!(s.size, Ok(Some(12))))
        })
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
    std::env::set_var("CARGO_MANIFEST_DIR", cargo_manifest_dir);
}

#[test]
fn load_missing_file() {
    App::new()
        .with_entity(FileSize::build("invalid.txt"))
        .updated_until_all::<(), _>(Some(100), |s: &FileSize| {
            thread::sleep(Duration::from_millis(10));
            s.size != Ok(None)
        })
        .assert::<With<FileSize>>(1, |e| {
            e.has(|s: &FileSize| assert!(matches!(s.size, Err(AssetLoadingError::IoError(_)))))
        })
        .updated()
        .assert::<With<FileSize>>(1, |e| e.has(|s: &FileSize| assert_eq!(s.size, Ok(None))));
}
