# Future 与 Poll 手动实验

本实验只使用标准库，直接调用 `Future::poll` 观察惰性 Future、`Pending`、最近 Waker、外部进展通知和重新 poll 之间的关系。
它不是 executor，也不解释 `RawWaker`、线程间调度、park、取消或 `Pin` 的完整安全模型。

## 可观察路径

实验包含两条互相补充的路径：

1. 调用一个 `async fn` 后先检查函数体尚未执行，再手动 poll 并观察函数体开始执行和 `Ready` 输出。
2. 连续使用两个不同 Waker poll 一个受测试控制的 Future，观察两次 `Pending`、最近 Waker 替换、完成信号只唤醒最近 Waker，以及重新 poll 后得到 `Ready`。

第二次手动 poll 只用于验证最近 Waker 契约，不表示 executor 应在 `Pending` 后忙循环；正常调用方应等待进展通知。
单元测试还比较两个具体 Future 在完成后再次 poll 的不同表现。
这个比较只证明具体实现可以返回不同结果或 panic，不把任何一种行为提升为 `Future` trait 的通用保证。

## 运行

在仓库根目录执行：

```text
cargo run -p future-poll
cargo test -p future-poll
```

`cargo run` 打印每个观察点，`cargo test` 把相同观察固定为可重复检查。

也可以把实验作为库调用：

```rust
use std::task::Poll;

let observations = future_poll::run_experiment();

assert_eq!(observations.body_runs_before_first_poll, 0);
assert_eq!(observations.body_runs_after_first_poll, 1);
assert_eq!(observations.controlled_first_poll, Poll::Pending);
assert_eq!(observations.stale_waker_wakes, 0);
assert_eq!(observations.latest_waker_wakes, 1);
assert_eq!(observations.controlled_completion_poll, Poll::Ready("完成"));
```

## 证据边界

调用 `async fn` 的行为来自 [Rust 1.91.1 Reference](https://doc.rust-lang.org/1.91.1/reference/items/functions.html#async-functions)，`Future::poll` 与 `Poll` 的责任来自同版本的 [`Future` API](https://doc.rust-lang.org/1.91.1/core/future/trait.Future.html) 和 [`Poll` API](https://doc.rust-lang.org/1.91.1/core/task/enum.Poll.html)。
完成后再次 poll 标准库 `Ready<T>` 时发生 panic 是固定 commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 中 `library/core/src/future/ready.rs::Ready<T>::poll` 的实现事实，不是 trait 保证。
实验只能证明这里的显式 Future 和测试驱动如何遵守这些契约，不能证明所有 Future 的内部状态或完成后行为都相同。
