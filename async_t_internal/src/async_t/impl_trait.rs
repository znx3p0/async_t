use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ItemTrait;

pub(crate) fn trait_implementation(mut inner_trait: ItemTrait) -> TokenStream {
    inner_trait.items.iter_mut().for_each(|item| match item {
        syn::TraitItem::Method(method) => {
            method.sig.asyncness.take().and_then(|_| {
                let (index, send) = {
                    let mut index = None;
                    if method
                        .attrs
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| s.path.is_ident(&format_ident!("unsend")))
                        .any(|(i, _)| {
                            index = Some(i);
                            true
                        })
                    {
                        (index, quote!())
                    } else {
                        (index, quote!(+ Send))
                    }
                };
                index.and_then(|index| {
                    method.attrs.remove(index);
                    Some(())
                });
                let ret = match &method.sig.output {
                    syn::ReturnType::Default => {
                        quote!(-> impl ::core::future::Future<Output = ()> + 'async_trait #send)
                    }
                    syn::ReturnType::Type(_, ty) => {
                        quote!(-> impl ::core::future::Future<Output = #ty> + 'async_trait #send)
                    }
                };
                method.sig.output = syn::parse2(ret).unwrap();
                method
                    .sig
                    .generics
                    .params
                    .push(syn::parse2(quote!('async_trait)).unwrap());
                method.sig.generics.type_params_mut().for_each(|param| {
                    param
                        .bounds
                        .push(syn::parse2(quote!('async_trait)).unwrap());
                });
                Some(())
            });
        }
        _ => (),
    });
    crate::impl_trait::impl_trait(inner_trait)
}
