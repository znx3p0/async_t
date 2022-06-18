// trait declaration

/*
#[impl_trait]
trait Test {
    fn test() -> impl IsOk;
}
*/

use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Add;
use syn::{FnArg, ImplItem, ImplItemMethod, ItemImpl, Type, TypeParam, TypeParamBound};

pub(crate) struct TraitImplementation {
    pub(crate) inner_trait: ItemImpl,
}

pub(crate) fn trait_implementation(inner_trait: ItemImpl) -> TokenStream {
    TraitImplementation::new(inner_trait).process()
}

impl TraitImplementation {
    pub(crate) fn new(inner_trait: ItemImpl) -> Self {
        Self { inner_trait }
    }
    pub(crate) fn process(self) -> TokenStream {
        let mut t = self.inner_trait;
        let mut new_types = vec![];
        let generics = t.generics.type_params().collect::<Vec<_>>();
        t.items
            .iter_mut()
            .map(|mut s| match &mut s {
                ImplItem::Method(method) => process_method(method, &mut new_types, &generics),
                ImplItem::Verbatim(_)
                | ImplItem::Const(_)
                | ImplItem::Macro(_)
                | ImplItem::Type(_) => (),
                _ => abort!(s.span(), "please report this bug"),
            })
            .for_each(drop);
        let mut new_types = new_types
            .into_iter()
            .map(|s| ImplItem::Verbatim(s.into()))
            .collect();
        t.items.append(&mut new_types);
        quote!(#t).into()
    }
}

fn process_method<'a>(
    method: &'a mut ImplItemMethod,
    new_types: &'a mut Vec<TokenStream>,
    trait_lifetimes: &'a [&'a TypeParam],
) {
    let mut register = MethodRegister::new(method, new_types, 0, trait_lifetimes);
    if let syn::ReturnType::Type(arr, mut ty) = method.sig.output.clone() {
        process_type(&mut ty, &mut register);
        method.sig.output = syn::ReturnType::Type(arr, ty);
    }
}

fn process_type(ty: &mut Type, register: &mut MethodRegister) {
    match ty {
        Type::Array(arr) => process_type(&mut arr.elem, register),
        Type::Group(group) => process_type(&mut group.elem, register),
        Type::Paren(paren) => process_type(&mut paren.elem, register),
        Type::Ptr(ptr) => process_type(&mut ptr.elem, register),
        Type::Reference(ptr) => process_type(&mut ptr.elem, register),
        Type::Slice(slice) => process_type(&mut slice.elem, register),

        Type::BareFn(func) => {
            func.inputs
                .iter_mut()
                .for_each(|s| process_type(&mut s.ty, register));
            if let syn::ReturnType::Type(_, ty) = &mut func.output {
                process_type(ty, register)
            }
        }
        Type::Path(path) => {
            path.path
                .segments
                .iter_mut()
                .for_each(|path| match &mut path.arguments {
                    syn::PathArguments::AngleBracketed(bracketed) => {
                        bracketed.args.iter_mut().for_each(|arg| match arg {
                            syn::GenericArgument::Type(ty) => process_type(ty, register),
                            syn::GenericArgument::Binding(binding) => {
                                process_type(&mut binding.ty, register)
                            }
                            _ => (),
                        })
                    }
                    syn::PathArguments::Parenthesized(paren) => {
                        paren
                            .inputs
                            .iter_mut()
                            .for_each(|ty| process_type(ty, register));
                        if let syn::ReturnType::Type(_, ty) = &mut paren.output {
                            process_type(ty, register)
                        }
                    }
                    syn::PathArguments::None => (),
                })
        }
        Type::Tuple(tuple) => tuple
            .elems
            .iter_mut()
            .for_each(|ty| process_type(ty, register)),

        Type::ImplTrait(tr) => {
            tr.bounds.iter_mut().for_each(|s| {
                if let TypeParamBound::Trait(s) = s {
                    s.path
                        .segments
                        .iter_mut()
                        .for_each(|path| match &mut path.arguments {
                            syn::PathArguments::AngleBracketed(bracketed) => {
                                bracketed.args.iter_mut().for_each(|arg| match arg {
                                    syn::GenericArgument::Type(ty) => process_type(ty, register),
                                    syn::GenericArgument::Binding(binding) => {
                                        process_type(&mut binding.ty, register)
                                    }
                                    _ => (),
                                })
                            }
                            syn::PathArguments::Parenthesized(paren) => {
                                paren
                                    .inputs
                                    .iter_mut()
                                    .for_each(|ty| process_type(ty, register));
                                if let syn::ReturnType::Type(_, ty) = &mut paren.output {
                                    process_type(ty, register)
                                }
                            }
                            syn::PathArguments::None => (),
                        })
                }
            });
            *ty = register.register(&tr.bounds);
        }
        Type::Never(_)
        | Type::Verbatim(_)
        | Type::Macro(_)
        | Type::TraitObject(_)
        | Type::Infer(_) => (), // these types don't encapsulate any other type.
        _ => abort!(ty.span(), "please report this bug"),
    }
}

struct MethodRegister<'a> {
    method: &'a ImplItemMethod,
    new_types: &'a mut Vec<TokenStream>,
    counter: u64,
    types: &'a [&'a TypeParam],
}

impl<'a> MethodRegister<'a> {
    fn new(
        method: &'a ImplItemMethod,
        new_types: &'a mut Vec<TokenStream>,
        counter: u64,
        types: &'a [&'a TypeParam],
    ) -> Self {
        Self {
            method,
            new_types,
            counter,
            types,
        }
    }

    fn register(&mut self, bounds: &Punctuated<TypeParamBound, Add>) -> Type {
        let ident = &self.method.sig.ident;
        let mut where_clause = self.method.sig.generics.clone();
        let where_clause = where_clause.make_where_clause();

        // check for self lifetimes
        self.method.sig.inputs.first().and_then(|arg| {
            if let FnArg::Receiver(receiver) = arg {
                receiver.reference.as_ref().and_then(|(_, lt)| {
                    lt.as_ref().and_then(|lt| {
                        where_clause
                            .predicates
                            .push(syn::parse2(quote!(Self: #lt)).unwrap());
                        Some(())
                    });
                    Some(())
                });
                Some(())
            } else {
                Some(())
            }
        });

        let (bound_generics, generics, _) = &self.method.sig.generics.split_for_impl();
        let num = self.counter;
        let ident = format_ident!("impl_trait_{}_{}", ident, num);

        let mut extra_bounds = vec![];
        self.types.clone().iter().for_each(|s| {
            for lt in self.method.sig.generics.lifetimes() {
                let ident = &s.ident;
                extra_bounds.push(syn::parse2(quote!(#ident: #lt)).unwrap());
            }
        });
        for bound in extra_bounds {
            where_clause.predicates.push(bound);
        }

        let ts = quote!(
            #[allow(non_camel_case_types)]
            type #ident #bound_generics #where_clause = impl #bounds;
        );

        self.counter += 1;
        self.new_types.push(ts.into());
        let ty = Type::Path(syn::parse2(quote!(Self::#ident #generics)).unwrap());
        ty
    }
}
