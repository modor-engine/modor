use modor::{App, Built, EntityBuilder, SingleMut, With};

struct Counter(u32);

#[singleton]
impl Counter {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(1))
    }
}

struct Value1(u32);

#[entity]
impl Value1 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0))
    }

    #[run_after(Value3)]
    fn run(&mut self, mut counter: SingleMut<'_, Counter>) {
        self.0 = counter.0;
        counter.0 += 1;
    }
}

struct Value2(u32);

#[entity]
impl Value2 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0))
    }

    #[run]
    fn run(&mut self, mut counter: SingleMut<'_, Counter>) {
        self.0 = counter.0;
        counter.0 += 1;
    }
}

struct Value3(u32);

#[entity]
impl Value3 {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0))
    }

    #[run_after(Value2)]
    fn run(&mut self, mut counter: SingleMut<'_, Counter>) {
        self.0 = counter.0;
        counter.0 += 1;
    }
}

#[test]
fn run_systems_depending_on_entities() {
    App::new()
        .with_entity(Counter::build())
        .with_entity(Value1::build())
        .with_entity(Value2::build())
        .with_entity(Value3::build())
        .updated()
        .assert::<With<Value1>>(1, |e| e.has(|v: &Value1| assert_eq!(v.0, 3)))
        .assert::<With<Value2>>(1, |e| e.has(|v: &Value2| assert_eq!(v.0, 1)))
        .assert::<With<Value3>>(1, |e| e.has(|v: &Value3| assert_eq!(v.0, 2)));
}
