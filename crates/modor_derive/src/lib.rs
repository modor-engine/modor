//! Procedural macros of Modor.

use darling::{ast, FromDeriveInput, FromField, FromGenerics};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span, TokenTree};
use proc_macro_error::{abort, OptionExt, ResultExt};
use quote::quote;
use std::collections::HashMap;
use syn::__private::TokenStream2;
use syn::spanned::Spanned;
use syn::token::Ge;
use syn::{
    parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, GenericParam, Generics,
    ImplGenerics, ItemFn, ItemImpl, Meta, NestedMeta, Type,
};

mod attributes;
mod idents;
mod impl_block;
mod systems;

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn modor_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let conditions: HashMap<_, _> = platform_conditions();
    let item = parse_macro_input!(item as ItemFn);
    let attr = parse_macro_input!(attr as AttributeArgs);
    if attr.len() > 1 {
        abort!(attr[1], "max one argument is allowed");
    }
    let excluded_platforms = attr
        .first()
        .map_or_else(|| Some(vec![]), parse_test_meta)
        .expect_or_abort(
            "expected syntax: `#[modor_test]` or `#[modor_test(disabled(platform1, ...))]`",
        );
    let mut platforms: Vec<_> = conditions.keys().collect();
    platforms.sort_unstable();
    for excluded_platform in &excluded_platforms {
        if !conditions.contains_key(excluded_platform.to_string().as_str()) {
            abort!(excluded_platform, "allowed platforms are {:?}", platforms);
        }
    }
    let conditions = excluded_platforms
        .iter()
        .map(|p| &conditions[p.to_string().as_str()])
        .collect::<Vec<_>>();
    let output = quote! {
        #[cfg_attr(any(#(#conditions),*), allow(unused))]
        #[cfg_attr(not(any(#(#conditions),*)), test)]
        #[cfg_attr(
            all(target_arch = "wasm32", not(any(#(#conditions),*))),
            ::wasm_bindgen_test::wasm_bindgen_test)
        ]
        #item
    };
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Action)]
#[proc_macro_error::proc_macro_error]
pub fn action_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ident = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let dependencies: Vec<_> = match &item.data {
        Data::Struct(struct_) => match &struct_.fields {
            Fields::Unnamed(fields) => fields.unnamed.iter().map(|f| &f.ty).collect(),
            Fields::Unit => vec![],
            Fields::Named(_) => abort!(item, "structs with named fields cannot be actions"),
        },
        Data::Enum(_) | Data::Union(_) => abort!(item, "only structs can be actions"),
    };
    let output = quote! {
        impl #impl_generics #crate_ident::Action for #ident #type_generics #where_clause {
            fn dependency_types() -> ::std::vec::Vec<::std::any::TypeId> {
                let mut types = vec![#(::std::any::TypeId::of::<#dependencies>()),*];
                #(types.extend(<#dependencies as #crate_ident::Action>::dependency_types());)*
                types
            }
        }
    };
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Component)]
#[proc_macro_error::proc_macro_error]
pub fn component_derive(item: TokenStream) -> TokenStream {
    derive_component(item, false)
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SingletonComponent)]
#[proc_macro_error::proc_macro_error]
pub fn singleton_component_derive(item: TokenStream) -> TokenStream {
    derive_component(item, true)
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(NoSystem)]
#[proc_macro_error::proc_macro_error]
pub fn no_system_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ident = &item.ident;
    let generics = item.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let finish_system_call = finish_system_call(ident);
    let output = quote! {
        impl #impl_generics #crate_ident::ComponentSystems for #ident #type_generics #where_clause {
            type Action = std::marker::PhantomData<()>;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                runner
                #finish_system_call
            }
        }
    };
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn systems(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let crate_ident = idents::find_crate_ident(item.span());
    let cleaned_block = impl_block::clean(&item);
    let type_ = &item.self_ty;
    let type_ident = idents::extract_type_ident(type_);
    let generics = &item.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let action_type_ident = Ident::new(&(type_ident.to_string() + "Action"), item.span());
    let actions = systems::action_dependencies(&item);
    let generic_type_names: Vec<&Ident> = generics.type_params().map(|g| &g.ident).collect();
    let action_phantom = generics
        .lt_token
        .is_some()
        .then(|| quote!(std::marker::PhantomData <(#(#generic_type_names,)*)>));
    let update_statement = systems::generate_update_statement(&item);
    let finish_system_call = finish_system_call(&type_ident);
    let output = quote! {
        #cleaned_block

        impl #impl_generics #crate_ident::ComponentSystems for #type_ #where_clause {
            type Action = #action_type_ident #type_generics;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                #update_statement
                #finish_system_call
            }
        }

        #[doc(hidden)]
        #[non_exhaustive]
        #[derive(#crate_ident::Action)]
        pub struct #action_type_ident #impl_generics(#(#actions,)* #action_phantom) #where_clause;
    };
    output.into()
}

fn platform_conditions() -> HashMap<&'static str, TokenStream2> {
    [
        ("windows", quote!(target_os = "windows")),
        ("macos", quote!(target_os = "macos")),
        ("android", quote!(target_os = "android")),
        ("wasm", quote!(target_arch = "wasm32")),
        (
            "linux",
            quote!(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )),
        ),
    ]
    .into_iter()
    .collect()
}

