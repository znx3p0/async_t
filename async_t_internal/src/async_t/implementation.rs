use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ImplItem, ItemImpl};

pub(crate) fn implementation(mut inner_trait: ItemImpl) -> TokenStream {
    inner_trait.items.iter_mut().for_each(|item| match item {
        ImplItem::Method(method) => {
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
                        quote!(-> impl core::future::Future<Output = ()> + 'future #send)
                    }
                    syn::ReturnType::Type(_, ty) => {
                        quote!(-> impl core::future::Future<Output = #ty> + 'future #send)
                    }
                };
                method.sig.output = syn::parse2(ret).unwrap();
                method
                    .sig
                    .generics
                    .params
                    .push(syn::parse2(quote!('future)).unwrap());
                let block = &method.block;
                method.block = syn::parse2(quote! {
                    {
                        async move {
                            #block
                        }
                    }
                })
                .unwrap();
                method.sig.generics.type_params_mut().for_each(|param| {
                    param.bounds.push(syn::parse2(quote!('future)).unwrap());
                });
                Some(())
            });
        }
        _ => (),
    });
    crate::impl_trait::trait_implementation(inner_trait)
}
