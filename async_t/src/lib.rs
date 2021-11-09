
#[rustversion::any(nightly, beta)]
pub use async_t_internal::async_trait;

#[rustversion::stable]
pub use async_trait::async_trait as async_trait;

