use modor::{App, SingleMut, With};

#[derive(SingletonComponent, NoSystem)]
struct Counter(u32);

#[derive(Component)]
struct Value1(u32);

#[systems]
impl Value1 {
    #[run_after(component(Value3))]
    fn run(&mut self, mut counter: SingleMut<'_, Counter>) {
        self.0 = counter.0;
        counter.0 += 1;
    }
}

#[derive(Component)]
struct Value2(u32);

#[systems]
impl Value2 {
    #[run_after()]
    fn run(&mut self, mut counter: SingleMut<'_, Counter>) {
        self.0 = counter.0;
        counter.0 += 1;
    }
}

#[derive(Component)]
struct Value3(u32);

#[systems]
impl Value3 {
    #[run_after(component(Value2))]
    fn run(&mut self, mut counter: SingleMut<'_, Counter>) {
        self.0 = counter.0;
        counter.0 += 1;
    }
}

#[modor_test]
fn run_systems_depending_on_entities() {
    App::new()
        .with_entity(Counter(1))
        .with_entity(Value1(0))
        .with_entity(Value2(0))
        .with_entity(Value3(0))
        .updated()
        .assert::<With<Value1>>(1, |e| e.has(|v: &Value1| assert_eq!(v.0, 3)))
        .assert::<With<Value2>>(1, |e| e.has(|v: &Value2| assert_eq!(v.0, 1)))
        .assert::<With<Value3>>(1, |e| e.has(|v: &Value3| assert_eq!(v.0, 2)));
}
