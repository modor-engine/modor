#[macro_use]
extern crate modor;

fn main() {}

#[derive(Component, NoSystem)]
struct Component1;

#[derive(Component, NoSystem)]
struct Component2;

#[derive(QuerySystemParam)]
enum EnumSystemParam {
    //~^ error: custom system param cannot be an enum
    Variant1,
    Variant2,
}

#[derive(QuerySystemParam)]
union UnionSystemParam<'a> {
    //~^ error: custom system param cannot be a union
    c1: &'a Component1,
    c2: &'a Component2,
}

#[derive(QuerySystemParam)]
struct SystemParamWithMoreThanOneLifetime<'a, 'b> {
    //~^ error: custom system param should have exactly one generic lifetime
    c1: &'a Component1,
    c2: &'b Component2,
}

#[derive(QuerySystemParam)]
//~^ error: custom system param should have exactly one generic lifetime
struct SystemParamWithNoLifetime;

#[derive(QuerySystemParam)]
struct SystemParamWithNoLifetimeAndGeneric<T: modor::ComponentSystems>(&'static T);
//~^ error: custom system param should have exactly one generic lifetime
