use modor::UsizeRange;

#[test]
fn check_whether_value_is_contained() {
    assert!(1.contains_value(1));
    assert!(!1.contains_value(2));
    assert!((..).contains_value(1));
    assert!(!(1..2).contains_value(0));
    assert!((1..2).contains_value(1));
    assert!(!(1..2).contains_value(2));
    assert!(!(1..=2).contains_value(0));
    assert!((1..=2).contains_value(1));
    assert!((1..=2).contains_value(2));
    assert!(!(1..=2).contains_value(3));
    assert!(!(1..).contains_value(0));
    assert!((1..).contains_value(1));
    assert!((1..).contains_value(42));
    assert!((..1).contains_value(0));
    assert!(!(..1).contains_value(1));
    assert!((..=1).contains_value(0));
    assert!((..=1).contains_value(1));
    assert!(!(..=1).contains_value(2));
}
