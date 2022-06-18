mod async_t;
mod impl_trait;

use proc_macro::TokenStream;
use syn::{ItemImpl, ItemTrait};

/// requires nightly and cannot be used with dynamic dispatch.
/// also has limited support for generics.
/// | it doesn't use any dynamic dispatch and is a complete zero cost wrapper.
/// | requires features [ generic_associated_types, type_alias_impl_trait ]
#[proc_macro_attribute]
pub fn async_trait(_: TokenStream, tokens: TokenStream) -> TokenStream {
    match syn::parse::<ItemTrait>(tokens.clone()) {
        Ok(inner_trait) => async_t::trait_implementation(inner_trait),
        Err(_) => {
            let inner_trait = syn::parse::<ItemImpl>(tokens).unwrap();
            async_t::implementation(inner_trait)
        }
    }
}

/// impl overload! add superpowers to your traits!
/// makes existential types returnable for methods **recursively**,
/// meaning that these methods can be more flexible than normal rust functions.
/// The only downside is that it doesn't support dynamic dispatch.
/// ```norun
/// #[impl_trait]
/// trait A {
///     fn a(&self) -> (
///         impl Display, // supports using `impl Trait` as a first-class type
///         Result<impl AllTraitsSupported, impl Iterator<Item = impl IsOk>>,
///         [impl Display; 30],
///         fn(impl AnyTrait) -> impl Any
///     );
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_trait(_: TokenStream, tokens: TokenStream) -> TokenStream {
    let ts = match syn::parse::<ItemTrait>(tokens.clone()) {
        Ok(inner_trait) => impl_trait::impl_trait(inner_trait),
        Err(_) => {
            let inner_trait = syn::parse::<ItemImpl>(tokens).unwrap();
            impl_trait::trait_implementation(inner_trait)
        }
    };

    ts.into()
}
