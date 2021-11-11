#![allow(unused_imports)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    spanned::Spanned, Block, GenericParam, ImplItem, ItemImpl, ItemTrait, Lifetime, LifetimeDef,
    Path, ReturnType, TraitItem,
};

/// requires nightly and cannot be used with dynamic dispatch.
/// also has limited support for generics.
/// | it doesn't use any dynamic dispatch and is a complete zero cost wrapper.
/// | requires features [ generic_associated_types, type_alias_impl_trait ]
/// | if the compiler is not nightly, it will default to dtolnay's async_trait
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
    let mut extra_types = vec![];

    let impl_generics = tr.generics.type_params();
    let has_generics = impl_generics.collect::<Vec<_>>().len() > 0;
    let impl_generics = tr.generics.type_params();
    let where_clause = if has_generics {
        quote!(where #(#impl_generics: 'async_t,)*)
    } else {
        quote!()
    };

    for item in tr.items.iter_mut() {
        if let TraitItem::Method(method) = item {
            if method.sig.asyncness.take().is_some() {
                let mut unsend = false;
                let mut attr_iter = method.attrs.iter().enumerate();
                let index = loop {
                    if let Some((index, attr)) = attr_iter.next() {
                        if quote!(#attr).to_string() == quote!(#[unsend]).to_string() {
                            unsend = true;
                            break Some(index);
                        };
                    } else {
                        break None;
                    }
                };
                if let Some(i) = index {
                    method.attrs.remove(i);
                }

                let tp = method.sig.generics.type_params().map(|s| &s.ident);
                let lf = method.sig.generics.lifetimes();
                let name = format_ident!("{}Fut", &method.sig.ident);

                let ty = match method.sig.output.clone() {
                    syn::ReturnType::Default => {
                        quote!(())
                    }
                    syn::ReturnType::Type(_, ty) => {
                        quote!(#ty)
                    }
                };
                let extra_type = quote!(type #name<'async_t #(,#lf: 'async_t)* #(,#tp: 'async_t)*>: Future<Output = #ty> + 'async_t #where_clause;);
                extra_types.push(extra_type);

                let tp = method.sig.generics.type_params().map(|s| &s.ident);
                let lf = method.sig.generics.lifetimes();
                let name = format_ident!("{}Fut", &method.sig.ident);
                let ret_ty: ReturnType =
                    syn::parse2(quote!(-> Self::#name<'async_t #(,#lf)* #(,#tp)*>)).unwrap();
                method.sig.output = ret_ty;

                method
                    .sig
                    .generics
                    .type_params_mut()
                    .map(|p| {
                        if !unsend {
                            p.bounds.push(syn::parse2(quote!(Send)).unwrap());
                        }
                        p.bounds.push(syn::parse2(quote!('async_t)).unwrap());
                    })
                    .for_each(drop);

                method
                    .sig
                    .generics
                    .lifetimes_mut()
                    .map(|s| s.bounds.push_value(syn::parse2(quote!('async_t)).unwrap()))
                    .for_each(drop);

                method
                    .sig
                    .generics
                    .params
                    .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                        "'async_t", span,
                    ))))
            }
        }
    }
    for ty in extra_types {
        tr.items.push(TraitItem::Verbatim(ty))
    }

    quote!(
        #tr
    )
    .into()
}

fn process_impl(mut imp: ItemImpl) -> TokenStream {
    let span = imp.span();
    let mut extra_types = vec![];

    let impl_generics = imp.generics.type_params();
    let has_generics = impl_generics.collect::<Vec<_>>().len() > 0;
    let impl_generics = imp.generics.type_params();
    let where_clause = if has_generics {
        quote!(where #(#impl_generics: 'async_t,)*)
    } else {
        quote!()
    };
    // panic!("{}", where_clause.to_string());

    for item in imp.items.iter_mut() {
        if let ImplItem::Method(method) = item {
            if method.sig.asyncness.take().is_some() {
                let mut unsend = false;
                let mut attr_iter = method.attrs.iter().enumerate();
                let index = loop {
                    if let Some((index, attr)) = attr_iter.next() {
                        if quote!(#attr).to_string() == quote!(#[unsend]).to_string() {
                            unsend = true;
                            break Some(index);
                        };
                    } else {
                        break None;
                    }
                };
                if let Some(i) = index {
                    method.attrs.remove(i);
                }

                let gen = method.sig.generics.clone();
                // let tp = gen.type_params();
                let tp = gen.type_params().map(|s| &s.ident);
                let lf = gen.lifetimes();

                let name = format_ident!("{}Fut", &method.sig.ident);

                let ty = match method.sig.output.clone() {
                    syn::ReturnType::Default => {
                        quote!(())
                    }
                    syn::ReturnType::Type(_, ty) => {
                        quote!(#ty)
                    }
                };

                let extra_type = quote!(
                    type #name<'async_t #(,#lf: 'async_t)* #(,#tp: 'async_t)*> #where_clause = impl std::future::Future<Output = #ty>;
                );
                extra_types.push(extra_type);

                let tp = method.sig.generics.type_params().map(|s| &s.ident);
                // let tp = method.sig.generics.type_params();
                let lf = method.sig.generics.lifetimes();
                let name = format_ident!("{}Fut", &method.sig.ident);
                let ret_ty: ReturnType =
                    syn::parse2(quote!(-> Self::#name<'async_t #(,#lf)* #(,#tp)*>)).unwrap();
                method.sig.output = ret_ty;

                method
                    .sig
                    .generics
                    .type_params_mut()
                    .map(|p| {
                        if !unsend {
                            p.bounds.push(syn::parse2(quote!(Send)).unwrap());
                        }
                        p.bounds.push(syn::parse2(quote!('async_t)).unwrap());
                    })
                    .for_each(drop);

                let block = &method.block;
                let block: Block = syn::parse2(quote!({ async move { #block } })).unwrap();
                method.block = block;

                method
                    .sig
                    .generics
                    .lifetimes_mut()
                    .map(|s| s.bounds.push_value(syn::parse2(quote!('async_t)).unwrap()))
                    .for_each(drop);

                method
                    .sig
                    .generics
                    .params
                    .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                        "'async_t", span,
                    ))))
            }
        }
    }
    for ty in extra_types {
        imp.items.push(ImplItem::Verbatim(ty))
    }

    quote!(
        #imp
    )
    .into()
}
