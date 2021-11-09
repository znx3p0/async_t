
#![allow(unused_imports)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    spanned::Spanned,
    Block, GenericParam, ImplItem, ItemImpl, ItemTrait, Lifetime, LifetimeDef,
    TraitItem,
};

/// requires nightly and cannot be used with dynamic dispatch.
/// also has limited support for generics.
/// | it doesn't use any dynamic dispatch and is a complete zero cost wrapper.
/// | requires features [ generic_associated_types, type_alias_impl_trait ]
/// | if the compiler is not nighlty, it will default to dtolnay's async_trait
/// | known bug: lifetime errors might be thrown when bounds are different than declared
#[proc_macro_attribute]
pub fn async_trait(_: TokenStream, tokens: TokenStream) -> TokenStream {
    match syn::parse::<ItemTrait>(tokens.clone()) {
        Ok(tr) => process_trait(tr),
        Err(_) => {
            let imp = syn::parse::<ItemImpl>(tokens).unwrap();
            process_impl(imp)
        }
    }
}

fn process_trait(mut tr: ItemTrait) -> TokenStream {
    let span = tr.span();
    let mut original = tr.clone();
    original.ident = format_ident!("Dyn{}", original.ident);
    let mut extra_types = vec![];
    for item in tr.items.iter_mut() {
        if let TraitItem::Method(method) = item {
            if method.sig.asyncness.is_some() {
                let ident = format_ident!("{}Fut", method.sig.ident);

                // generate where clause from generics
                let mut needs_where = false;
                let gl = tr.generics.params.clone()
                    .into_iter()
                    .map(|s| {
                        match s {
                            GenericParam::Type(s) => {
                                needs_where = true;
                                let ident = s.ident;
                                quote!(#ident: 'async_t)
                            },
                            _=> quote!(),
                        }
                    })
                    .collect::<Vec<_>>();
                let wh = match needs_where {
                    true => quote!(where #(#gl),*),
                    false => quote!(),
                };

                let extra = match method.sig.output.clone() {
                    syn::ReturnType::Default => {
                        quote!(#[allow(non_camel_case_types)] type #ident<'async_t>: std::future::Future<Output = ()> + Send + 'async_t #wh;)
                    }
                    syn::ReturnType::Type(_, ty) => {
                        quote!(#[allow(non_camel_case_types)] type #ident<'async_t>: std::future::Future<Output = #ty> + Send + 'async_t #wh;)
                    }
                };
                extra_types.push(extra.clone());

                // remove async since it is not yet supported by traits
                method.sig.asyncness = None;
                method.sig.output = syn::parse2(quote!( -> Self::#ident<'async_t> )).unwrap();
            }

            method
                .sig
                .generics
                .params
                .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                    "'async_t", span,
                ))))
        }
    }
    for ty in extra_types {
        tr.items.push(TraitItem::Verbatim(ty))
    }

    quote!(
        #tr
    ).into()
}

fn process_impl(mut imp: ItemImpl) -> TokenStream {
    let span = imp.span();

    let mut extra_types = vec![];
    for item in imp.items.iter_mut() {
        if let syn::ImplItem::Method(method) = item {
            // add async to method
            if method.sig.asyncness.is_some() {

                // add where to extra ype representing async method
                let mut needs_where = false;
                let gl = imp.generics.type_params()
                    .into_iter()
                    .map(|s| {
                            needs_where = true;
                            let ident = &s.ident;
                            quote!(#ident: 'async_t)
                    })
                    .collect::<Vec<_>>();
                let wh = match needs_where {
                    true => quote!(where #(#gl),*),
                    false => quote!(),
                };

                let ident = format_ident!("{}Fut", method.sig.ident);
                let extra = match method.sig.output.clone() {
                    syn::ReturnType::Default => {
                        quote!(#[allow(non_camel_case_types)] type #ident<'async_t> #wh = impl std::future::Future<Output = ()> + Send + 'async_t;)
                    }
                    syn::ReturnType::Type(_, ty) => {
                        quote!(#[allow(non_camel_case_types)] type #ident<'async_t> #wh = impl std::future::Future<Output = #ty> + Send + 'async_t;)
                    }
                };

                extra_types.push(extra);
                method.sig.asyncness = None;
                method.sig.output = syn::parse2(quote!( -> Self::#ident<'async_t> )).unwrap();

                // add 'async_t lifetime to method
                method
                    .sig
                    .generics
                    .params
                    .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                        "'async_t", span,
                    ))));
                let block = method.block.clone();
                let block = syn::parse2::<Block>(quote!({ async move #block })).unwrap();
                method.block = block;
            }
        }
    }
    for ty in extra_types {
        imp.items.push(ImplItem::Verbatim(ty))
    }
    quote!(
        #imp
    ).into()
}
