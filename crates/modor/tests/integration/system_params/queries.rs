use crate::system_params::assert_iter;
use modor::testing::TestApp;
use modor::{App, Built, Entity, EntityBuilder, Query, With};

struct Tester {
    done_count: u32,
}

#[singleton]
impl Tester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { done_count: 0 })
    }

    #[run]
    fn iter_with_no_filter(&mut self, query: Query<'_, Entity<'_>>) {
        assert_iter(query.iter().map(Entity::id), [0, 1, 4, 2, 5, 3, 6]);
        self.done_count += 1;
    }

    #[run]
    fn iter_with_one_filter(&mut self, query: Query<'_, Entity<'_>, With<Value1>>) {
        assert_iter(query.iter().map(Entity::id), [1, 4, 3, 6]);
        self.done_count += 1;
    }

    #[run]
    fn iter_with_multiple_filters(
        &mut self,
        query: Query<'_, Entity<'_>, (With<Value1>, With<Value2>)>,
    ) {
        assert_iter(query.iter().map(Entity::id), [3, 6]);
        self.done_count += 1;
    }

    #[run]
    fn get_both_mut(&mut self, mut query: Query<'_, &Value1, With<Value2>>) {
        let (left, right) = query.get_both_mut(3, 6);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), Some(30));
        let (left, right) = query.get_both_mut(3, 4);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), None);
        let (left, right) = query.get_both_mut(1, 6);
        assert_eq!(left.map(|v| v.0), None);
        assert_eq!(right.map(|v| v.0), Some(30));
        let (left, right) = query.get_both_mut(1, 4);
        assert_eq!(left.map(|v| v.0), None);
        assert_eq!(right.map(|v| v.0), None);
        let (left, right) = query.get_both_mut(3, 3);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), None);
        self.done_count += 1;
    }

    #[run]
    fn get_both_mut_with_other_query(&mut self, mut query: Query<'_, Option<&mut Value3>>) {
        let (left, right) = query.get_both_mut(3, 6);
        assert_eq!(left.flatten().map(|v| v.0), None);
        assert_eq!(right.flatten().map(|v| v.0), None);
        let (left, right) = query.get_both_mut(4, 3);
        assert_eq!(left.flatten().map(|v| v.0), Some(32));
        assert_eq!(right.flatten().map(|v| v.0), None);
        let (left, right) = query.get_both_mut(3, 4);
        assert_eq!(left.flatten().map(|v| v.0), None);
        assert_eq!(right.flatten().map(|v| v.0), Some(32));
        self.done_count += 1;
    }

    #[run]
    fn get_with_first_parent(
        &mut self,
        query1: Query<'_, &Value1>,
        query2: Query<'_, &Value2, With<Level2>>,
    ) {
        let (left, right) = query1.get_with_first_parent(3);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), Some(12));
        let (left, right) = query1.get_with_first_parent(1);
        assert_eq!(left.map(|v| v.0), Some(12));
        assert_eq!(right.map(|v| v.0), None);
        let (left, right) = query2.get_with_first_parent(3);
        assert_eq!(left.map(|v| v.0), None);
        assert_eq!(right.map(|v| v.0), Some(21));
        let (left, right) = query2.get_with_first_parent(1);
        assert_eq!(left.map(|v| v.0), None);
        assert_eq!(right.map(|v| v.0), None);
        self.done_count += 1;
    }

    #[run]
    fn get_with_first_parent_mut(
        &mut self,
        mut query1: Query<'_, &Value1>,
        mut query2: Query<'_, &Value2, With<Level2>>,
    ) {
        let (left, right) = query1.get_with_first_parent_mut(3);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), Some(12));
        let (left, right) = query1.get_with_first_parent_mut(1);
        assert_eq!(left.map(|v| v.0), Some(12));
        assert_eq!(right.map(|v| v.0), None);
        let (left, right) = query2.get_with_first_parent_mut(3);
        assert_eq!(left.map(|v| v.0), None);
        assert_eq!(right.map(|v| v.0), Some(21));
        let (left, right) = query2.get_with_first_parent_mut(1);
        assert_eq!(left.map(|v| v.0), None);
        assert_eq!(right.map(|v| v.0), None);
        self.done_count += 1;
    }
}

struct Value1(u32);

struct Value2(u32);

struct Value3(u32);

struct Level1;

#[entity]
impl Level1 {
    fn build(value1: u32, value2: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value1(value1 + 2))
            .with(Value3(value1 + 2))
            .with_child(Level2::build(value1, value2))
    }
}

struct Level2;

#[entity]
impl Level2 {
    fn build(value1: u32, value2: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value2(value2 + 1))
            .with_child(Level3::build(value1, value2))
    }
}

struct Level3;

#[entity]
impl Level3 {
    fn build(value1: u32, value2: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value1(value1))
            .with(Value2(value2))
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_query() {
    let mut app: TestApp = App::new()
        .with_entity(Tester::build())
        .with_entity(Level1::build(10, 20))
        .with_entity(Level1::build(30, 40))
        .into();
    app.update();
    app.assert_singleton::<Tester>()
        .has(|t: &Tester| assert_eq!(t.done_count, 7));
}
