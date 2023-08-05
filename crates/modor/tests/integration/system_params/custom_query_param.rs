use crate::system_params::{OtherValue, Value};
use modor::{
    App, Custom, CustomSystemParam, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};

// TODO: add derive macros SystemParam and QuerySystemParam
// TODO: add doc
// TODO: refactor custom system param tests
// TODO: add compilation tests
// TODO: use Custom type when useful

struct MySystemParam<'a> {
    value: &'a mut Value,
    other_value: &'a OtherValue,
}

impl<'a> CustomSystemParam for MySystemParam<'a> {
    type ConstParam<'b> = ConstMySystemParam<'b>;
    type Param<'b> = MySystemParam<'b>;
    type Tuple = (&'a mut Value, &'a OtherValue);

    fn from_tuple_const_param_mut_param<'b>(
        tuple: <<Self::Tuple as QuerySystemParamWithLifetime<'b>>::ConstParam as SystemParamWithLifetime<'b>>::Param,
    ) -> <Custom<Self::ConstParam<'b>> as SystemParamWithLifetime<'b>>::Param {
        Custom::new(ConstMySystemParam {
            value: tuple.0,
            other_value: tuple.1,
        })
    }

    fn from_tuple_const_param(
        tuple: <Self::Tuple as QuerySystemParamWithLifetime<'_>>::ConstParam,
    ) -> Custom<Self::ConstParam<'_>> {
        Custom::new(ConstMySystemParam {
            value: tuple.0,
            other_value: tuple.1,
        })
    }

    fn from_tuple_mut_param(
        tuple: <Self::Tuple as SystemParamWithLifetime<'_>>::Param,
    ) -> Custom<Self::Param<'_>> {
        Custom::new(MySystemParam {
            value: tuple.0,
            other_value: tuple.1,
        })
    }
}

struct ConstMySystemParam<'a> {
    value: <&'a mut Value as QuerySystemParamWithLifetime<'a>>::ConstParam,
    other_value: <&'a OtherValue as QuerySystemParamWithLifetime<'a>>::ConstParam,
}

impl<'a> CustomSystemParam for ConstMySystemParam<'a> {
    type ConstParam<'b> = ConstMySystemParam<'b>;
    type Param<'b> = ConstMySystemParam<'b>;
    type Tuple = (
        <&'a mut Value as QuerySystemParamWithLifetime<'a>>::ConstParam,
        <&'a OtherValue as QuerySystemParamWithLifetime<'a>>::ConstParam,
    );

    fn from_tuple_const_param_mut_param<'b>(
        tuple: <<Self::Tuple as QuerySystemParamWithLifetime<'b>>::ConstParam as SystemParamWithLifetime<'b>>::Param,
    ) -> <Custom<Self::ConstParam<'b>> as SystemParamWithLifetime<'b>>::Param {
        Custom::new(ConstMySystemParam {
            value: tuple.0,
            other_value: tuple.1,
        })
    }

    fn from_tuple_const_param(
        tuple: <Self::Tuple as QuerySystemParamWithLifetime<'_>>::ConstParam,
    ) -> Custom<Self::ConstParam<'_>> {
        Custom::new(ConstMySystemParam {
            value: tuple.0,
            other_value: tuple.1,
        })
    }

    fn from_tuple_mut_param(
        tuple: <Self::Tuple as SystemParamWithLifetime<'_>>::Param,
    ) -> Custom<Self::Param<'_>> {
        Custom::new(ConstMySystemParam {
            value: tuple.0,
            other_value: tuple.1,
        })
    }
}

#[derive(Component)]
struct Tester;

#[systems]
impl Tester {
    #[run]
    fn update(mut param: Custom<MySystemParam<'_>>) {
        param.value.0 += 1;
    }
}

#[test]
fn test() {
    App::new().with_entity(Tester).updated();
}
