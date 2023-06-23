use crate::system_params::assert_iter;
use modor::{App, BuiltEntity, Entity, EntityBuilder, Filter, Query, With};

#[derive(SingletonComponent, Default)]
struct Tester {
    done_count: u32,
}

#[systems]
impl Tester {
    #[run]
    fn iter_with_no_filter(&mut self, query: Query<'_, Entity<'_>>) {
        assert_iter(query.iter().map(Entity::id), [0, 1, 4, 2, 5, 3, 6]);
        self.done_count += 1;
    }

    #[run]
    fn iter_with_one_filter(&mut self, query: Query<'_, (Entity<'_>, Filter<With<Value1>>)>) {
        assert_iter(query.iter().map(|v| v.0.id()), [1, 4, 3, 6]);
        self.done_count += 1;
    }

    #[allow(clippy::type_complexity)]
    #[run]
    fn iter_with_multiple_filters(
        &mut self,
        query: Query<'_, (Entity<'_>, Filter<(With<Value1>, With<Value2>)>)>,
    ) {
        assert_iter(query.iter().map(|v| v.0.id()), [3, 6]);
        self.done_count += 1;
    }

    #[run]
    fn get_both_mut(&mut self, mut query: Query<'_, (&Value1, Filter<With<Value2>>)>) {
        let (left, right) = query.get_both_mut(3, 6);
        assert_eq!(left.map(|v| v.0 .0), Some(10));
        assert_eq!(right.map(|v| v.0 .0), Some(30));
        let (left, right) = query.get_both_mut(3, 4);
        assert_eq!(left.map(|v| v.0 .0), Some(10));
        assert_eq!(right.map(|v| v.0 .0), None);
        let (left, right) = query.get_both_mut(1, 6);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), Some(30));
        let (left, right) = query.get_both_mut(1, 4);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), None);
        let (left, right) = query.get_both_mut(3, 3);
        assert_eq!(left.map(|v| v.0 .0), Some(10));
        assert_eq!(right.map(|v| v.0 .0), None);
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
        query2: Query<'_, (&Value2, Filter<With<Level2>>)>,
    ) {
        let (left, right) = query1.get_with_first_parent(3);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), Some(12));
        let (left, right) = query1.get_with_first_parent(1);
        assert_eq!(left.map(|v| v.0), Some(12));
        assert_eq!(right.map(|v| v.0), None);
        let (left, right) = query2.get_with_first_parent(3);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), Some(21));
        let (left, right) = query2.get_with_first_parent(1);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), None);
        self.done_count += 1;
    }

    #[run]
    fn get_with_first_parent_mut(
        &mut self,
        mut query1: Query<'_, &Value1>,
        mut query2: Query<'_, (&Value2, Filter<With<Level2>>)>,
    ) {
        let (left, right) = query1.get_with_first_parent_mut(3);
        assert_eq!(left.map(|v| v.0), Some(10));
        assert_eq!(right.map(|v| v.0), Some(12));
        let (left, right) = query1.get_with_first_parent_mut(1);
        assert_eq!(left.map(|v| v.0), Some(12));
        assert_eq!(right.map(|v| v.0), None);
        let (left, right) = query2.get_with_first_parent_mut(3);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), Some(21));
        let (left, right) = query2.get_with_first_parent_mut(1);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), None);
        self.done_count += 1;
    }
}

#[derive(Component, NoSystem)]
struct Value1(u32);

#[derive(Component, NoSystem)]
struct Value2(u32);

#[derive(Component, NoSystem)]
struct Value3(u32);

#[derive(Component, NoSystem)]
struct Level1;

impl Level1 {
    fn build(value1: u32, value2: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Value1(value1 + 2))
            .with(Value3(value1 + 2))
            .with_child(Level2::build(value1, value2))
    }
}

#[derive(Component, NoSystem)]
struct Level2;

impl Level2 {
    fn build(value1: u32, value2: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Value2(value2 + 1))
            .with_child(Level3::build(value1, value2))
    }
}

#[derive(Component, NoSystem)]
struct Level3;

impl Level3 {
    fn build(value1: u32, value2: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Value1(value1))
            .with(Value2(value2))
    }
}

#[modor_test]
fn use_query() {
    App::new()
        .with_entity(Tester::default())
        .with_entity(Level1::build(10, 20))
        .with_entity(Level1::build(30, 40))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|t: &Tester| assert_eq!(t.done_count, 7)));
}