fn parse_test_meta(meta: &NestedMeta) -> Option<Vec<Ident>> {
    if let NestedMeta::Meta(Meta::List(platforms)) = meta {
        if platforms.path.segments.len() != 1 || platforms.path.segments[0].ident != "disabled" {
            None
        } else {
            platforms
                .nested
                .iter()
                .map(|n| {
                    if let NestedMeta::Meta(Meta::Path(path)) = n {
                        Some(path.segments[0].ident.clone())
                    } else {
                        None
                    }
                })
                .collect::<Option<Vec<_>>>()
        }
    } else {
        None
    }
}

fn derive_component(item: TokenStream, is_singleton: bool) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ident = &item.ident;
    let generics = item.generics;
    let is_singleton_type = if is_singleton {
        quote! { #crate_ident::True }
    } else {
        quote! { #crate_ident::False }
    };
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics #crate_ident::Component for #ident #type_generics #where_clause {
            type IsSingleton = #is_singleton_type;
        }
    };
    output.into()
}

fn finish_system_call(entity_type: &Ident) -> proc_macro2::TokenStream {
    let label = format!("{entity_type}::modor_finish");
    let label_tokens = TokenTree::Literal(Literal::string(&label));
    quote! {
        .finish(#label_tokens)
    }
}

//////////////////////////////////////////////////

// TODO: what is the usefulness of darling here ?
#[derive(FromDeriveInput, Debug)]
struct ParsedSystemParam {
    ident: Ident,
    generics: Generics,
    data: ast::Data<(), ParsedSystemParamField>,
}

impl ParsedSystemParam {
    fn field_type_tuple(data: ast::Data<(), ParsedSystemParamField>) -> ParsedSystemParamField {
        todo!()
    }
}

#[derive(FromField, Debug)]
struct ParsedSystemParamField {
    ident: Option<Ident>,
    ty: Type,
}

fn rename_first_lifetime(generics: &Generics, new_name: &str) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Lifetime(lifetime) = param {
            lifetime.lifetime.ident = Ident::new(new_name, lifetime.lifetime.ident.span());
            break;
        }
    }
    generics
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SystemParam)]
#[proc_macro_error::proc_macro_error]
pub fn system_param_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ParsedSystemParam {
        ident,
        generics,
        data,
        ..
    } = match ParsedSystemParam::from_derive_input(&item) {
        Ok(parsed) => parsed,
        Err(error) => return error.write_errors().into(),
    };
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let renamed_generics = rename_first_lifetime(&generics, "b");
    let (_, renamed_type_generics, _) = renamed_generics.split_for_impl();
    let fields = data
        .take_struct()
        .expect_or_abort("custom system param must be a struct");
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let field_names: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            f.ident
                .clone()
                .unwrap_or_else(|| Ident::new(&i.to_string(), Span::call_site()))
        })
        .collect();
    let tuple_ids = 0..field_names.len();
    let output = quote! {
        impl #impl_generics #crate_ident::CustomSystemParam for #ident #type_generics #where_clause {
            type ConstParam<'b> = #ident #renamed_type_generics;
            type Param<'b> = #ident #renamed_type_generics;
            type Tuple = (#(#field_types,)*);

            fn from_tuple_const_param_mut_param<'b>(
                _tuple: <<Self::Tuple as #crate_ident::QuerySystemParamWithLifetime<'b>>::ConstParam as #crate_ident::SystemParamWithLifetime<'b>>::Param,
            ) -> <#crate_ident::Custom<Self::ConstParam<'b>> as #crate_ident::SystemParamWithLifetime<'b>>::Param
            where
                Self::Tuple: #crate_ident::QuerySystemParam,
            {
                unreachable!()
            }

            fn from_tuple_const_param(
                _tuple: <Self::Tuple as #crate_ident::QuerySystemParamWithLifetime<'_>>::ConstParam,
            ) -> #crate_ident::Custom<Self::ConstParam<'_>>
            where
                Self::Tuple: #crate_ident::QuerySystemParam,
            {
                unreachable!()
            }

            fn from_tuple_mut_param(
                tuple: <Self::Tuple as #crate_ident::SystemParamWithLifetime<'_>>::Param,
            ) -> #crate_ident::Custom<Self::Param<'_>> {
                #crate_ident::Custom::new(#ident {
                    #(#field_names: tuple.#tuple_ids)*
                })
            }
        }
    };
    output.into()
}
