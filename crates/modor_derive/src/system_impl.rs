use crate::{common, idents};
use darling::util::{Flag, PathList, SpannedValue};
use darling::FromMeta;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{
    parse_quote, parse_quote_spanned, Attribute, Expr, ImplItem, ImplItemMethod, ItemImpl,
    ItemStruct, NestedMeta, Path, Type,
};

pub(crate) struct SystemImpl<'a> {
    crate_name: String,
    item: &'a ItemImpl,
    ident: Ident,
    action_ident: Ident,
    method_attributes: HashMap<&'a Ident, RunAttribute>,
    runner_arg: Ident,
}

impl<'a> SystemImpl<'a> {
    pub(crate) fn new(item: &'a ItemImpl) -> Result<Self, TokenStream> {
        Self::method_attributes(item)
            .map_err(darling::Error::write_errors)
            .map(|method_attributes| {
                let ident = idents::extract_type_ident(&item.self_ty);
                Self {
                    crate_name: idents::crate_name(),
                    item,
                    action_ident: common::ident_with_suffix(&ident, "Action"),
                    ident,
                    method_attributes,
                    runner_arg: parse_quote! { runner },
                }
            })
    }

    pub(crate) fn component_systems_impl(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let cleaned_item = self.clean_item();
        let impl_header = self.impl_header();
        let action_type = self.action_type();
        let action_struct = self.action_struct();
        let update_calls = self.update_calls();
        let runner_arg = &self.runner_arg;
        let finish_label = Literal::string(&format!("{}::modor_finish", self.ident));
        quote! {
            #cleaned_item

            #impl_header {
                type Action = #action_type;

                #[allow(unused_qualifications)]
                fn on_update(#runner_arg: #crate_::SystemRunner<'_>) -> #crate_::FinishedSystemRunner {
                    #update_calls.finish(#finish_label)
                }
            }

            #[allow(unused_qualifications)]
            #[doc(hidden)]
            #[non_exhaustive]
            #[derive(#crate_::Action)]
            #action_struct
        }
    }

    fn method_attributes(item: &'a ItemImpl) -> darling::Result<HashMap<&'a Ident, RunAttribute>> {
        let mut errors = darling::Error::accumulator();
        let attributes: HashMap<_, _> = item
            .items
            .iter()
            .filter_map(|i| {
                if let ImplItem::Method(method) = i {
                    Some((
                        &method.sig.ident,
                        errors.handle_in(|| Self::method_attribute(method))??,
                    ))
                } else {
                    None
                }
            })
            .collect();
        for attribute in attributes.values() {
            for result in attribute.validate() {
                errors.handle(result);
            }
        }
        errors.finish()?;
        Ok(attributes)
    }

    fn method_attribute(method: &ImplItemMethod) -> darling::Result<Option<RunAttribute>> {
        let supported_attributes: Vec<_> = method
            .attrs
            .iter()
            .filter(|a| Self::is_supported_attribute(a))
            .collect();
        if supported_attributes.len() > 1 {
            return Err(syn::Error::new(
                supported_attributes[1].span(),
                "found more than one `run*` attribute",
            )
            .into());
        }
        supported_attributes
            .into_iter()
            .map(|a| {
                let path = &a.path;
                let tokens = &a.tokens;
                let meta: NestedMeta = parse_quote!(#path #tokens);
                RunAttribute::from_list(&[meta])
            })
            .next()
            .map_or(Ok(None), |a| a.map(Some))
    }

    fn clean_item(&self) -> ItemImpl {
        let mut item = self.item.clone();
        for inner_item in &mut item.items {
            if let ImplItem::Method(method) = inner_item {
                method.attrs.retain(|a| !Self::is_supported_attribute(a));
            }
        }
        item
    }

    fn is_supported_attribute(attribute: &Attribute) -> bool {
        attribute.path.segments.len() == 1
            && ATTRIBUTE_NAMES.contains(&attribute.path.segments[0].ident.to_string().as_str())
    }

    fn impl_header(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        common::impl_header(
            &self.item.generics,
            &self.ident,
            &parse_quote! { #crate_::ComponentSystems },
        )
    }

    fn action_type(&self) -> Type {
        let type_ = &self.action_ident;
        let (_impl_generics, type_generics, _where_clause) = self.item.generics.split_for_impl();
        parse_quote! { #type_ #type_generics }
    }

    fn action_struct(&self) -> ItemStruct {
        let mut dependencies = self.action_dependencies();
        if let Some(phantom) = self.phantom_action_dependency() {
            dependencies.push(phantom);
        }
        common::tuple_struct(
            &parse_quote! { pub },
            &self.action_ident,
            &self.item.generics,
            &dependencies,
        )
    }

    fn action_dependencies(&self) -> Vec<Type> {
        self.method_attributes
            .values()
            .flat_map(RunAttribute::dependencies)
            .filter_map(|d| d.action_type(&self.crate_name))
            .collect()
    }

    fn phantom_action_dependency(&self) -> Option<Type> {
        let generic_type_names = self.item.generics.type_params().map(|t| &t.ident);
        self.item
            .generics
            .lt_token
            .is_some()
            .then(|| parse_quote! { std::marker::PhantomData <(#(#generic_type_names,)*)> })
    }

    fn update_calls(&self) -> Expr {
        let runner_arg = &self.runner_arg;
        self.method_attributes
            .iter()
            .filter_map(|(m, a)| self.update_call(m, a))
            .fold(parse_quote! { #runner_arg }, |o, c| parse_quote! { #o.#c })
    }

    fn update_call(&self, method: &Ident, attribute: &RunAttribute) -> Option<Expr> {
        let span = method.span();
        let crate_ = Ident::new(&self.crate_name, span);
        let label = Literal::string(&format!("{}::{}", self.ident, method));
        Some(match attribute {
            RunAttribute::Run(_) => parse_quote_spanned! {
                span =>
                run(#crate_::system!(Self::#method), #label)
            },
            RunAttribute::RunAs(dependency) => {
                let action = dependency.action_type(&self.crate_name)?;
                parse_quote_spanned! {
                    span =>
                    run_as::<#action>(#crate_::system!(Self::#method), #label)
                }
            }
            RunAttribute::RunAfter(dependencies) => {
                let actions = dependencies.action_types(&self.crate_name);
                let constraint = common::recursive_tuple(actions);
                parse_quote_spanned! {
                    span =>
                    run_constrained::<#constraint>(#crate_::system!(Self::#method), #label)
                }
            }
            RunAttribute::RunAfterPrevious(_) => {
                parse_quote_spanned! {
                    span =>
                    and_then::<()>(#crate_::system!(Self::#method), #label)
                }
            }
            RunAttribute::RunAfterPreviousAnd(dependencies) => {
                let actions = dependencies.action_types(&self.crate_name);
                let constraint = common::recursive_tuple(actions);
                parse_quote_spanned! {
                    span =>
                    and_then::<#constraint>(#crate_::system!(Self::#method), #label)
                }
            }
        })
    }
}

const ATTRIBUTE_NAMES: [&str; 5] = [
    "run",
    "run_as",
    "run_after",
    "run_after_previous",
    "run_after_previous_and",
];

#[derive(Debug, FromMeta)]
enum RunAttribute {
    Run(#[allow(unused_tuple_struct_fields)] Flag),
    RunAs(RunDependency),
    RunAfter(RunDependencies),
    RunAfterPrevious(#[allow(unused_tuple_struct_fields)] Flag),
    RunAfterPreviousAnd(RunDependencies),
}

impl RunAttribute {
    fn validate(&self) -> Vec<darling::Result<()>> {
        self.dependencies()
            .iter()
            .map(RunDependency::validate)
            .collect()
    }

    fn dependencies(&self) -> Vec<RunDependency> {
        match self {
            Self::RunAs(dependency) => vec![dependency.clone()],
            Self::RunAfter(dependencies) | Self::RunAfterPreviousAnd(dependencies) => {
                dependencies.0.clone()
            }
            Self::Run(_) | Self::RunAfterPrevious(_) => vec![],
        }
    }
}

#[derive(Clone, Debug)]
struct RunDependencies(Vec<RunDependency>);

impl FromMeta for RunDependencies {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let dependencies: darling::Result<Vec<RunDependency>> = items
            .iter()
            .map(|m| RunDependency::from_list(&[m.clone()]))
            .collect();
        Ok(Self(dependencies?))
    }
}

impl RunDependencies {
    fn action_types<'a>(&'a self, crate_name: &'a str) -> impl Iterator<Item = Type> + 'a {
        self.0.iter().filter_map(|d| d.action_type(crate_name))
    }
}

#[derive(Debug, Clone, FromMeta)]
enum RunDependency {
    Action(SpannedValue<PathList>),
    Component(SpannedValue<PathList>),
}

impl RunDependency {
    fn validate(&self) -> darling::Result<()> {
        let (Self::Action(paths) | Self::Component(paths)) = self;
        if paths.len() == 1 {
            Ok(())
        } else {
            Err(syn::Error::new(paths.span(), "expected exactly one type").into())
        }
    }

    fn action_type(&self, crate_name: &str) -> Option<Type> {
        match self {
            Self::Action(paths) => paths.iter().map(|p| parse_quote! { #p }).next(),
            Self::Component(paths) => paths
                .iter()
                .map(|p| Self::component_to_action_type(p, crate_name))
                .next(),
        }
    }

    fn component_to_action_type(component_path: &Path, crate_name: &str) -> Type {
        let span = component_path.span();
        let crate_ = Ident::new(crate_name, span);
        parse_quote_spanned! { span => <#component_path as #crate_::ComponentSystems>::Action }
    }
}
