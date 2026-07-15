# 建立 Future、Poll 与唤醒协议基础

- 状态：active
- 最近更新：2026-07-16
- 关联路线图：`ROADMAP.md` 阶段 0 的恢复演练，以及阶段 1 的异步系统边界、`Future`/`Poll`、唤醒协议和最小 executor
- 关联 PR：里程碑 1 的 [#13](https://github.com/captainlee1024/tiny-async-lab/pull/13) 已合并到 `master`，merge commit `43fb7b0e901ae8e2b8502ce7d5a6ff77d0519be1`；里程碑 2 的首个实验 PR 尚未创建

## 目标

用固定 Rust 版本的公开契约、标准库源码和聚焦实验，建立从 `async` 产生 Future 到 executor 根据唤醒重新 poll 的完整基础模型。
任务完成后，没有原对话的新 Agent 应能从本计划和仓库事实继续工作，读者应能区分 Future、task、executor、scheduler、reactor 和 runtime，并解释 poll/park/wake 闭环的关键不变量。

## 本次不做

- 不研究编译器实际生成的 HIR、THIR、MIR 和状态机布局。
- 不深入研究 `Pin`、取消、`Send`/`Sync` 或 async trait。
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
- [ ] `Future`/`Poll` 章节闭合首次 poll、`Pending`、`Ready`、重复 poll 和最新 Waker 契约，并由最小手动 poll 实验验证可观察行为。
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

- 主问题：调用者每次 poll Future 时，调用者和 Future 实现分别承诺什么；首次 poll、`Pending`、`Ready` 和完成后再次 poll 的边界如何组成一个不会把具体实现行为误写成通用保证的状态模型？
- 因果链：创建惰性 Future → 外部使用 `Pin<&mut Self>` 与当前 `Context` 主动 poll → Future 尝试推进 → `Pending` 保留未完成状态并安排进展通知 → 条件改变后调用者重新 poll → `Ready` 交付输出并结束本次计算；完成后再次 poll 的允许行为必须回到公共契约，而不能从单个实验外推。

| 结论类型 | 主要证据 | 本里程碑边界 |
| --- | --- | --- |
| 当前语言语义 | Rust 1.91.1 Reference 中的 async function 与 await expression | 只证明编译器生成 Future 的调用、首次执行和 `.await` 传播语义，不替任意手写 Future 补充 trait 契约 |
| 公共协议 | Rust 1.91.1 `Future::poll`、`Poll` 与完成后 poll 的 API 文档 | 分开记录调用者责任、实现者责任、保证、允许行为和未规定行为；完整 Waker 身份、生命周期与安全边界留给里程碑 3 |
| 固定实现 | commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` 的 `core::future` 与 `core::task` | 复查 doc comment、stability 和标准库聚焦实现，不把某个 Future 的完成后行为提升为 trait 保证 |
| 可观察行为 | 标准库手动 poll 实验 `labs/future-poll` | 分别观察创建、首次 poll、`Pending`、外部条件改变、重新 poll 和 `Ready`；实验只证明被测 Future 与测试驱动的行为 |
| 设计原因 | RFC 2592 与 `ASYNC-MODEL` 已路由资料 | 解释 poll 模型的取舍，不用历史文章覆盖当前 API 契约 |

首个实验切片使用显式状态和测试控制的进展条件，保证返回 `Pending` 的路径登记当前 Waker，并由测试在条件改变时触发通知。
实验先闭合单 Future 的调用与状态观察，不实现 ready queue、park、跨线程调度或 `RawWaker`，这些职责分别留给里程碑 3 和 4。

#### 实验关卡与读者回写

观察版固定为 commit `39c7969231ac1ae24d1bf64fb30419633bcb6875`，相邻书页 `docs/src/rust-async/future-poll-lab.md` 负责把读者从总体模型路由到运行、源码阅读和理解检查。
当前 PR 保留观察版已有的必要注释，不加入本轮讨论形成的详细解释。
观察版合入后，指定读者完成逐段走读、复述和纠偏，再由下一 PR 先形成解释版 lab commit，随后用独立 book commit 记录观察版、解释版及固定 diff。

解释版允许采用面向教学的高密度注释，完整保存已经核验的状态转换、责任分工、顺序原因、容易混淆的直觉、真实实现差异和后续源码入口，而不是只留下简短标签。
它仍只解释当前实验及其边界；跨段的公共契约留在书中，未经固定契约或源码核验的讨论不进入注释。

本关卡的“理解足够”以本里程碑已有五个理解问题和书页中的具体检查为准，只验收 Future/Poll 实验边界。
`Future::poll`、`Poll`、`Ready<T>` 和 `Pending<T>` 的固定标准库源码必须在里程碑 2 完成前逐个走读；Waker 系列源码和真实重新入队分别由里程碑 3、4 回收。

正文和实验完成后应能仅依靠它们回答以下问题：

1. 调用 `async fn`、首次 poll 返回的 Future 和重新 poll 分别由谁触发，函数体何时开始执行？
2. `Pending` 表示什么、不表示什么，Future 实现方和调用方各自还承担什么责任？
3. 为什么同一个 Future 的一次实验结果不能定义所有 Future 的首次或重复 poll 行为？
4. `Ready` 之后调用方应如何处理 Future，错误地再次 poll 时公共契约仍保证和不保证什么？
5. 哪些结论来自 Reference、公共 API、固定源码、聚焦实验或历史资料，为什么这些证据不能互相替代？

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

## Surprises and Discoveries

- 已验证：Rust 1.91.1 的 `rust-src` 同时包含 stable 与 unstable 异步 API；源码存在不能证明 API 已稳定，正式章节必须以 stability 标记和版本化 API 文档为准。
- 已验证：`std::future` 直接 re-export `core::future`，`std::task` 汇合 `core::task` 与 `alloc::task`；核心协议本身不依赖完整 `std`。
- 已验证：事实有出处和检查通过仍不足以构成合格的机制解释；第一版把术语与源码索引放在主推理链之前，读者难以建立从等待、暂停到重新 poll 的连续模型。
- 已验证：2020 年的 async 访谈包含当时尚未实现或后来变化的设想，只能提供历史线索，不能用来陈述 Rust 1.91.1 的现行能力。
- 已验证：PR 合并后，计划中的分支、关联 PR 和 `Next Step` 可以落后于 Git 事实；恢复时必须先核对 merge commit、当前分支和工作树，不能直接执行旧交接文字。
- 已验证：`Poll<T>` 只描述一次调用是否取得输出，不是 Future 内部状态机的完整公共模型；`Pending` 的通知责任以“能够取得进展”为条件，永不完成的 `core::future::Pending<T>` 因此无需制造 wake。
- 已验证：Future 完成后再次 poll 的通用边界只有调用方不应这样做、实现方仍不得造成 undefined behavior；panic、永久阻塞或其他普通行为都不能由 trait 统一推断。
- 已验证：可运行实验和必要字段注释足以支持独立走读，但不会自动暴露读者是否把资源状态变化、Waker 通知和 executor 调度合并成同一动作；先要求读者复述再回写详细注释能够定位真实理解断点。
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

## Idempotence and Recovery

标准库源码只读自 `$(rustc --print sysroot)/lib/rustlib/src/rust/library`，不修改工具链文件，也不需要克隆完整 `rust-lang/rust` 仓库。
恢复时先核对分支、Git 状态、固定工具链和本计划，再从 `Next Step` 继续；文档生成物不作为事实源。
跨机器演练使用干净 clone、`make tools` 和固定 `rust-src` 恢复环境，不复制 `.tools/`、`node_modules/`、`target/` 或 `docs/book/`。

## Next Step

先完成并验证当前 PR 的观察版 `labs/future-poll`、workspace/CI 接入、契约矩阵、实验关卡书页和学习方法约束，由用户按独立模块提交、推送并完成 review，再使用 merge commit 保留观察版检查点 `39c7969`。
合入后从最新 `master` 创建下一分支，把本轮读者讨论梳理为解释版 lab commit，再用独立 book commit 补齐第二个固定版本和 diff。
解释版合入后走读本计划列出的 Future/Poll 固定标准库源码，并从 `research/CATALOG.md` 重新进入 `ASYNC-MODEL` 已路由资料，只补齐本里程碑所需的设计原因与证据边界，再起草 Future/Poll 正文。

## Outcomes and Retrospective

任务完成前保留为空。
