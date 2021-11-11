# async_t

This library allows for zero-cost compile-time async-traits.
This library needs nightly and features `generic_associated_types` and `type_alias_impl_trait` to be enabled.
Compiling in stable will automatically use dtolnay's async_trait instead.

It supports everything a normal trait would except:
- default async methods
- blanket implementations
- dynamic dispatch

It can also have problems with lifetimes where they have to be specified.

```rust
// spawn example

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


```rust
#[async_trait]
trait Sleeper {
    #[unsend]
    async fn sleep<T>(t: T) -> T;
}

#[async_trait]
impl Sleeper for () {
    #[unsend]
    async fn sleep<T>(t: T) -> T {
        task::sleep(Duration::from_secs(2)).await;
        t
    }
}
```


