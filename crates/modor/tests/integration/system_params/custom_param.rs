use crate::system_params::{OtherValue, Value};
use modor::{
    App, Custom, CustomSystemParam, Query, QuerySystemParam, QuerySystemParamWithLifetime,
    SystemParamWithLifetime,
};

// TODO: refactor custom system param tests

struct MySystemParam<'a> {
    value: &'a mut Value,
    query: Query<'a, &'static OtherValue>,
}

impl<'a> CustomSystemParam for MySystemParam<'a> {
    type ConstParam<'b> = MySystemParam<'b>;
    type Param<'b> = MySystemParam<'b>;
    type Tuple = (&'a mut Value, Query<'a, &'static OtherValue>);

    fn from_tuple_const_param_mut_param<'b>(
        _tuple: <<Self::Tuple as QuerySystemParamWithLifetime<'b>>::ConstParam as SystemParamWithLifetime<'b>>::Param,
    ) -> <Custom<Self::ConstParam<'b>> as SystemParamWithLifetime<'b>>::Param
    where
        Self::Tuple: QuerySystemParam,
    {
        unreachable!()
    }

    fn from_tuple_const_param(
        _tuple: <Self::Tuple as QuerySystemParamWithLifetime<'_>>::ConstParam,
    ) -> Custom<Self::ConstParam<'_>>
    where
        Self::Tuple: QuerySystemParam,
    {
        unreachable!()
    }

    fn from_tuple_mut_param(
        tuple: <Self::Tuple as SystemParamWithLifetime<'_>>::Param,
    ) -> Custom<Self::Param<'_>> {
        Custom::new(MySystemParam {
            value: tuple.0,
            query: tuple.1,
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
