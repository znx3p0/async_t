
#[rustversion::any(nightly, beta)]
pub use impl_trait::async_trait;

#[rustversion::stable]
pub use async_trait::async_trait as async_trait;

