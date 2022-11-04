#[macro_export]
macro_rules! retry {
    ($count:literal, $expr:expr) => {
        for i in 0..$count {
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

#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        approx::assert_abs_diff_eq!($left, $right, epsilon = 0.000_01)
    };
}
