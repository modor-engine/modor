#[macro_export]
macro_rules! retry {
    ($count:literal, $expr:expr) => {
        for i in 0..$count {
            println!("Try #{}...", i);
            let r = std::panic::catch_unwind(|| $expr);
            if r.is_ok() {
                break;
            }
            if i == $count - 1 {
                std::panic::resume_unwind(r.unwrap_err());
            } else {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    };
}
