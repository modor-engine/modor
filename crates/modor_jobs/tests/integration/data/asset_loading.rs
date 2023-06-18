use modor::{App, With};
use modor_jobs::{AssetLoadingError, AssetLoadingJob};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

#[derive(Component)]
struct FileLoader {
    job: AssetLoadingJob<usize>,
    size: Result<Option<usize>, AssetLoadingError>,
}

#[systems]
impl FileLoader {
    fn new(path: impl AsRef<str>) -> Self {
        Self {
            job: AssetLoadingJob::new(path, |b| async move {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    async_std::task::sleep(Duration::from_millis(10)).await;
                    b.len()
                }
                #[cfg(target_arch = "wasm32")]
                {
                    0
                }
            }),
            size: Ok(None),
        }
    }

    #[run]
    fn poll(&mut self) {
        self.size = self.job.try_poll();
    }
}

#[modor_test(disabled(wasm))]
fn load_valid_file_with_cargo() {
    // Multiple retries allowed as `load_valid_file_without_cargo` updates `CARGO_MANIFEST_DIR`.
    modor_internal::retry!(3, {
        App::new()
            .with_entity(FileLoader::new("test.txt"))
            .updated_until_all::<(), _>(Some(100), |l: &FileLoader| {
                thread::sleep(Duration::from_millis(10));
                l.size != Ok(None)
            })
            .assert::<With<FileLoader>>(1, |e| {
                e.has(|l: &FileLoader| assert_eq!(l.size, Ok(Some(12))))
            })
            .updated()
            .assert::<With<FileLoader>>(1, |e| {
                e.has(|l: &FileLoader| assert_eq!(l.size, Ok(None)))
            });
    });
}

#[modor_test(disabled(wasm))]
fn load_valid_file_without_cargo() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let asset_path = PathBuf::from(cargo_manifest_dir).join("assets/test.txt");
    let executable_path: PathBuf = std::env::current_exe().unwrap().parent().unwrap().into();
    fs::create_dir_all(executable_path.join("assets")).unwrap();
    fs::copy(asset_path, executable_path.join("assets/copied_test.txt")).unwrap();
    std::env::remove_var("CARGO_MANIFEST_DIR");
    App::new()
        .with_entity(FileLoader::new("copied_test.txt"))
        .updated_until_all::<(), _>(Some(100), |l: &FileLoader| {
            thread::sleep(Duration::from_millis(10));
            l.size != Ok(None)
        })
        .assert::<With<FileLoader>>(1, |e| {
            e.has(|l: &FileLoader| assert_eq!(l.size, Ok(Some(12))))
        })
        .updated()
        .assert::<With<FileLoader>>(1, |e| e.has(|l: &FileLoader| assert_eq!(l.size, Ok(None))));
    std::env::set_var("CARGO_MANIFEST_DIR", cargo_manifest_dir);
}

#[modor_test(disabled(wasm))]
fn load_missing_file() {
    App::new()
        .with_entity(FileLoader::new("invalid.txt"))
        .updated_until_all::<(), _>(Some(100), |l: &FileLoader| {
            thread::sleep(Duration::from_millis(10));
            l.size != Ok(None)
        })
        .assert::<With<FileLoader>>(1, |e| {
            e.has(|l: &FileLoader| assert!(matches!(l.size, Err(AssetLoadingError::IoError(_)))))
        })
        .updated()
        .assert::<With<FileLoader>>(1, |e| e.has(|l: &FileLoader| assert_eq!(l.size, Ok(None))));
}
