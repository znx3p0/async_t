#[rustversion::any(nightly, beta)]
pub use async_t_internal::async_trait;

pub use async_t_internal::impl_trait;

#[rustversion::stable]
pub use async_trait::async_trait;
