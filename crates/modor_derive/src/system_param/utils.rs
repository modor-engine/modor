use syn::{GenericParam, Generics, Lifetime};

pub(super) fn replace_first_lifetime(generics: &Generics, new_lifetime: &Lifetime) -> Generics {
    let mut generics = generics.clone();
    if let Some(GenericParam::Lifetime(lifetime)) = generics.params.iter_mut().next() {
        lifetime.lifetime = new_lifetime.clone();
    }
    generics
}
