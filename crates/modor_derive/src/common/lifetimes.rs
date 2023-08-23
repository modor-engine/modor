use darling::ast::GenericParamExt;
use syn::{DeriveInput, GenericParam, Generics, Lifetime};

pub(crate) fn count(input: &DeriveInput) -> usize {
    input
        .generics
        .params
        .iter()
        .filter(|p| p.as_lifetime_param().is_some())
        .count()
}

pub(crate) fn nth(generics: &Generics, position: usize) -> Option<&Lifetime> {
    generics.params.iter().nth(position).and_then(|p| {
        if let GenericParam::Lifetime(lifetime) = p {
            Some(&lifetime.lifetime)
        } else {
            None
        }
    })
}

pub(crate) fn rename_nth(
    generics: &Generics,
    position: usize,
    new_lifetime: &Lifetime,
) -> Generics {
    let mut generics = generics.clone();
    if let Some(GenericParam::Lifetime(lifetime)) = generics.params.iter_mut().nth(position) {
        lifetime.lifetime = new_lifetime.clone();
    }
    generics
}
