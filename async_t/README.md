# async_t

This library allows for zero-cost compile-time async-traits.
This library needs nightly and features `generic_associated_types` and `type_alias_impl_trait` to be enabled.
It doesn't support dynamic dispatch and has limited generic support.
Compiling in stable will automatically use dtolnay's async_trait instead.

```rust
#[async_trait]
trait Spawn {
    // supports self, &self, &mut self and no self
    async fn spawn() -> JoinHandle<()>;
}

#[async_trait]
impl Spawn for Spawner {
    async fn spawn() -> JoinHandle<()> {
        task::spawn(async {
            // ...
        })
    }
}

async fn spawn<T: Spawn>() -> JoinHandle<()> {
    T::spawn().await
}
```