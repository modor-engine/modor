use crate::Glob;

pub trait Updater: Sized {
    type Updater<'a>
    where
        Self: 'a;

    fn updater(glob: &Glob<Self>) -> Self::Updater<'_>;
}

pub fn update_field<U>(field: &mut U, new_value: Option<U>, is_updated: &mut bool)
where
    U: PartialEq,
{
    if let Some(new_value) = new_value {
        *is_updated |= field != &new_value;
        *field = new_value;
    }
}
