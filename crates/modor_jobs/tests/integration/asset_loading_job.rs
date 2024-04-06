use modor_jobs::{AssetLoadingError, AssetLoadingJob};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
static CARGO_MANIFEST_DIR_LOCK: Mutex<()> = Mutex::new(());

#[modor::test(disabled(wasm))]
fn load_valid_file_with_cargo() {
    let _lock = CARGO_MANIFEST_DIR_LOCK.lock();
    let mut job = AssetLoadingJob::new("test.txt", file_size);
    let result = retrieve_result(&mut job);
    assert_eq!(result, Ok(Some(12)));
    assert_eq!(job.try_poll(), Ok(None));
}

#[modor::test(disabled(wasm))]
fn load_valid_file_without_cargo() {
    let _lock = CARGO_MANIFEST_DIR_LOCK.lock();
    let asset_path = PathBuf::from(CARGO_MANIFEST_DIR).join("assets/test.txt");
    std::env::remove_var("CARGO_MANIFEST_DIR");
    let mut job = AssetLoadingJob::new(asset_path.to_str().unwrap(), file_size);
    let result = retrieve_result(&mut job);
    assert_eq!(result, Ok(Some(12)));
    assert_eq!(job.try_poll(), Ok(None));
    std::env::set_var("CARGO_MANIFEST_DIR", CARGO_MANIFEST_DIR);
}

#[modor::test(disabled(wasm))]
fn load_missing_file() {
    let mut job = AssetLoadingJob::new("invalid.txt", file_size);
    let result = retrieve_result(&mut job);
    assert!(matches!(result, Err(AssetLoadingError::IoError(_))));
    assert_eq!(job.try_poll(), Ok(None));
}

#[allow(clippy::unused_async)]
async fn file_size(bytes: Vec<u8>) -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    async_std::task::sleep(Duration::from_millis(10)).await;
    bytes.len()
}

fn retrieve_result(job: &mut AssetLoadingJob<usize>) -> Result<Option<usize>, AssetLoadingError> {
    const MAX_RETIES: u32 = 100;
    for _ in 0..MAX_RETIES {
        thread::sleep(Duration::from_millis(10));
        let result = job.try_poll();
        if result != Ok(None) {
            return result;
        }
    }
    panic!("max retries reached");
}
