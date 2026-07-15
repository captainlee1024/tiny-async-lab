# Designing futures for Rust

## 元数据

- 目录 ID：`AT-FUTURES-DESIGN`
- 作者或讲者：Aaron Turon
- 形式：文章
- 标题：Designing futures for Rust
- 发布日期：2016-09-07
- 访问日期：2026-07-16
- URL：[原文](https://aturon.github.io/blog/2016/09/07/futures-design/)
- 版本定位：当前网页版本
- 关联主题：`ASYNC-MODEL`
- 主张状态：设计动机已由 RFC 2592 核验；历史实现细节仅作背景

## 问题与范围

本次只回答 Rust Future 为什么从完成回调转向由外部主动 poll，以及 task 为什么成为 Pending 与下一次 poll 之间的稳定连接。
取消语义、`Pin`、现代 executor 的具体表示和性能结论不在本次范围内。

## 来源主张

文章报告，完成回调模型在 `join` 等 Future 组合处需要共享 continuation，并在异构事件源处引入分配与动态派发。
poll 模型把控制权交给外部执行者，使组合 Future 可以把子 Future 与中间结果保存在自身状态中。
这种反转随即产生“`Pending` 后由谁、在何时再次 poll”的问题，task 作为稳定执行单元把事件通知连接回根 Future。

## 定义与前提

- 文中的 `Async::NotReady`、`park`/`unpark` 和 futures 0.1 API 是历史接口，不能直接替换 Rust 1.91.1 的 `Poll::Pending` 与 `Waker` 契约。
- “readiness-based”描述 Future 协议的历史视角，不限制底层只能使用 readiness I/O；完成事件同样可以转成一次可重新 poll 的通知。
- 文章比较的是当时探索过的 Rust API 方案，不证明任意 callback 实现都必然慢于任意 poll 实现。

## 方案与取舍

| 方案 | 解决的问题 | 依赖的约束 | 主要代价 |
| --- | --- | --- | --- |
| 完成回调 | 操作完成时主动交付结果 | continuation 必须能被事件源持有和调用 | Future 组合需要共享与异构回调，Rust 中常引出分配、同步和动态派发 |
| 外部 poll + task | 调用者按需推进，事件只通知可再次推进 | Future 保存组合状态，task 提供稳定唤醒目标 | 必须额外定义 Pending、唤醒协议和 executor |

## 当前状态核验

| 结论类型 | 核验来源 | 精确位置 | 结果 |
| --- | --- | --- | --- |
| 设计状态 | [RFC 2592](https://rust-lang.github.io/rfcs/2592-futures.html) | “Rationale, drawbacks, and alternatives” | 已接受 poll/task 模型，并明确记录 callback 方案的分配、取消和组合代价 |
| 公共契约 | Rust 1.91.1 `core::future::Future` | `Future::poll` 文档 | 当前接口使用 `Poll`、`Context` 与最新 `Waker`，历史 API 名称不得沿用 |
| 当前实现 | Rust commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` | `library/core/src/future/future.rs::Future` | poll 协议位于 `core`，executor 不在标准库中定义 |

## 结论

### 已确认

- Rust 采用 poll/task 模型是一次明确的控制权反转，用 Future 自身状态承载组合，用 Waker 把外部进展连接回 task。
- 该模型规定异步计算的互操作协议，但不规定 executor、I/O 模型或调度策略。

### 仅说明历史或设计动机

- callback 原型的具体实现经历和 futures 0.1 的 API 形态。

### 开放问题

- 编译器生成的 async Future 如何把嵌套 `.await` 编译成可恢复状态机，需要固定编译器输出与实验单独核验。

## 后续去向

- 正式说明：`docs/src/rust-async/system-boundaries.md`
- 可运行证据：后续 `Future`/`Poll` 最小实验
- 复查触发条件：Future 公共协议或相关 RFC 状态变化
