# 实验关卡：手动观察 Future 与 Poll

上一章已经建立从 Future 被主动 poll、返回 `Pending`、收到 wake 通知到再次 poll 的总体路径。
在继续拆解 `Future::poll` 的公共契约和标准库实现之前，先用一个只依赖标准库的受控实验观察这条路径，避免只记住术语而没有亲手追踪状态变化。

本页是阅读入口，不提前给出实验中每一处代码的解释。
先运行并阅读观察版，再用自己的语言回答理解检查；完成读者 review 和讨论后，后续 PR 才会加入解释版注释与第二个固定版本入口。

## 观察版

观察版固定在 commit [`39c7969231ac1ae24d1bf64fb30419633bcb6875`](https://github.com/captainlee1024/tiny-async-lab/commit/39c7969231ac1ae24d1bf64fb30419633bcb6875)。
先阅读该版本的 [`labs/future-poll/src/lib.rs`](https://github.com/captainlee1024/tiny-async-lab/blob/39c7969231ac1ae24d1bf64fb30419633bcb6875/labs/future-poll/src/lib.rs) 和 [实验说明](https://github.com/captainlee1024/tiny-async-lab/blob/39c7969231ac1ae24d1bf64fb30419633bcb6875/labs/future-poll/README.md)。
这个版本保留了识别公共观察结果、字段角色和运行边界所需的注释，但没有写入读者走读后形成的详细解释。

在仓库根目录运行：

```text
cargo run -p future-poll
cargo test -p future-poll
```

如果当前工作树中的 lab 已经演进到解释版，可以不切换分支，直接查看固定观察版：

```text
git show 39c7969231ac1ae24d1bf64fb30419633bcb6875:labs/future-poll/src/lib.rs
```

## 阅读顺序

先从 `run_experiment` 追踪每次手动 poll 使用了哪个 `Context`，并把输出中的观察点对应回调用顺序。
再阅读 `controlled_future`、`ControlledFuture::poll`、`Completion::complete` 和 `State`，画出当前实验允许的状态转换。
最后阅读 `WakeCounter` 与测试，区分实验真正观察到的行为、测试驱动主动执行的动作，以及真实 executor 尚未出现的职责。

第一遍阅读时不要急着评价 `Rc<RefCell<_>>`、`mem::replace` 或手动 poll 是否适合生产运行时。
先回答它们在这个受控实验中分别隔离了什么问题，再把真实线程同步、任务入队和底层 Waker 实现留给后续源码单元。

## 理解检查

完成本关卡不要求已经理解 `RawWaker`、多线程调度、I/O reactor 或编译器生成的状态机，但应当能够用自己的语言回答：

1. `ControlledFuture` 与 `Completion` 分别持有哪些共享状态，谁负责观察状态，谁负责改变状态？
2. `poll` 返回 `Pending` 时保存了什么，为什么连续两次 poll 后只应通知最近登记的 Waker？
3. `Completion::complete` 为什么先发布 `Ready(output)`，再调用 `wake()`；这两个动作分别承担什么责任？
4. 为什么通用的 `wake()` 只表示 task 值得再次 poll，而不能保证下一次 poll 一定返回 `Ready`；当前实验又为什么在 `complete` 后确定返回 `Ready`？
5. `Poll::Ready(output)` 对调用方意味着什么，当前 `ControlledFuture` 又如何观察错误的完成后再次 poll？

指定读者能够解释这些关系、指出实验没有实现的 executor 入队动作，并把通用契约与这个 Future 的具体行为分开后，本关卡才进入解释版回写。

## 两个版本如何对照

| 版本 | 固定入口 | 用途 |
| --- | --- | --- |
| 观察版 | [`39c7969`](https://github.com/captainlee1024/tiny-async-lab/tree/39c7969231ac1ae24d1bf64fb30419633bcb6875/labs/future-poll) | 先运行、阅读、查资料并形成自己的解释 |
| 解释版 | 完成读者 review 后由下一 PR 补入 | 对照讨论后重新组织的详细注释、扩展边界与源码入口 |

观察版所在 PR 必须通过 merge commit 合入，使这里引用的 commit 成为 `master` 历史的一部分，而不是只存在于可能删除的临时分支。
解释版采用相同合并方式，并且在 book commit 记录其 permalink 后不再改写对应代码 commit。

解释版不会改变本实验的目标，也不会把它扩展成 executor。
它会把已经核验的状态转换、Waker 登记、发布与通知顺序、完成边界和简化条件写回代码，并提供与观察版的固定 diff。

## 后续标准库源码路径

实验只提供可观察行为，不能替代标准库契约和实现。
完成本关卡后，`Future`/`Poll` 机制单元将继续走读项目固定的 Rust 1.91.1、commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 中以下入口：

- `library/core/src/future/future.rs::Future::poll`：调用者、实现者与最近 Waker 的公共责任；
- `library/core/src/task/poll.rs::Poll`：一次 poll 是否已经交付最终输出；
- `library/core/src/future/ready.rs::Ready<T>::poll`：一个具体的立即完成 Future 如何保存并取出输出；
- `library/core/src/future/pending.rs::Pending<T>::poll`：永不取得进展的具体 Future 为什么可以只返回 `Pending`。

`Context`、`Waker`、`RawWaker` 与 `Wake` 的完整生命周期、身份和安全不变量属于紧随其后的唤醒协议单元。
ready queue、park 和 `wake` 如何真正导致 task 再次获得 poll，则由再下一步的标准库最小 executor 实验闭合。
