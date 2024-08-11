use modor::Update;

#[modor::test]
fn retrieve_value() {
    assert_eq!(Update::None.take_value(|| 5), None);
    assert_eq!(Update::Value(1).take_value(|| 5), Some(1));
    assert_eq!(Update::Value(5).take_value(|| 5), Some(5));
    assert_eq!(
        Update::Closure(Box::new(|v| *v += 1)).take_value(|| 5),
        Some(6)
    );
}

#[modor::test]
fn retrieve_value_checked() {
    assert_eq!(Update::None.take_value_checked(|| 5), None);
    assert_eq!(Update::Value(1).take_value_checked(|| 5), Some(1));
    assert_eq!(Update::Value(5).take_value_checked(|| 5), None);
    let closure = Box::new(|v: &mut u32| *v += 1);
    assert_eq!(Update::Closure(closure).take_value_checked(|| 5), Some(6));
}

#[modor::test]
fn apply_none() {
    let mut value: u32 = 0;
    Update::None.apply(&mut value);
    assert_eq!(value, 0);
}

#[modor::test]
fn apply_value() {
    let mut value: u32 = 0;
    Update::Value(42).apply(&mut value);
    assert_eq!(value, 42);
}

#[modor::test]
fn apply_closure() {
    let mut value: u32 = 0;
    Update::Closure(Box::new(|v| *v += 1)).apply(&mut value);
    assert_eq!(value, 1);
}

#[modor::test]
fn apply_checked_none() {
    let mut value: u32 = 0;
    assert!(!Update::None.apply_checked(&mut value));
    assert_eq!(value, 0);
}

#[modor::test]
fn apply_checked_same_value() {
    let mut value: u32 = 0;
    assert!(!Update::Value(0).apply_checked(&mut value));
    assert_eq!(value, 0);
}

#[modor::test]
fn apply_checked_different_value() {
    let mut value: u32 = 0;
    assert!(Update::Value(42).apply_checked(&mut value));
    assert_eq!(value, 42);
}

#[modor::test]
fn apply_checked_closure() {
    let mut value: u32 = 0;
    assert!(Update::Closure(Box::new(|v| *v += 1)).apply_checked(&mut value));
    assert_eq!(value, 1);
}
