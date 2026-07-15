# Why async Rust?

## 元数据

- 目录 ID：`WB-ASYNC-ORIGIN`
- 作者或讲者：Without Boats
- 形式：文章
- 标题：Why async Rust?
- 发布日期：2023-10-15
- 访问日期：2026-07-16
- URL：[原文](https://without.boats/blog/why-async-rust/)
- 版本定位：当前网页版本
- 关联主题：`ASYNC-MODEL`
- 主张状态：部分核验；只把已由 RFC 和现行契约支持的内容写入正式说明

## 问题与范围

本次只回答 Rust 为什么选择由 Future 保存暂停状态的 stackless async 模型，以及这项选择与 Rust 的运行时边界和可组合状态机有什么关系。
文章对 green thread、FFI 和未来语言方向的完整论证不在本次范围内。

## 来源主张

文章把 Rust async 描述为 stackless coroutine：编译器生成的 Future 保存暂停时需要的状态，恢复时不需要保留一套独立调用栈。
作者将 Future 的 poll 模型与 Rust 的外部迭代器类比，强调较小状态机可以组合成一个由调用者主动推进的状态机。
文章回顾了内置 green-thread runtime 的栈管理、FFI 和全局运行时约束，并把 library-based async 视为更符合 Rust 系统语言定位的取舍。

## 定义与前提

- stackless 说明暂停状态不由每个 coroutine 的独立调用栈保存，不意味着 Future 没有内存成本或运行时开销。
- “编译成状态机”是解释编译器生成 Future 的语义模型，不是对 enum 形态、字段顺序或内存布局的保证。
- 文章是设计者回顾，可说明作者的历史解释和价值排序，但不能单独证明 Rust 1.91.1 的公共契约。

## 方案与取舍

| 方案 | 解决的问题 | 依赖的约束 | 主要代价 |
| --- | --- | --- | --- |
| 内置 stackful green thread | 用近似同步的调用栈表达大量并发工作 | 语言与 runtime 共同管理可增长栈、切换和 FFI | 全局运行时选择与栈管理会影响所有程序和外部互操作 |
| library-based stackless Future | 把暂停状态放入值，由库选择执行与 I/O 策略 | 需要 `async`/`.await` 转换、poll/wake 协议与显式 async 边界 | 函数着色、协作式让出和 runtime 生态复杂度 |

## 当前状态核验

| 结论类型 | 核验来源 | 精确位置 | 结果 |
| --- | --- | --- | --- |
| 设计状态 | [RFC 2394](https://rust-lang.github.io/rfcs/2394-async_await.html) | “Motivation”与“Prior art” | 核验内置 green-thread runtime 因过于强制而移除，以及 library-based async 的取向 |
| 公共契约 | Rust 1.91.1 Reference | async function 与 await expression | 核验调用返回 Future、函数体延迟到 poll、`.await` 轮询子 Future |
| 当前实现 | Rust commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` | `library/core/src/future/future.rs::Future` | 核验 Future 是标准库协议；具体状态机布局仍不属于公共保证 |

## 结论

### 已确认

- Rust async 把可暂停计算表示为 Future，并把执行策略留给库提供的 runtime。
- Future 组合与 Rust 的值语义、借用和静态派发可以协同，但具体成本必须由实现与测量证明。

### 仅说明历史或设计动机

- green thread 各种栈方案的具体演进与作者对 Rust 系统语言定位的评价。

### 开放问题

- `Pin` 如何使包含跨暂停点借用的 Future 安全，以及未来 `Move` 设计会改变哪些边界。

## 后续去向

- 正式说明：`docs/src/rust-async/system-boundaries.md`
- 可运行证据：后续编译器 lowering 与 Future 布局实验
- 需要继续阅读的目录 ID：`WB-PIN`、`NM-MINPIN`
- 复查触发条件：相关语言能力或官方 async roadmap 变化
