# async_t

Give up on dynamic dispatch and get compile-time async traits (with a few bonuses)

`async_t` provides a `#[impl_trait]` macro that allows any trait to return
existential types; e.g. `-> impl Future` and a `#[async_trait]` macro that
wraps your async methods under a `-> impl Future` existential type.

This allows for complete zero-cost async-traits, and allows for recursive existential
return types such as `Result<impl Display, impl Debug>`.

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
        task::sleep(Duration::from_secs(2)).await; // await inside
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

`async_t` also supports impl return types in traits (async traits are desigared to recursive impl return types)

```rust
#[impl_trait] // #[async_trait] can also be used
trait RetDebug {
    fn ret_debug() -> impl Debug;
}
```

## Features

`async_t` supports the `boxed` feature which will set `async_trait` to be the one from the `async-trait` crate from dtolnay.
