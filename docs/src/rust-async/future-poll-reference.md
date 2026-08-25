# Future/Poll 固定源码与 API 参考

[调用契约主章](future-poll.md)已经建立 `Future::poll`、`Pending`、`Ready` 与完成后边界，本页继续完整走读 Rust 1.91.1 四个固定源码单元中不必打断主推理链的公共表面。

## `Poll` 如何查询和转换已经返回的值

`map`、`map_ok` 和 `map_err` 不是 Future 的方法，而是固定 [`poll.rs`](https://github.com/rust-lang/rust/blob/ed61e7d7e242494fb7057f2657300d9e77bb4fcb/library/core/src/task/poll.rs)中 `Poll` 自己的方法。
两个真实调用覆盖 `map` 的全部分支，中文行尾注释由本书添加：

```rust
# use std::task::Poll;
let ready = Poll::Ready(2_u32).map(|value| value + 1); // `Ready(T) -> Ready(f(T))`
let pending = Poll::<u32>::Pending.map(|value| value + 1); // 仍是 `Pending`，closure 不执行。
# assert_eq!(ready, Poll::Ready(3));
# assert_eq!(pending, Poll::Pending);
```

`map` 只接收已有的 `Poll<T>` 和 closure，没有 Future 或 `Context`，因此不会发起 poll、登记 Waker、制造进展或保存 Future 状态。

当 payload 是 `Result<T, E>` 时，`map_ok` 和 `map_err` 只进入各自匹配的业务结果分支：

| 输入 | `map_ok(f)` | `map_err(g)` |
| --- | --- | --- |
| `Poll::Pending` | `Poll::Pending` | `Poll::Pending` |
| `Poll::Ready(Ok(value))` | `Poll::Ready(Ok(f(value)))` | 原样保留 `Ok(value)` |
| `Poll::Ready(Err(error))` | 原样保留 `Err(error)` | `Poll::Ready(Err(g(error)))` |

Rust 1.91.1 还为 `Poll<Option<Result<T, E>>>` 提供同名方法，它们只转换 `Some(Ok(_))` 或 `Some(Err(_))`，并原样保留外层的 `Pending` 与中层的 `Ready(None)`。
这三个嵌套层次依次回答“本次是否完成”“是否有元素”和“业务是否成功”，不能互换。

`map` 与处理 `Poll<Result<...>>` 的两种映射从 Rust 1.36 起 stable，处理 `Poll<Option<Result<...>>>` 的两种映射从 Rust 1.51 起 stable。
其余普通值能力如下：

| 表面 | 固定行为与 Rust 1.91.1 状态 |
| --- | --- |
| `is_ready(&self)`、`is_pending(&self)` | 只查询当前变体，不取值或等待；从 1.36 起 stable，当前可用于 stable const context |
| `From<T> for Poll<T>` | 把已有 `T` 包成 `Ready(T)`，不能产生 `Pending`；普通 conversion 从 1.36 起 stable，const impl 仍是 unstable `const_convert` |
| `Copy`、`Clone`、`Debug`、比较、排序与 hash impl | 在 `T` 满足相应约束时操作这个 `Poll<T>` 值，不复制、比较或驱动原 Future |
| `#[must_use]` | 对直接丢弃一个 `Poll` 发出 lint，不替调用方处理 `Pending` |

## `ready!` 与 unstable `?` 传播不同层次

下面是 stable [`ready!` macro](https://doc.rust-lang.org/1.91.1/core/task/macro.ready.html)在固定 `core/src/task/ready.rs` 中的完整 match；节选省略 attributes 与 rustdoc，中文注释由本书添加，并因依赖标准库内部上下文而标为 `ignore`：

```rust,ignore
pub macro ready($e:expr) {
    match $e {
        $crate::task::Poll::Ready(t) => t, // 取出完整 T；T 是 Result 时也不会自动传播 Err。
        $crate::task::Poll::Pending => {
            return $crate::task::Poll::Pending; // 只有外层 Pending 让当前函数提前返回。
        }
    }
}
```

如果 `$e` 是一次子 Future poll，真正发起 poll 的是这个表达式，macro 只匹配已经得到的 `Poll`。

同一 `poll.rs` 还为某些 `Poll<Result<...>>` 形状实现 `Try` 与 `FromResidual`，但固定版本将这些 impl 标为 unstable `try_trait_v2`：

| 输入 | unstable `Try::branch` |
| --- | --- |
| `Poll::Pending` | `Continue(Poll::Pending)`，不提前返回 |
| `Poll::Ready(Ok(value))` | `Continue(Poll::Ready(value))`，解开 `Ok` 但保留 `Poll` |
| `Poll::Ready(Err(error))` | `Break(Err(error))`，再由 `FromResidual` 形成外围 `Poll::Ready(Err(...))` |

`Poll<Option<Result<T, E>>>` 的 unstable impl 同样只把 `Some(Err(error))` 作为 residual 提前传播，`Pending`、`Ready(None)` 与 `Ready(Some(Ok(value)))` 都以对应的 `Poll<Option<T>>` 继续。
因此，`ready!` 提前传播外层 `Pending`，这些 `Try` impl 提前传播内层 `Err`；固定源码可以解释差异，stable 学习路径却不能把后者当作 Rust 1.91.1 已承诺的公共 API。

## `Future` 的 blanket impl 如何转发 poll

固定 [`future.rs`](https://github.com/rust-lang/rust/blob/ed61e7d7e242494fb7057f2657300d9e77bb4fcb/library/core/src/future/future.rs)除 trait 本身外还有两个 stable blanket impl。
下面保留 impl 签名、关联输出和 poll 路径，省略 attributes、rustdoc 与 imports；中文注释由本书添加，代码依赖 `core` 内部上下文，因而作为固定源码节选而不是独立示例编译：

```rust,ignore
impl<F: ?Sized + Future + Unpin> Future for &mut F {
    // `?Sized` 取消默认 Sized 要求；`Unpin` 允许从普通 `&mut F` 安全建立 Pin。
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // `&mut **self` reborrow 底层 F；`Pin::new` 依赖上面的 `F: Unpin`。
        F::poll(Pin::new(&mut **self), cx)
    }
}

impl<P> Future for Pin<P>
where
    P: ops::DerefMut<Target: Future>,
    // P 必须能可变 deref 到一个 Future；底层 Future 不需要 Unpin。
{
    type Output = <<P as ops::Deref>::Target as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // `as_deref_mut` 只把已有 pinning guarantee 短暂 reborrow 给底层 Future。
        <P::Target as Future>::poll(self.as_deref_mut(), cx)
    }
}
```

第二项使常见的 `Pin<Box<F>>` 可以直接作为 Future 使用，而 `.as_mut()` 还能取得一次传给底层 `poll` 的短期 pinning reference。
这种转发不创建新计算、不克隆底层 Future，也不改变完成后再次 poll 的契约。
它也不等同于 [`async { child.await }`](https://doc.rust-lang.org/1.91.1/reference/expressions/block-expr.html#async-blocks)：后者按语言规则产生一个新的匿名 Future，前者只是让 `Pin<P>` 沿既有 pinning guarantee 调用同一个底层 Future。
`Pin<Box<F>>` 可能因 [`Box`](https://doc.rust-lang.org/1.91.1/std/boxed/index.html) 带来 heap allocation 和 pointer indirection，但不能仅凭这段转发源码断言机器码中必然保留一次额外函数调用；表示方式、所有权、派发与实测成本将在完成 compiler lowering 和 `Pin`/`Unpin` 前置知识后作为独立机制单元研究。

## `Ready<T>` 与 `Pending<T>` 的其余公共表面

主章已经分别走读两个类型的字段与 poll，剩余表面集中如下：

| 表面 | 固定行为与边界 |
| --- | --- |
| `Debug for Ready<T>` | 仅在 `T: Debug` 时可用 |
| `Clone for Ready<T>` | 要求 `T: Clone` 并复制当前 `Option<T>`；`Some(T)` 产生另一份可独立交付的值，`None` 产生另一个已完成值 |
| `Ready::into_inner(self)` | 内部仍为 `Some(T)` 时取出 `T`，完成后的 `None` 会 panic |
| `Debug`、`Clone for Pending<T>` | `Clone` 不要求 `T: Clone`，每次都构造另一个永不完成的 `Pending<T>` |
| `PhantomData<fn() -> T>` | 对 auto traits 与 variance 的进一步影响留到标准库第二遍工程设计复盘 |
| `Ready<T>`、`Pending<T>` 上的 `#[must_use]` | 对直接丢弃尚未驱动的最小 Future 发出 lint |

固定源码中的 lang item、diagnostic attribute 和 doc-hidden compiler bridge 不是普通库调用入口，它们将在 compiler lowering 单元结合固定 HIR、THIR 与 MIR 回收。

## 不同证据回答不同问题

| 证据类别 | 能证明什么，以及不能外推到哪里 |
| --- | --- |
| Rust 1.91.1 Reference 与 stable `Future`、`Poll`、`Context` API | 规定语言可观察语义、通用 borrow 语法和公共 poll 契约；不规定具体 compiler lowering、任意手写构造行为或统一内部状态机 |
| Rust commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 的 `core::pin`、`future.rs`、`poll.rs`、`ready.rs` 与 `pending.rs` | 证明固定实现、trait impl、逐项 stability 和本章所需的最小 Pin 模型；不能升级为所有版本、所有 Future 或完整 pinning soundness 证明，源码存在本身也不代表 stable |
| [RFC 2349](https://rust-lang.github.io/rfcs/2349-pin.html)、[RFC 2592](https://rust-lang.github.io/rfcs/2592-futures.html)、[rust-lang/rust #59113](https://github.com/rust-lang/rust/issues/59113) 与 [#59119](https://github.com/rust-lang/rust/pull/59119) | 说明 pinned receiver、错误分层与可变 `Context` 的设计历史，不能替代当前 Reference、API 和 stability |
| [`future-poll` 实验关卡](future-poll-lab.md) | 观察 async Future、`ControlledFuture`、`RepeatReady` 与标准库 `Ready<T>`；不能代表未测试的 Future |

## 相邻异步表面的后续路线

| 相邻表面 | 已命名路线 |
| --- | --- |
| `IntoFuture` | [总体路径](system-boundaries.md)与后续 async/await 语言语义 |
| `Context`、Waker 系列与 `Wake` | 下一机制单元 |
| `poll_fn` / `PollFn` | 手写 Future 工具切片 |
| `Pin` / `Unpin` | `PINNING` |
| unstable `join`、`AsyncDrop` | 组合机制与取消语义 |
| stable [`AsyncFn`、`AsyncFnMut`、`AsyncFnOnce`](https://github.com/rust-lang/rust/blob/ed61e7d7e242494fb7057f2657300d9e77bb4fcb/library/core/src/ops/async_function.rs) | async closure 与 `ASYNC-TRAITS` |
| unstable [`AsyncIterator`](https://github.com/rust-lang/rust/blob/ed61e7d7e242494fb7057f2657300d9e77bb4fcb/library/core/src/async_iter/async_iter.rs) | 评估 `tiny-stream` 前的独立机制单元 |
| coroutine 与 async generator 的 compiler-only bridge | 固定 compiler lowering 走读 |
