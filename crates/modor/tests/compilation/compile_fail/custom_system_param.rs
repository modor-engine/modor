#[macro_use]
extern crate modor;

fn main() {}

#[derive(Component, NoSystem)]
struct Component1;

#[derive(Component, NoSystem)]
struct Component2;

#[derive(SystemParam)]
enum EnumSystemParam {
    //~^ error: custom system param cannot be an enum
    Variant1,
    Variant2,
}

#[derive(SystemParam)]
union UnionSystemParam<'a> {
    //~^ error: custom system param cannot be a union
    c1: &'a Component1,
    c2: &'a Component2,
}

#[derive(SystemParam)]
struct SystemParamWithMoreThanOneLifetime<'a, 'b> {
    //~^ error: custom system param should have exactly one generic lifetime
    c1: &'a Component1,
    c2: &'b Component2,
}

#[derive(SystemParam)]
//~^ error: custom system param should have exactly one generic lifetime
struct SystemParamWithNoLifetime;
