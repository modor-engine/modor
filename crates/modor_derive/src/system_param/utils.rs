use syn::{GenericParam, Generics, Lifetime};

pub(super) fn replace_first_lifetime(generics: &Generics, new_lifetime: &Lifetime) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Lifetime(lifetime) = param {
            lifetime.lifetime = new_lifetime.clone();
            break;
        }
    }
    generics
}
