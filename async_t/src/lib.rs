#[cfg(not(feature = "boxed"))]
pub use async_t_internal::async_trait;

pub use async_t_internal::impl_trait;

#[cfg(feature = "boxed")]
pub use async_trait::async_trait;
