# 建立 Future、Poll 与唤醒协议基础

- 状态：active
- 最近更新：2026-08-25
- 关联路线图：`ROADMAP.md` 阶段 0 的恢复演练，以及阶段 1 的异步系统边界、`Future`/`Poll`、唤醒协议和最小 executor
- 关联 PR：里程碑 1 的 [#13](https://github.com/captainlee1024/tiny-async-lab/pull/13) 已通过 merge commit `43fb7b0e901ae8e2b8502ce7d5a6ff77d0519be1` 合入；里程碑 2 的观察版 [#14](https://github.com/captainlee1024/tiny-async-lab/pull/14) 已通过 merge commit `5c58b6fde6e66e6690f38f2226f8e3a7a32031a8` 合入，解释版 [#15](https://github.com/captainlee1024/tiny-async-lab/pull/15) 已通过 merge commit `8862f5df8f1adf9386c12e856d26e2ddda73e3ef` 合入；Future/Poll 契约正文 PR 尚未创建

## 目标

用固定 Rust 版本的公开契约、标准库源码和聚焦实验，建立从 `async` 产生 Future 到 executor 根据唤醒重新 poll 的完整基础模型。
任务完成后，没有原对话的新 Agent 应能从本计划和仓库事实继续工作，读者应能区分 Future、task、executor、scheduler、reactor 和 runtime，并解释 poll/park/wake 闭环的关键不变量。

## 本次不做

- 不研究编译器实际生成的 HIR、THIR、MIR 和状态机布局。
- 不在本里程碑完成 `Pin` 的安全证明、projection、drop guarantee、取消、`Send`/`Sync` 或 async trait；但 `Future::poll` 签名首次承担推理时，必须解释 `Pin<&mut Self>` 的最小操作契约，并把完整证明路由到 `ROADMAP.md` 阶段 1 与 `research/TOPICS.md` 的 `PINNING`。
- 不研究 Tokio/Mio，也不实现生产可用运行时、异步 I/O 或 timer。
- 不提前为后续 `tiny-runtime` 设计公共抽象。

## 上下文地图

- `AGENTS.md` — 证据、源码研究、变更范围和恢复约束。
- `ROADMAP.md` — 当前阶段、机制依赖与完成状态。
- `upstream/BASELINES.md` — Rust 1.91.1、commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 和 `rust-src` 入口。
- `docs/source-reading-method.md` — 每个机制单元的理解闭环和两遍源码阅读方法。
- `docs/src/SUMMARY.md` — 学习书的实际阅读顺序。
- `research/TOPICS.md` 中的 `ASYNC-MODEL` — Future 模型设计原因的研究问题和候选资料。

## 验收条件

- [x] 学习书用公开契约和固定源码划清语言、编译器、标准库与运行时的职责，并给出可复查的源码入口地图。
- [x] `Future`/`Poll` 章节闭合首次 poll、`Pending`、`Ready`、重复 poll 和最新 Waker 契约，并由最小手动 poll 实验验证可观察行为。
- [ ] `Context`、`Waker`、`RawWaker` 与 `Wake` 章节闭合进度、生命周期和安全不变量，并由聚焦实验连接 wake 与重新 poll。
- [ ] 标准库最小 executor 实验闭合 poll、park、wake 和退出路径，同时明确它不是生产运行时。
- [ ] 新 Codex 对话只依靠已跟踪仓库文件完成一次冷启动恢复核验；任务结束前再完成一次干净 clone 的跨机器恢复核验。
- [ ] 每个 PR 运行 `make ci`，长期结论进入正式文档，路线图只在对应结果完成后更新。

## 里程碑

### 1. 异步系统职责与源码边界

在 `docs/src/` 建立第一章，解释调用、poll、wake 的总体路径，区分各层职责，并定位固定 `rust-src` 中的稳定入口。
本里程碑不展开 `Future::poll` 的完整契约和编译器 lowering。

#### 异步系统边界内部写作设计

本节只约束研究、写作和评审，不作为学习书的固定结构。

- 主问题：当异步操作当前不能完成时，Rust 如何在不阻塞执行线程的情况下保存计算，并在进展条件改变后继续；语言、编译器、标准库和 runtime 分别负责哪一段？
- 因果链：等待型工作负载 → 保存可恢复状态 → `Future::poll` 主动推进 → `Pending` 登记进展通知 → `Waker` 使 task 再次可调度 → executor 重新 poll；`.await` 在同一条 poll 链中组合子 Future，runtime 负责协议之外的任务与资源策略。

| 结论类型 | 主要证据 | 本里程碑边界 |
| --- | --- | --- |
| 当前语言语义 | Rust 1.91.1 Reference | 说明调用与 `.await` 的可观察语义，不推断具体 MIR 或内存布局 |
| 公共协议 | Rust 1.91.1 `Future`、`Waker` API 文档 | 只引入闭合总体路径所需的契约，完整状态和安全边界留给后续里程碑 |
| 固定实现 | commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 的 `rust-src` | 证明模块位置、re-export 和 stability，不把实现位置提升为语言保证 |
| 设计原因 | RFC 2394、RFC 2592 与已路由的设计者回顾 | 区分已接受设计、历史解释和当前保证，不用博客证明现行契约 |

正文完成后应能仅依靠文章回答以下问题：

1. 为什么调用 `async fn` 不会执行函数体，谁最终使它获得进展？
2. Future、task 与 OS thread 分别是什么，为什么 `.await` 不等于 `spawn`？
3. `Pending` 之后为什么不需要忙循环，`Waker::wake` 又为什么不等于立即从 `.await` 后继续执行？
4. 为什么 `Future` 与唤醒协议进入标准库，而 executor、scheduler 和 I/O driver 仍由 runtime 提供？

### 2. Future 与 Poll 契约

从 Rust Reference、标准库 API、固定源码和最小手动 poll 实验核验 Future 的状态与调用边界，并同步阅读 `ASYNC-MODEL` 路由的设计资料。

#### Future/Poll 内部写作设计

本节只约束研究、实验、写作和评审，不作为学习书的固定结构。

- 主问题：`Future::poll` 签名中的每一层约束如何共同建立一次合法调用，调用者和 Future 实现分别承诺什么；首次 poll、`Pending`、`Ready` 和完成后再次 poll 的边界如何组成一个不会把具体实现行为误写成通用保证的状态模型？
- 因果链：Future 是需要跨调用保存状态的值 → `&mut Self` 提供本次独占可变访问 → 某些 Future 可能在 poll 后依赖地址关系，因此 `Pin` 限制安全移动 → `Context<'_>` 借用当前 task 的唤醒能力 → 外部主动 poll → Future 尝试推进 → `Pending` 保留未完成状态并安排进展通知 → 条件改变后调用者重新 poll → `Ready` 交付 `Self::Output` 并结束本次计算；完成后再次 poll 的允许行为必须回到公共契约，而不能从单个实验外推。

| 结论类型 | 主要证据 | 本里程碑边界 |
| --- | --- | --- |
| 当前语言语义 | Rust 1.91.1 Reference 中的 async function 与 await expression | 只证明编译器生成 Future 的调用、首次执行和 `.await` 传播语义，不替任意手写 Future 补充 trait 契约 |
| 公共协议 | Rust 1.91.1 `Future::poll`、`Poll`、`Pin`、`Context` 与 lifetime elision 的 API/Reference 文档 | 逐层解释 receiver、借用、可变性、地址稳定性、内部与外部 lifetime 以及关联输出；完整 Pin 安全证明与 Waker 身份、安全表示仍分别留给 `PINNING` 和里程碑 3 |
| 固定实现 | commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 的 `core::future` 与 `core::task` | 复查 doc comment、stability 和标准库聚焦实现，不把某个 Future 的完成后行为提升为 trait 保证 |
| 可观察行为 | 标准库手动 poll 实验 `labs/future-poll` | 分别观察创建、首次 poll、`Pending`、外部条件改变、重新 poll 和 `Ready`；实验只证明被测 Future 与测试驱动的行为 |
| 设计原因 | RFC 2592、RFC 2349、rust-lang/rust #59113/#59119 与 `ASYNC-MODEL` 已路由资料 | 分开解释 poll 模型、pinning 和可扩展 task context 的设计取舍，不用历史讨论覆盖当前稳定契约 |

#### 主线与参考页的内容去向

本轮结构调整按问题而不是篇幅拆分，下面的去向表用于确认主线减负没有变成知识删减。

| 知识单元 | 主机制页 | 相邻参考页 |
| --- | --- | --- |
| `poll` 完整签名 | 用可编译的注释版签名逐层解释 receiver、borrow、`Pin`、三个输入 lifetime、`Context`、关联输出与安全底线，再用短正文回答代码无法证明的设计原因 | 只在证据矩阵标明 Reference、API、固定源码和历史理由各自能证明什么 |
| 单次调用与跨调用状态 | 解释一次 poll、Future/外部进展/`Poll` 三层状态、首次与后续 poll 的发起者以及 `.await` 的传播 | 不重复主机制解释 |
| `Pending`、`Ready` 与业务结果 | 保留双方进展责任、条件性 wake、完成与业务成功的分层图，以及取消与下一 Waker 单元的边界 | 在辅助 API 中复用已经建立的外层进度与内层输出模型 |
| 完成后再次 poll | 保留 trait 通用边界、实验反例与带教学注释的固定 `Ready<T>`/`Pending<T>` 核心状态转换 | 继续列出两种最小 Future 的 `Clone`、`Debug`、`Unpin`、`into_inner` 与 marker 等其余表面 |
| `Poll` 与 `Future` 的其余公共表面 | 结尾先定位它们只处理已返回值或转发 poll | 完整解释映射、查询、`From`、`ready!`、unstable `Try`/`FromResidual`、stability 和两个 blanket impl |
| 证据与相邻异步表面 | 保留直接支撑主推理的行内证据与明确参考入口 | 集中保存证据边界，以及 `IntoFuture`、Waker、`Pin`、`poll_fn`、`AsyncFn*`、`AsyncIterator` 和 compiler bridge 的后续路线 |

#### 相邻标准库表面盘点与路由

当前机制的第一层盘点以 Rust 1.91.1 的 `core::future`、`core::task`、`alloc::task` 以及签名直接依赖的 `core::pin` 为边界，并用跨模块公开项扫描补充 `core::ops`、`core::async_iter` 和 coroutine 入口，避免把目录边界误当成整个标准库异步边界。
正文是否展开一项取决于当前推理是否需要它，但下表中的每一项都必须有明确去向。

| 固定表面 | 本轮必须确认的内容 | 学习路径 |
| --- | --- | --- |
| `Future`、`Future::Output`、`Future::poll` | `Self`、显式 receiver、`Pin<&mut Self>`、`&mut Context<'_>`、`Poll<Self::Output>`、`#[must_use]` 与完成后边界 | 本章逐层解释；`Pin` 的完整 soundness 路由 `PINNING`，`Context` 的唤醒与生命周期路由里程碑 3 |
| `Future for &mut F`、`Future for Pin<P>` | 两个 stable blanket impl 如何保持 `Output` 并把 poll 转发给底层 Future，以及 `Unpin`/`DerefMut` 约束分别保护什么 | 相邻参考页解释；projection 的完整理由路由 `PINNING` |
| `Poll<T>` | `Ready`、`Pending`、`#[must_use]`、derived traits、`is_ready`、`is_pending`、`map`、两组 `map_ok`/`map_err` 与 `From<T>` | 主机制页先建立进度层与输出层，相邻参考页再用独立小节和参考表完整解释，不能在状态机段落突然点名 |
| `Poll` 的 `Try`/`FromResidual` impl | 固定源码中的 `Continue`/`Break` 分支、`Pending` 与 `Err` 的不同传播方向，以及 `try_trait_v2` stability | 相邻参考页逐分支解释并明确标为 unstable，不把 nightly 行为教成 Rust 1.91.1 stable API；后续仅在 feature 稳定或项目实际需要时重开 |
| `Ready<T>`、`ready`、`Pending<T>`、`pending` | 构造、内部状态、`Unpin`/auto traits、`into_inner`、重复 poll、是否读取 `Context` | 主机制页解释契约、核心状态与可观察差异，相邻参考页回收其余公共表面；marker 与 API 设计取舍列入阶段 1 标准库第二遍复盘 |
| `ready!` | 从子调用提取 `Ready(T)` 或提前传播 `Pending`，不负责 poll、wake 或保存状态 | 相邻参考页在 Future/Poll 机制闭合后讲解，并与 unstable `Try` impl 分开 |
| `poll_fn`、`PollFn<F>` | `FnMut(&mut Context<'_>) -> Poll<T>` 如何适配为 Future，以及 pinned closure capture 的边界 | 在本章之后建立“手写 Future 的标准库工具”切片，读取其 unsafe projection 前先完成本章签名与 `PINNING` 的必要前置 |
| `IntoFuture` | `.await` 先调用 `into_future`，所有 Future 的 identity blanket impl | 复用 `system-boundaries.md` 已有入口，并在阶段 1 async/await 语言语义里结合 Reference 完整回收 |
| `Context`、`Waker`、`RawWaker`、`RawWakerVTable`、`Wake` | 当前 task context、身份、clone/wake、合并、所有权与 vtable safety | 里程碑 3；本章只解释读懂 poll 签名和 `Pending` 责任所需的最小部分 |
| `LocalWaker`、`ContextBuilder`、`Context::ext` | 固定源码存在但 Rust 1.91.1 仍 unstable，并显示 `Context` 的扩展方向 | 里程碑 3 逐项核对 stability，不用它们反向定义 stable `Context` 当前保证 |
| `AsyncDrop`、`async_drop_in_place` | `async_drop` feature 与 drop/cancellation 关系 | `ROADMAP.md` 阶段 1 取消、drop 与资源释放机制单元 |
| `join` | `future_join` feature、多个 Future 的组合与 Pin projection | 阶段 1 async/await 组合机制和标准库第二遍复盘；稳定学习路径不把它当作可用 API |
| `ResumeTy`、`get_context` 与相关 lang items | compiler-only、doc-hidden、`gen_future` 标记及 raw pointer bridge | 阶段 1 固定 nightly 的 HIR/THIR/MIR lowering 单元，不从它们推断当前 trait 的公共保证 |
| `Pin`、`Unpin`、`pin!` 及核心访问方法 | pinning pointer 与 pointee、普通 `&mut` 仍可 move-out、`Unpin` 何时解除限制、safe/unsafe 访问边界 | 本章给出 `Future::poll` 所需最小模型；RFC 2349、固定 `core::pin` 与实验由 `PINNING` 完整证明 |
| `AsyncFn`、`AsyncFnMut`、`AsyncFnOnce` | 三个 trait 自 Rust 1.85 stable 且进入 prelude，但直接 call methods 与关联 Future types 在 1.91.1 仍由 `async_fn_traits` 标为 unstable | 随 async closure 语言语义和 `research/TOPICS.md::ASYNC-TRAITS` 核验调用方式、closure borrow、返回 Future 与 dyn/`Send` 边界 |
| `core::async_iter` | `AsyncIterator`、`IntoAsyncIterator`、`FromIter`、`from_iter` 与 compiler helpers 在 1.91.1 均 unstable，并复用 `Pin<&mut Self>`、`Context` 与 `Poll<Option<Item>>` | 在评估 `ROADMAP.md` 阶段 5 的 `tiny-stream` 前建立独立异步迭代机制单元，不把 ecosystem `Stream` 契约冒充标准库 stable API |
| `Coroutine`、`CoroutineState`、`from_coroutine` 与 async generator helpers | 固定源码中的 unstable coroutine 表面和 compiler desugaring bridge | 阶段 1 固定 nightly lowering 单元，先区分语言可观察语义、unstable library surface 与具体 compiler representation |

首个实验切片使用显式状态和测试控制的进展条件，保证返回 `Pending` 的路径登记当前 Waker，并由测试在条件改变时触发通知。
实验先闭合单 Future 的调用与状态观察，不实现 ready queue、park、跨线程调度或 `RawWaker`，这些职责分别留给里程碑 3 和 4。

#### 实验关卡与读者回写

观察版固定为 commit `39c7969231ac1ae24d1bf64fb30419633bcb6875`，相邻书页 `docs/src/rust-async/future-poll-lab.md` 只负责在正确位置把读者路由到固定 lab 版本。
观察版已随 PR #14 合入，指定读者随后完成逐段走读、复述和纠偏。
解释版固定为 commit `6fb9ee38aedb9e6d1ee6d9de9426ea4827b9ade1`，只向 `labs/future-poll/src/lib.rs` 增加讨论后核验的教学注释，不改变实验行为；相邻书页用独立 book commit 记录两个版本与注释差异。

解释版允许采用面向教学的高密度注释，完整保存已经核验的状态转换、责任分工、顺序原因、容易混淆的直觉、真实实现差异和后续源码入口，而不是只留下简短标签。
它仍只解释当前实验及其边界；跨段的公共契约留在书中，未经固定契约或源码核验的讨论不进入注释。

本关卡的“理解足够”以本里程碑已有五个理解问题和指定读者 review 为准，这些内部检查不复制到学习书入口，只验收 Future/Poll 实验边界。
`Future::poll`、`Poll`、`Ready<T>` 和 `Pending<T>` 的固定标准库源码必须在里程碑 2 完成前逐个走读；Waker 系列源码和真实重新入队分别由里程碑 3、4 回收。

正文和实验完成后应能仅依靠它们回答以下问题：

1. 能否从内向外解释 `self: Pin<&mut Self>`、`cx: &mut Context<'_>` 和 `Poll<Self::Output>`，包括两个 `&mut` 各自约束谁、`Pin` 固定什么、`'_` 代表哪一层借用以及哪些完整理由仍被路由到后续？
2. 调用 `async fn`、首次 poll、`Pending` 后条件改变和重新 poll 分别由谁触发，Future 实现方与调用方各自承担什么进展责任？
3. 为什么 `Poll` 不是 Future 的内部状态机，`map`、`map_ok`、`map_err`、状态查询与 `From<T>` 分别转换哪一层并保留哪些分支？
4. `Ready` 与业务成功为什么是两层问题，完成后错误地再次 poll 时公共契约保证什么，`Ready<T>` 与实验 Future 的差异又只能证明什么？
5. 哪些结论来自 Reference、公共 API、固定源码、聚焦实验或历史资料，相邻 stable、unstable 与 compiler-only 表面分别被安排到哪里，为什么这些证据和路线不能互相替代？

#### 契约核验矩阵

| 调用点 | 调用方可以依赖与必须承担的责任 | Future 实现方的责任与允许行为 | 证据与边界 |
| --- | --- | --- | --- |
| 调用 `async fn` | 得到捕获参数的 Future，调用本身不执行函数体；外部必须在之后 poll 它才会开始执行 | 编译器生成的 Future 在获得 poll 时执行函数体 | Rust 1.91.1 Reference [`items.fn.async.future`](https://doc.rust-lang.org/1.91.1/reference/items/functions.html#async-functions)；只约束 async function，不证明任意 Future 构造函数都没有同步工作 |
| 完成前调用 `poll` | 使用 pinned mutable receiver 和当前 `Context` 发起一次完成尝试；Future 不会因作为一个值存在而自行获得 poll | 尝试推进计算并返回 `Poll`；实现应快速返回且不应阻塞，但 API 文档中的 runtime characteristics 不是完成时限保证 | Rust 1.91.1 [`Future::poll`](https://doc.rust-lang.org/1.91.1/core/future/trait.Future.html#tymethod.poll) 与固定源码 `library/core/src/future/future.rs::Future::poll` |
| 返回 `Pending` | 只能得出本次尚未产生输出；保留兴趣的调用方应等待进展通知后再次 poll，下一次仍可能是 `Pending` | 当未来可能取得进展时，必须安排当前 task 被唤醒；多次 poll 后只应让最近一次 `Context` 中的 Waker 收到后续通知 | Rust 1.91.1 [`Poll::Pending`](https://doc.rust-lang.org/1.91.1/core/task/enum.Poll.html#variant.Pending) 与 `Future::poll`；Waker 身份、合并和生命周期的完整解释留给里程碑 3 |
| `.await` 遇到 `Pending` | 外围 async context 把 `Pending` 返回给自己的 poll 调用方，待外围 Future 再次被 poll 后继续 | 被等待的 Future 仍只遵守普通 `Future::poll` 契约，`.await` 不创建独立 task | Rust 1.91.1 Reference [`expr.await.effects`](https://doc.rust-lang.org/1.91.1/reference/expressions/await-expr.html)；近似脱糖不是具体 compiler lowering |
| 返回 `Ready(output)` | 取得本次计算的最终输出，并停止 poll 这个 Future | 交付 `Output`，此后无需再支持正常 poll 路径 | Rust 1.91.1 `Future::poll` 与 [`Poll::Ready`](https://doc.rust-lang.org/1.91.1/core/task/enum.Poll.html#variant.Ready) |
| `Ready` 后错误地再次 poll | 调用方违反“完成后不再 poll”的契约，不能依赖返回、panic 或终止性 | trait 不约束这次调用的普通行为，可以 panic、永久阻塞或有其他结果；无论状态如何都不得产生 undefined behavior | Rust 1.91.1 `Future::poll` 的 `Panics` 小节；固定 `core::future::Ready<T>` 会 panic 只是一个实现事实 |

`Poll<T>` 是一次 `poll` 调用对“当前是否得到输出”的观察结果，不是 Future 全部内部状态的通用枚举。
固定实现 `core::future::Pending<T>` 永远不可能取得进展，因此它返回 `Pending` 而不安排 wake 并不违反“能够取得进展时唤醒”的条件；这个实现不能证明普通等待型 Future 可以遗漏通知。

#### 固定标准库源码第一遍走读记录

本次走读固定在 Rust `1.91.1`、commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 的本地 `rust-src`，目标四个文件没有 target 条件编译或 unsafe 实现，Waker 的表示和安全边界不在本次读取范围内。

| 固定单元 | 状态、控制流与进度责任 | 失败行为与证据边界 |
| --- | --- | --- |
| `library/core/src/future/future.rs::Future::poll` | trait 只用 `Pin<&mut Self>`、当前 `Context` 和 `Poll<Output>` 规定一次推进尝试，不暴露通用内部状态；返回 `Pending` 的可进展 Future 必须安排当前 task 在可以继续时被唤醒，多次 poll 后只应通知最近一次 `Context` 的 Waker；`&mut F` 与 `Pin<P>` 的 blanket impl 只把调用转发给底层 Future | 返回 `Ready` 后调用方应停止 poll；错误地再次 poll 可以 panic、永久阻塞或产生其他普通问题，但安全方法在任何状态都不得导致 undefined behavior；“Future 是 inert”只说明 Future 自身仍需 poll 才能交付或推进，源码同时明确其他 task 中的底层计算可以独立继续 |
| `library/core/src/task/poll.rs::Poll` | `Ready(T)` 与 `Pending` 只表达本次调用有没有取得值，enum 自己不保存 Future 生命周期或状态转换；`map`、`map_ok` 和 `map_err` 只转换匹配的 `Ready` 内容并原样传播 `Pending` | `Poll` 没有独立失败分支，错误属于 `T` 的结构，例如 `Poll<Result<T, E>>`；enum 和常用映射 API 已稳定，但同文件的 `Try` 与 `FromResidual` impl 仍由 `try_trait_v2` 标为 unstable，源码存在不能替代逐项 stability 证据 |
| `library/core/src/future/ready.rs::Ready<T>::poll` | `ready(t)` 构造 `Ready(Some(t))`，第一次 poll 用 `Option::take` 完成 `Some(T) -> None` 并返回 `Ready(T)`；实现显式为 `Unpin`，且立即完成路径不读取 `Context`、不登记 wake | 完成后再次 poll 会因 `expect` panic，完成后调用 `into_inner` 也会 panic；这是 `Ready<T>` 的一次性交付实现事实，不能提升为所有 Future 的重复 poll 保证 |
| `library/core/src/future/pending.rs::Pending<T>::poll` | 类型只保存 `PhantomData<fn() -> T>`，poll 不修改自身、不读取 `Context`，每次都返回 `Pending`，因此没有从未完成到完成的状态转换 | 它永远不可能取得进展，所以不登记 Waker 仍满足 `Pending` 的条件化通知责任；这个特例不能证明能够进展的 Future 可以返回 `Pending` 而遗漏通知 |

固定 `coretests` 只补充局部实现证据：`coretests/tests/task.rs::poll_const` 检查 `Poll` 状态查询可用于 const context，`coretests/tests/waker.rs::nop_waker::test_const_waker` 检查 `Ready<T>` 可在 no-op Waker 下立即完成，`coretests/tests/future.rs::_pending_impl_all_auto_traits` 以编译检查固定 `Pending<T>` 不受 `T` 的常见 auto traits 约束。
`Ready<T>` 的显式 `Unpin`、`Pending<T>` 的 marker 选择和无条件 auto traits 只记录为第二遍工程设计复查候选，不在本里程碑推断设计动机；通用 Future 的 drop、取消与清理也不由这四个单元定义，按 `ROADMAP.md` 阶段 1 和 `research/TOPICS.md` 的 `CANCELLATION` 路由后续回收。

### 3. Context 与唤醒协议

解释 Waker 身份、最新 Waker、wake 合并、资源生命周期和 `RawWaker` 安全边界，再用聚焦实验观察重新 poll。

### 4. 标准库最小 executor

只使用标准库实现教学用 executor，闭合 poll/park/wake 路径，并记录生产实现仍需处理的竞态、任务管理和关闭问题。

## Progress

- [x] `2026-07-15` — 完成里程碑 1 的第一版正文、固定源码地图和检查，但尚未通过读者评审。
- [x] `2026-07-16` — 用户评审判定第一版未形成充分的因果与教学结构；撤销完成结论，补充内部写作设计并开始整章重写。
- [x] `2026-07-16` — 新版按等待、保存状态、poll、wake、重新 poll 与职责边界的因果链完成，并通过技术评审、教学自检、图形目视检查和本地 CI。
- [x] `2026-07-16` — 用户接受新版作为当前迭代的质量基线；完成里程碑 1，并同步更新路线图与可复用写作约束。
- [x] `2026-07-16` — PR #13 合并到 `master`；全新 Codex 对话在无原对话上下文的条件下，从跟踪文件、Git、源码入口和最小检查恢复到里程碑 2，用户确认恢复报告准确；干净 clone 的跨机器核验仍待完成。
- [x] `2026-07-16` — 对照 Rust 1.91.1 版本化 Reference、API 文档和 commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 的固定 `rust-src`，完成首次 poll、`Pending`、`Ready` 和完成后再次 poll 的契约矩阵，并分开调用者责任、实现者责任与实验边界。
- [x] `2026-07-16` — 建立标准库实验 package `labs/future-poll`，用显式单线程状态和安全 `Wake` 实现观察惰性执行、最近 Waker、`Pending` 到 `Ready` 以及两种具体完成后行为；同时把 Rust 格式化、Clippy、测试、doctest 和 rustdoc 接入本地 `make ci` 与 PR workflow，并完成阶段 0 对应路线图项。
- [x] `2026-07-16` — 指定读者完成观察版首次走读并复述资源状态、Waker 通知和重新 poll 的关系；讨论纠正了“wake 负责把 Future 改成 Ready”和“wake 后必然 Ready”两处边界，并确认当前实验在 `complete` 后具有更强的确定行为。
- [x] `2026-07-16` — 决定把观察版、读者复述、讨论纠偏和解释版回写形成两个 PR 的实验关卡；当前 PR 增加书中明确入口与项目方法约束，解释版注释留到观察版合入后的下一 PR。
- [x] `2026-07-16` — 用户 review 指出首版实验入口把理解检查、Git 流程和源码计划堆成了第二篇文章；将其收缩为只链接观察版和 README 的短路标，并把各类信息恢复到代码、计划与正式章节。
- [x] `2026-07-16` — PR #14 通过 merge commit `5c58b6f` 合入，`master` 与 `origin/master` 同步且工作树干净；观察版 `39c7969` 是 `master` 祖先，教学检查点没有被 squash。
- [x] `2026-07-16` — 在独立分支形成解释版 commit `6fb9ee3`，只增加 73 行注释，梳理 readiness 与通知、最近 Waker、发布顺序、完成终态、task 唤醒和手动驱动边界；书页保持 7 行并增加解释版与该 commit 的差异入口。
- [x] `2026-07-16` — PR #15 通过 merge commit `8862f5d` 合入，解释版 `6fb9ee3` 成为 `master` 祖先；书页中的观察版、解释版和注释差异入口均指向已经保留的历史。
- [x] `2026-07-16` — 完成 Rust 1.91.1 固定源码第一遍走读，逐项核验 `Future::poll`、`Poll`、`Ready<T>::poll` 与 `Pending<T>::poll` 的契约落点、具体状态、进度责任、重复 poll 边界和 stability，并把 Waker 安全、取消与工程设计复盘保留在已命名的后续路径。
- [x] `2026-07-16` — 从 `research/CATALOG.md` 和 `research/TOPICS.md` 重新进入 `ASYNC-MODEL`，复核两份已摘录笔记与 RFC 2592；现有证据已经覆盖 callback 到 poll/task 的控制权反转，RFC 还明确支持错误处理与完成状态正交，因此不需要改写研究笔记或打开只作历史线索的 2020 访谈。
- [x] `2026-07-16` — 完成 `docs/src/rust-async/future-poll.md` 第一版并接入学习书，按单次调用、状态分层、首次与后续 poll、`Pending`、`Ready`、完成后误 poll 和证据边界组织；当时的技术自检与教学自检没有发现签名解释和 API 出场顺序缺陷，里程碑 2 始终未完成。
- [x] `2026-07-16` — 指定用户读到前两节即停止评审：`Pin<&mut Self>`、两个 `&mut` 与 `Context<'_>` 没有在首次出现处拆解，`Poll::map`、`map_ok`、`map_err` 又在没有定义和位置说明时突然进入状态机论证；本次读者评审推翻首稿的教学自检结论，正文恢复到待重写状态。
- [x] `2026-07-16` — 扩大到 Rust 1.91.1 的 `core::future`、`core::task`、`alloc::task` 与直接依赖的 `core::pin`，完成 stable、unstable 和 compiler-only 相邻表面盘点；同时核对 lifetime elision Reference、固定 `Pin`/`Context` 源码、RFC 2349、RFC 2592 以及 rust-lang/rust #59113/#59119，为签名拆解和后续路由建立证据。
- [x] `2026-07-16` — 对固定 `core`/`alloc`/`std` 追加跨模块公开项扫描并完整读取命中的 async callable 与 async iteration 单元，确认 stable `AsyncFn*` 位于 `core::ops` 且已进入 prelude、直接 call surface 仍 unstable，`core::async_iter` 与 coroutine 相关表面则整体仍 unstable；这些入口已经分别路由到 `ASYNC-TRAITS`、异步迭代和 compiler lowering，而没有冒充当前 Future/Poll 主线。
- [x] `2026-07-16` — 完成 Future/Poll 正文第二版：先显式展开三个输入 lifetime 并逐层解释 receiver、两个 `&mut`、`Pin`、`Context<'_>` 与 `Self::Output`，再恢复单次 poll 主推理链；`Poll` 的 stable 辅助 API 与 unstable `Try` 控制流移到进度层和业务结果层已经建立之后，固定四单元的其余公共表面集中放在参考区。
- [x] `2026-07-16` — 将本次反馈推广为工程约束：机制章节起草前必须盘点固定版本相邻表面并逐项路由，核心签名首次承担推理时必须解释所有会改变状态、所有权、可变性、地址、lifetime、进度、失败、安全或 stability 结论的语法；当前正文仍待指定用户从头评审，未恢复里程碑完成状态。
- [x] `2026-07-16` — 用户确认第二版知识内容有助于理解，但指出连续正文过重，并要求只调整承载方式而不缩减表达内容；据此先建立六类知识单元的去向表，再尝试机制页、参考页与关系图的组合。
- [x] `2026-07-16` — 完成结构第三版：主机制页从 315 个源码行调整为 240 行，相邻参考页用 111 行完整承接辅助 API、trait impl、stability、证据与后续路线；两页合计 351 行，增加的源码主要来自导航和 Mermaid 定义而不是新增推理旁支，指定用户仍需复读确认真实理解成本是否下降。
- [x] `2026-07-16` — 用户仍感觉主机制页纯文字偏多，并提出用代码注释承载局部解释；据此把“原签名、显式展开签名、借用图、三轮语法复述”收敛为一份可编译注释签名，同时保留 `Pin` 与可变 `Context` 的设计原因和证据正文。
- [x] `2026-07-16` — 完成结构第四版：用带教学注释的可编译固定源码节选替换 `Ready<T>`/`Pending<T>` 的实现复述与对照表，用责任矩阵替换 `Pending` 后的连续交接段落；主机制页为 219 行，所有既有契约、语法细节、固定实现事实和后续边界仍在内容去向表中有明确位置。
- [x] `2026-07-16` — 用户要求继续发散评估两页中的图、源码和构造示例，并授权实际尝试；核对真实 mdBook 页面后决定不为 `Pin` 再添加一张重复因果图，而是隐藏主机制页两段可编译代码中的 imports 与 assertions，并保留所有可见签名、字段和状态转换。
- [x] `2026-07-16` — 完成结构第五版：参考页按 `Poll` 值转换、`ready!`/`Try` 控制流、blanket impl 转发、其余公共表面、证据边界和后续路线重新分区；真实 `Poll::map` 调用、固定 `ready!` match 与两个固定 blanket impl 分别替换等价说明，三列长表改为两列，主机制页为 217 行、参考页为 148 行，知识单元仍逐项保留并等待指定用户复读。
- [x] `2026-07-16` — 指定用户继续指出代码之外的文字仍偏多；逐项对照后确认第五版虽然改变了载体，却没有同步删除全部旧正文，同一事实仍可能在代码注释、紧邻正文、表格、图和章尾总结中出现两到三次。
- [x] `2026-07-16` — 完成结构第六版：为每项事实指定单一主要载体，`Pin`/`Context` 正文只保留原因与证据，推进顺序交给表格，结果分层交给图，`Pending<T>`/`Ready<T>` 源码分别贴近对应机制；参考页删除 map 语义与 stability、`ready!` 源码与对照表、blanket impl 与总结句之间的重复，并把证据矩阵压回四种证据类别；主机制页为 206 行、参考页为 137 行，两页从第五版的 365 行降至 343 行而没有删除盘点中的知识单元。
- [x] `2026-08-25` — 指定用户完成结构第六版的连续复读；讨论进一步暴露“只尝试推进一次”容易被误读为只执行一个内部步骤，以及 task、根 Future、父 Future、blanket impl 转发和 async wrapper 之间仍缺少就地边界，因而在验收收尾中改为有边界的完成尝试并补入最小调用链。
- [x] `2026-08-25` — 指定用户确认两页阅读完成并要求进入 Future 表示与成本的独立章节；该问题具有独立的所有权、地址稳定性、状态机、派发与性能证据路径，但完整展开依赖 compiler lowering 和 `Pin`/`Unpin`，不混入当前 Future/Poll PR。
- [x] `2026-08-25` — Future/Poll 两页在指定用户验收后通过聚焦实验、外部链接、文档检查和完整 `make ci`；`ROADMAP.md` 阶段 1 对应条目已完成，当前切片进入 PR-ready 状态。

## Surprises and Discoveries

- 已验证：Rust 1.91.1 的 `rust-src` 同时包含 stable 与 unstable 异步 API；源码存在不能证明 API 已稳定，正式章节必须以 stability 标记和版本化 API 文档为准。
- 已验证：`std::future` 直接 re-export `core::future`，`std::task` 汇合 `core::task` 与 `alloc::task`；核心协议本身不依赖完整 `std`。
- 已验证：事实有出处和检查通过仍不足以构成合格的机制解释；第一版把术语与源码索引放在主推理链之前，读者难以建立从等待、暂停到重新 poll 的连续模型。
- 已验证：2020 年的 async 访谈包含当时尚未实现或后来变化的设想，只能提供历史线索，不能用来陈述 Rust 1.91.1 的现行能力。
- 已验证：PR 合并后，计划中的分支、关联 PR 和 `Next Step` 可以落后于 Git 事实；恢复时必须先核对 merge commit、当前分支和工作树，不能直接执行旧交接文字。
- 已验证：`Poll<T>` 只描述一次调用是否取得输出，不是 Future 内部状态机的完整公共模型；`Pending` 的通知责任以“能够取得进展”为条件，永不完成的 `core::future::Pending<T>` 因此无需制造 wake。
- 已验证：Future 完成后再次 poll 的通用边界只有调用方不应这样做、实现方仍不得造成 undefined behavior；panic、永久阻塞或其他普通行为都不能由 trait 统一推断。
- 已验证：可运行实验和必要字段注释足以支持独立走读，但不会自动暴露读者是否把资源状态变化、Waker 通知和 executor 调度合并成同一动作；先要求读者复述再回写详细注释能够定位真实理解断点。
- 已验证：把内部理解检查和协作脚手架直接复制到实验入口，会与 lab 代码争夺注意力并制造额外阅读负担；入口只需在正确时机指向固定实验版本。
- 已验证：`Future` 的 inert 不能解释为其关联的所有外部工作都停止；固定 trait 文档明确区分必须靠 poll 推进或交付的 Future 与可能在另一个 task 中独立计算、只由 Future 转交结果的工作。
- 已验证：`Poll` 源文件同时包含 stable enum/映射 API 和 unstable `try_trait_v2` impl，引用固定源码时仍须逐项核对 stability，不能把文件级存在当作稳定保证。
- 已验证：`Poll::Ready` 只回答 Future 是否已交付输出，不等于业务成功；RFC 2592 有意移除内建错误类型，使 `Result` 等失败模型通过 `Future::Output` 正交组合。
- 已验证：把一个相邻 API 从当前因果链移走与省略它是两件不同的事；没有先做完整表面盘点和显式路由时，“避免旁支”会退化成读者无法判断后续是否回收的知识缺口。
- 已验证：核心签名中的短语法本身承载机制边界；如果首次出现时不区分 receiver、内外两层 mutable borrow、pointee pinning、placeholder lifetime 和关联输出，后文即使准确描述 `Pending`/`Ready`，读者仍缺少理解调用为何安全且可恢复的入口。
- 已验证：Rust 1.91.1 的 stable `Context::waker` 只需要共享访问，但 `Future::poll` 历史上选择 `&mut Context<'_>` 是为 task-local 等可变、非 `Send` context 保留扩展空间；当前固定源码中的 unstable `Context::ext` 可以证明该版本存在扩展实验，不能倒推它已经成为 stable 保证。
- 已验证：`core::future` 与 `core::task` 不是固定标准库全部异步表面；`AsyncFn*` 位于 `core::ops` 并进入 prelude，`AsyncIterator` 位于独立的 unstable `core::async_iter`，coroutine/async generator 还有 compiler-only bridge，因此全量路线必须先声明 inventory 边界再跨模块补扫。
- 已验证：图示不是免费的压缩手段；如果保留等价正文再追加图，总篇幅只会增加，因此图只负责替代难扫描的关系说明，精确契约、例外和证据仍由正文承担。
- 已验证：知识覆盖与连续阅读负担是两个不同指标；这次总源码行数从 315 增至 351，但主机制页降至 240 行，说明有效优化来自把机制解释与查阅型内容分成相邻阅读路径，而不是让整个知识单元变短。
- 已验证：教学注释适合把局部语法或短状态转换与含义贴在一起，但不能承载“为什么选择该 API”、公共保证或历史证据；把这些推理也塞进代码只会把文字墙换成注释墙。
- 已验证：降低连续阅读负担不等于让 Markdown 总行数单调下降；参考页从 111 行增至 148 行，是因为原有连续说明被固定源码节选、短表和小节边界替代，是否真正更易读必须以渲染页和指定读者复读为准。
- 已验证：mdBook 隐藏行适合让 imports 与 assertions 继续参与编译而不占读者视野，但隐藏 `impl`、字段或控制流会让可见片段失去语法与状态上下文；依赖 `core` 内部上下文的固定源码则应明确使用 `ignore` 并说明原因，而不是伪装成独立示例。
- 已验证：把一段解释改成代码、表格或图并不自动完成优化；如果旧正文和章尾总结仍在，新载体只会成为同一事实的额外副本，载体替换必须以删除被替代表达结束。
- 已验证：去重不能只比较相邻段落；章首结论、局部代码注释、表格或图、实现后的比较句和章尾总结可能共同形成跨区重复，因此教学评审需要按事实归属而不是按段落逐个检查。
- 已验证：`Future for Pin<P>` 的 blanket impl 是对同一个底层 Future 的类型适配，不会像 async block 那样产生新的匿名 Future；`Pin<Box<F>>` 的表示成本主要需要从 `Box` 存储、静态或动态派发与优化结果分别论证，不能把源码中的转发调用直接写成运行时性能结论。
- 待验证：编译器对 `async`/`.await` 的实际 lowering 细节留给路线图中固定 nightly 的独立里程碑，不能从 Reference 的近似脱糖推断具体 MIR 布局。

## Decision Log

- `2026-07-15` — 把当前复杂任务限制在 Future/poll/wake 基础闭环；compiler lowering、Pin 和 Tokio 分别有独立证据路径，合入当前任务会破坏可 review 性。
- `2026-07-15` — 第一个 PR 只建立总体进展模型、职责和源码地图，不展开后续机制的完整契约；在其合并后开启全新 Codex 对话，从已跟踪计划恢复里程碑 2，以真实任务验证冷启动而不是另造演示任务。
- `2026-07-16` — 写作设计保留在 active ExecPlan，学习书只呈现建立正确模型所需的知识，不机械增加目标、误解清单或证据地图。
- `2026-07-16` — 第一版不做局部润色，而是按“等待 → 保存状态 → poll → Pending/wake → 重新 poll → 分层职责”的因果链重写；源码地图移到机制解释之后。
- `2026-07-16` — 把评审接受的章节作为证据纪律、概念完整性和读者清晰度的质量下限，而不是后续章节的固定结构；评审失败必须撤销完成状态并回写可复用教训。
- `2026-07-16` — 里程碑 2 先独立闭合 Future/Poll 的调用与状态契约；只引入返回 `Pending` 所需的 Waker 责任，Waker 身份、生命周期、wake 合并和 `RawWaker` 安全边界仍由里程碑 3 独立验收。
- `2026-07-16` — 手动 poll 实验使用安全的 `Wake` 到 `Waker` 转换观察通知，不自行构造 `RawWaker`；先用单线程、测试控制的进展源隔离 poll 契约，线程间同步和 executor 调度不进入本切片。
- `2026-07-16` — 首个实验与 workspace/CI 接入按 L2 评审，因为它固定 Future 的可观察状态与唤醒责任；显式状态只包含 `Waiting`、`Ready` 和 `Completed`，测试驱动与报告 API 只服务当前实验，不作为后续 runtime 抽象。
- `2026-07-16` — 本切片的手写 diff 约 415 行，略过 400 行 review 预警；契约矩阵定义实验结论，首个 package 又必须与 Rust 检查同时接入，继续拆分会暂时留下无实验支撑的声明或未进入统一检查的代码，因此只把学习章节拆到后续 PR。
- `2026-07-16` — 采用“观察版合入 → 指定读者运行和复述 → 讨论纠偏 → 下一 PR 回写解释版”的实验关卡；解释版可以保留较完整的教学推理，但实验和 book 仍使用不同 commit，书页以固定 commit 和 diff 连接两个版本。
- `2026-07-16` — 教学检查点必须成为 `master` 可达历史；包含观察版 `39c7969` 的当前 PR 和后续解释版 PR 使用 merge commit，不采用默认 squash，并且在 book 记录 permalink 后不再改写对应 commit。
- `2026-07-16` — lab 不替代固定标准库源码：里程碑 2 走读 `Future::poll`、`Poll`、`Ready<T>` 与 `Pending<T>`，里程碑 3 再闭合 Waker 身份与安全边界，里程碑 4 才实现真实 ready queue 与重新 poll。
- `2026-07-16` — 学习书中的 lab 页面采用极短路标而不是教程：只保留出现原因、固定版本和 lab README 入口，理解检查、Git 操作、详细解释与源码计划分别留在权威位置。
- `2026-07-16` — Future/Poll 正文不重复上一章已经核验的 callback/poll 设计史，只回链该因果路径；本章新增的设计解释限于 `Poll` 为什么不内建错误，其余篇幅用于闭合调用契约、状态边界和证据分工。
- `2026-07-16` — “全部走读”采用覆盖与叙事分离：先盘点固定版本相邻表面并为每项指定当前正文、参考区、已有权威解释或命名后续里程碑，再按读者的因果依赖安排正文；不以源码顺序堆 API，也不以“当前主线不需要”为由无记录删除。
- `2026-07-16` — `Future::poll` 首次出现时完整拆解会改变机制结论的语法，但只在本章证明最小操作含义；`Pin` 的 projection/drop guarantee、自引用 soundness 与 `Context`/Waker 的完整生命周期分别由 `PINNING` 和里程碑 3 独立验收。
- `2026-07-16` — 当前正文同时承担连续机制解释与固定源码/API 查阅两种独立阅读任务，因此按内容去向表拆为相邻两页；完成后边界和两个最小 Future 的核心状态对照仍留在主线，辅助 API、其余 trait impl、stability、证据矩阵和后续路由进入参考页，拆分不以删减知识换取短篇幅。
- `2026-07-16` — 注释代码只替换能够逐项贴回 token、字段或状态转换的正文；能够独立编译的签名、示例与教学改写必须通过 mdBook test，依赖 `core` 内部路径、macro metavariable 或 orphan context 的固定源码节选则明确标为 `ignore`，同时记录固定路径、省略和新增注释；设计理由、trait 通用边界和证据限制继续使用正文，避免把教学改写伪装成上游源码。
- `2026-07-16` — 第五版不新增 `Pin` 图，因为注释签名和紧邻正文已经覆盖同一因果链；视觉优化优先让代码承担局部分支、让表格承担精确映射、让小节承担阅读停顿，图只在能替代跨节点关系说明时使用。
- `2026-07-16` — 第六版采用“一个事实、一个主要载体”：代码与教学注释负责局部语法、字段和短分支，表格负责精确映射与对比，图负责跨节点关系、状态和时序，正文负责原因、公共契约、例外与证据；其他载体只能引用或补充不能由主要载体表达的边界，不再完整复述结论。
- `2026-07-16` — 固定 `Pending<T>` 与 `Ready<T>` 不再为并排外观集中到完成后章节，而是分别放到 `Pending` 责任和完成后重复 poll 的推理位置；源码承担具体状态与分支，正文只推导通知责任或 trait 边界。
- `2026-08-25` — 将 Future 表示与成本确认为后续独立机制单元，而不组织成“写法越复杂、性能越高”的线性阶梯；章节先区分所有权、pinning、存储、派发与组合，再用等价契约、受控用例和测量说明场景化选择。

## Validation and Acceptance

- `node scripts/check-agent-workflow-review.mjs` — 2026-07-15 运行通过，下一次复审为 2026-07-29。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/system-boundaries.md` — 2026-07-15 对第一版正文运行，14 个外部链接全部通过；不作为新版验收结果。
- `make ci` — 2026-07-15 对第一版正文运行通过；不作为新版验收结果。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/system-boundaries.md research/notes/async-model/designing-futures-for-rust.md research/notes/async-model/why-async-rust.md docs/engineering-standards.md` — 2026-07-16 检查 37 个链接，全部通过。
- `make ci` — 2026-07-16 对新版正文、研究笔记和写作约束运行通过；Markdown、拼写、链接、doctest、Mermaid 与 Agent 工作流检查均通过。
- Mermaid 目视检查 — 2026-07-16 使用固定 Mermaid CLI 和 Chrome 检查进展时序图与标准库分层图，参与者、标签和箭头方向正确。
- 用户评审 — 2026-07-16 接受新版作为当前迭代基线，里程碑 1 可以完成。
- `node scripts/check-agent-workflow-review.mjs` — 2026-07-16 冷启动恢复时运行通过，当前日期为 2026-07-16，下一次复审为 2026-07-29。
- `git status --short --branch`、`git rev-parse HEAD origin/master` 与 `git diff --check` — 2026-07-16 确认位于干净的 `master`，本地与远端均指向 PR #13 的 merge commit `43fb7b0e901ae8e2b8502ce7d5a6ff77d0519be1`。
- `make docs` — 2026-07-16 冷启动恢复时运行通过；Markdown、拼写、离线链接、mdBook test 与 Mermaid 渲染均通过，没有产生 tracked diff。
- Rust 1.91.1 版本化 Reference/API 与固定 `rust-src` 交叉核对 — 2026-07-16 确认 async function 调用、`.await` 传播、`Future::poll`、`Poll`、最近 Waker 和完成后再次 poll 的矩阵结论；`rustc --version --verbose` 的 commit 与 `upstream/BASELINES.md` 一致。
- `cargo run -p future-poll` — 2026-07-16 观察到 `async fn` 函数体执行次数从首次 poll 前的 0 变为之后的 1，两次受控 poll 均为 `Pending`，旧 Waker 收到 0 次 wake、最近 Waker 收到 1 次 wake，随后重新 poll 得到 `Ready("完成")`。
- `cargo test -p future-poll` — 2026-07-16 通过 2 个单元测试和 1 个 doctest，覆盖惰性执行、最近 Waker 与两个具体 Future 的完成后行为差异。
- `make ci` — 2026-07-16 首次运行因计划内两个同名“内部写作设计”标题触发 Markdownlint `MD024`；改为里程碑特定标题后再次运行通过，覆盖 Rust 格式化、Clippy、测试、doctest、rustdoc、文档、Mermaid 和 Agent 工作流检查。
- `.tools/bin/lychee --no-progress --max-retries 3 labs/future-poll/README.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-07-16 检查 9 个链接，全部通过。
- 本地 Rust 1.91.1 `rust-src` 定位检查 — 2026-07-16 确认 `library/core/src/future/future.rs::Future::poll`、`library/core/src/task/poll.rs::Poll`、`library/core/src/future/ready.rs::Ready<T>::poll` 与 `library/core/src/future/pending.rs::Pending<T>::poll` 均存在，后续走读入口没有依赖猜测路径。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll-lab.md` — 2026-07-16 检查观察版 commit、源码和实验说明的 4 个外部链接，全部通过。
- `make ci` — 2026-07-16 对实验关卡书页、理解回写方法、Agent 约束和计划更新运行通过，覆盖 Rust 格式化、Clippy、2 个单元测试、1 个 doctest、rustdoc、Markdown、拼写、离线链接、mdBook、Mermaid 和工作流复审门禁。
- `make docs` — 2026-07-16 对精简后的 7 行 lab 入口和方法约束运行通过，mdBook 目录、Markdown、拼写、离线链接、doctest 与 Mermaid 检查均通过。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll-lab.md` — 2026-07-16 检查精简入口保留的观察版与 lab README 两个外部链接，全部通过。
- `git merge-base --is-ancestor 39c7969231ac1ae24d1bf64fb30419633bcb6875 master` — 2026-07-16 在 PR #14 合入后返回 0，确认观察版 commit 已由 merge commit 保留在 `master` 历史。
- `cargo test -p future-poll` — 2026-07-16 在更新后的 `master` 通过 2 个单元测试和 1 个 doctest，确认合并后实验基线可恢复。
- `git diff --unified=0 -- labs/future-poll/src/lib.rs` — 2026-07-16 确认解释版只增加 73 行注释，没有修改任何可执行行、测试或输出。
- `cargo fmt --all -- --check`、`cargo clippy -p future-poll --all-targets --all-features -- -D warnings` 与 `cargo test -p future-poll` — 2026-07-16 对解释版 commit 运行通过，覆盖格式化、lint、2 个单元测试和 1 个 doctest。
- `make ci` — 2026-07-16 对解释版注释、7 行 book 入口和恢复记录运行通过，覆盖 workspace 格式化、Clippy、测试、doctest、rustdoc、Markdown、拼写、离线链接、mdBook、Mermaid 和工作流复审门禁。
- `git merge-base --is-ancestor 39c7969231ac1ae24d1bf64fb30419633bcb6875 master` 与 `git merge-base --is-ancestor 6fb9ee38aedb9e6d1ee6d9de9426ea4827b9ade1 master` — 2026-07-16 均返回 0，确认观察版和解释版教学检查点都由 `master` 历史保留。
- `cargo test -p future-poll` — 2026-07-16 在 PR #15、#16 和 #17 合入后的 `master` 通过 2 个单元测试和 1 个 doctest，实验行为仍可恢复。
- `git status --short --branch` 与 `git rev-parse HEAD origin/master` — 2026-07-16 确认从干净且同步的 `master` `970acfa92b46a19e946306a4907e3ff6feaf01ff` 创建 `docs/future-poll-contract`，恢复时没有继承未提交变更。
- `rustc --version --verbose` 与 `cargo --version --verbose` — 2026-07-16 确认工具链分别为 Rust `1.91.1` commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 和 Cargo `1.91.1` commit `ea2d97820c16195b0ca3fadb4319fe512c199a43`，与 `upstream/BASELINES.md` 一致。
- 固定 `rust-src` 与 `coretests` 第一遍走读 — 2026-07-16 完整读取四个目标源码文件，并定位 `coretests/tests/task.rs::poll_const`、`coretests/tests/waker.rs::nop_waker::test_const_waker` 与 `coretests/tests/future.rs::_pending_impl_all_auto_traits`；没有把 Waker 实现、unstable `Try` impl 或具体 Future 的重复 poll 行为提升为通用稳定契约。
- `cargo test -p future-poll` — 2026-07-16 在固定源码第一遍走读后通过 2 个单元测试和 1 个 doctest，确认手动实验仍支持惰性执行、最新 Waker、`Pending -> Ready` 与具体 Future 重复 poll 差异的既有观察。
- `make docs` — 2026-07-16 对新增走读记录运行通过，覆盖 mdBook build/test、Markdownlint、拼写、离线链接和 Mermaid 渲染；mdBook 报告 `mdbook-mermaid` 按 `0.5.0` 构建而当前固定 mdBook 为 `0.5.2` 的非致命兼容性 warning。
- `make docs` — 2026-07-16 对 Future/Poll 正文首稿与目录接入运行通过，覆盖新章代码块的 mdBook test、Markdownlint、拼写、离线链接和全书 Mermaid 渲染；仍有相同的 mdBook preprocessor 非致命兼容性 warning。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md` — 2026-07-16 检查新章 19 个链接、15 个唯一目标，全部通过。
- `make external-links` — 2026-07-16 检查全库 194 个链接时，新章链接均未报错；命令最终因 `docs/agent-workflow/BASELINES.md` 中两个既有 OpenAI 页面返回 403 而失败，属于需要如实保留的全库外部检查结果。
- 技术与教学自检 — 2026-07-16 首稿曾被判断为能够回答当时的五个理解问题，但指定用户在前两节即发现签名语义缺失和 `Poll` 辅助 API 无铺垫；该自检结果已经失效，只保留为“内部自检不能替代真实读者评审”的反例。
- `make docs` — 2026-07-16 对第二版正文、相邻表面路由和工程约束运行通过，覆盖两个签名代码块与 `Poll::map` 示例的 mdBook test、Markdownlint、拼写、离线链接、构建和 Mermaid 渲染；仍只有既有 mdBook preprocessor 版本 warning。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/engineering-standards.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-07-16 检查 54 个链接、50 个唯一目标，全部通过，包括新增 lifetime Reference、固定 `Pin`/`Context` 源码、RFC 2349、rust-lang/rust #59113/#59119 与跨模块 async surface 链接。
- `make docs` — 2026-07-16 对结构第三版运行通过，mdBook 已按顺序构建并测试主机制页与固定源码/API 参考页，Markdownlint、拼写、离线链接和两张新增 Mermaid 图均通过；仍只有既有 mdBook preprocessor 版本 warning。
- Mermaid 目视检查 — 2026-07-16 用本机 Chrome 渲染固定 mdBook 页面后检查两张新图；签名图已从文字过小的横向版本调整为两条借用链汇入本次 poll 的纵向结构，`Poll<Result<T, E>>` 图清楚区分未完成、已完成和业务结果三层，均未重复上一章的 poll/wake 时序图。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/src/rust-async/future-poll-reference.md docs/engineering-standards.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-07-16 检查 58 个链接、52 个唯一目标，全部通过。
- `git diff --check` 与 `node scripts/check-agent-workflow-review.mjs` — 2026-07-16 均返回 0；当前 Agent 工作流复审仍有效至 2026-07-29。
- `make docs` — 2026-07-16 对结构第四版运行通过，mdBook test 编译了带中文教学注释的 `Future` 签名和 `Ready<T>`/`Pending<T>` 源码节选，Markdownlint、拼写、离线链接和保留的 `Poll<Result<T, E>>` Mermaid 图均通过；仍只有既有 mdBook preprocessor 版本 warning。
- 固定 mdBook 目视检查 — 2026-07-16 用本机 Chrome 检查结构第四版；注释签名能够在同一代码块内对应 token 与局部含义，`Pending` 责任矩阵在正常页面宽度下可扫描，固定源码节选没有与正文重复陈述状态转换。
- 内容去向检查 — 2026-07-16 逐项检索 receiver、move-out、pointee、`Unpin`、projection、三个 lifetime、可变 `Context` 理由、reborrow、binding mutability、关联输出、`must_use`、wake 责任、lost wake、重复 poll 和相邻 `Poll` API，全部仍在主机制页或参考页中出现。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/src/rust-async/future-poll-reference.md docs/engineering-standards.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-07-16 对结构第四版检查 58 个链接、51 个唯一目标，全部通过。
- `make docs` — 2026-07-16 对结构第五版运行通过，真实 `Poll::map` 调用和隐藏脚手架继续参与 mdBook test，固定 `ready!` 与 blanket impl 源码节选明确使用 `ignore`；Markdownlint、拼写、离线链接、构建和 Mermaid 渲染均通过，仍只有既有 mdBook preprocessor 版本 warning。
- 固定 mdBook 目视检查 — 2026-07-16 用本机 Chrome 以正常页面宽度检查结构第五版的两页；主机制页的 imports 已从可见代码移除，参考页六个职责区块边界清晰，`Poll` stability 表、`ready!`/`Try` 对照、两列证据表与路线表没有出现窄列文字墙。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/src/rust-async/future-poll-reference.md` — 2026-07-16 检查 33 个链接、28 个唯一目标，全部通过。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/src/rust-async/future-poll-reference.md docs/engineering-standards.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-07-16 对结构第五版及回写约束检查 58 个链接、51 个唯一目标，全部通过。
- `git diff --check` 与 `node scripts/check-agent-workflow-review.mjs` — 2026-07-16 对结构第五版均返回 0；当前 Agent 工作流复审仍有效至 2026-07-29。
- `make docs` — 2026-07-16 对结构第六版运行通过，两个固定最小 Future 的可编译节选分别位于对应机制小节，Markdownlint、拼写、离线链接、mdBook build/test 与 Mermaid 渲染均通过；仍只有既有 mdBook preprocessor 版本 warning。
- 固定 mdBook 目视检查 — 2026-07-16 用本机 Chrome 复查结构第六版；主机制页的表格、源码和结果分层图之间没有再次复述同一分支，参考页的 map stability、`ready!`/`Try` 与证据表缩短后仍能在正常宽度下独立读取。
- 内容归属检查 — 2026-07-16 逐项反查 receiver、move-out、pointee、`Unpin`、projection、三个 lifetime、可变 `Context` 理由、reborrow、binding mutability、关联输出、`must_use`、非阻塞、wake/lost wake、业务错误、重复 poll、`Try`/`FromResidual`、blanket impl、最小 Future 其余表面与相邻路线，均仍有单一主要载体或必要边界位置。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/src/rust-async/future-poll-reference.md docs/engineering-standards.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-07-16 对结构第六版及单一主要载体约束检查 56 个链接、51 个唯一目标，全部通过。
- `git diff --check` 与 `node scripts/check-agent-workflow-review.mjs` — 2026-07-16 对结构第六版均返回 0；当前 Agent 工作流复审仍有效至 2026-07-29。
- `make docs` — 2026-08-25 对验收收尾版运行通过，覆盖 mdBook build/test、Markdownlint、拼写、离线链接和 Mermaid 渲染；仍只有既有 mdBook preprocessor `0.5.0` 与 mdBook `0.5.2` 的非致命 compatibility warning。
- `cargo test -p future-poll` — 2026-08-25 通过 2 个单元测试和 1 个 doctest，继续覆盖惰性执行、最近 Waker、完成转换与具体 Future 的重复 poll 差异。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/future-poll.md docs/src/rust-async/future-poll-reference.md docs/engineering-standards.md docs/agent-workflow/plans/active/future-poll-wake-foundations.md` — 2026-08-25 检查 58 个链接、53 个唯一目标，全部通过。
- `make ci` — 2026-08-25 对 Future/Poll PR-ready 工作树运行通过，覆盖 workspace 格式化、Clippy、测试、doctest、rustdoc、Markdown、拼写、离线链接、mdBook 与 Mermaid；`agent-workflow-check --validate` 同时准确报告复审已于 2026-07-29 到期。

## Idempotence and Recovery

标准库源码只读自 `$(rustc --print sysroot)/lib/rustlib/src/rust/library`，不修改工具链文件，也不需要克隆完整 `rust-lang/rust` 仓库。
恢复时先核对分支、Git 状态、固定工具链和本计划，再从 `Next Step` 继续；文档生成物不作为事实源。
跨机器演练使用干净 clone、`make tools` 和固定 `rust-src` 恢复环境，不复制 `.tools/`、`node_modules/`、`target/` 或 `docs/book/`。

## Next Step

由用户 review、分组 stage、commit、push 并创建 Future/Poll 契约正文 PR；保留当前到期提醒 Issue，随后在独立分支和 PR 中完成已经获得许可的 deep workflow review，再恢复本计划并为 Future 表示与成本章节确定依赖正确的路线，复审完成前不开始新的实质调研或写作。

## Outcomes and Retrospective

任务完成前保留为空。
