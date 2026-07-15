# 建立 Future、Poll 与唤醒协议基础

- 状态：active
- 最近更新：2026-07-16
- 关联路线图：`ROADMAP.md` 阶段 0 的恢复演练，以及阶段 1 的异步系统边界、`Future`/`Poll`、唤醒协议和最小 executor
- 关联 PR：当前分支 `docs/async-system-boundaries`，PR 尚未创建

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

#### 内部写作设计

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

### 3. Context 与唤醒协议

解释 Waker 身份、最新 Waker、wake 合并、资源生命周期和 `RawWaker` 安全边界，再用聚焦实验观察重新 poll。

### 4. 标准库最小 executor

只使用标准库实现教学用 executor，闭合 poll/park/wake 路径，并记录生产实现仍需处理的竞态、任务管理和关闭问题。

## Progress

- [x] `2026-07-15` — 完成里程碑 1 的第一版正文、固定源码地图和检查，但尚未通过读者评审。
- [x] `2026-07-16` — 用户评审判定第一版未形成充分的因果与教学结构；撤销完成结论，补充内部写作设计并开始整章重写。
- [x] `2026-07-16` — 新版按等待、保存状态、poll、wake、重新 poll 与职责边界的因果链完成，并通过技术评审、教学自检、图形目视检查和本地 CI。
- [x] `2026-07-16` — 用户接受新版作为当前迭代的质量基线；完成里程碑 1，并同步更新路线图与可复用写作约束。

## Surprises and Discoveries

- 已验证：Rust 1.91.1 的 `rust-src` 同时包含 stable 与 unstable 异步 API；源码存在不能证明 API 已稳定，正式章节必须以 stability 标记和版本化 API 文档为准。
- 已验证：`std::future` 直接 re-export `core::future`，`std::task` 汇合 `core::task` 与 `alloc::task`；核心协议本身不依赖完整 `std`。
- 已验证：事实有出处和检查通过仍不足以构成合格的机制解释；第一版把术语与源码索引放在主推理链之前，读者难以建立从等待、暂停到重新 poll 的连续模型。
- 已验证：2020 年的 async 访谈包含当时尚未实现或后来变化的设想，只能提供历史线索，不能用来陈述 Rust 1.91.1 的现行能力。
- 待验证：编译器对 `async`/`.await` 的实际 lowering 细节留给路线图中固定 nightly 的独立里程碑，不能从 Reference 的近似脱糖推断具体 MIR 布局。

## Decision Log

- `2026-07-15` — 把当前复杂任务限制在 Future/poll/wake 基础闭环；compiler lowering、Pin 和 Tokio 分别有独立证据路径，合入当前任务会破坏可 review 性。
- `2026-07-15` — 第一个 PR 只建立总体进展模型、职责和源码地图，不展开后续机制的完整契约；在其合并后开启全新 Codex 对话，从已跟踪计划恢复里程碑 2，以真实任务验证冷启动而不是另造演示任务。
- `2026-07-16` — 写作设计保留在 active ExecPlan，学习书只呈现建立正确模型所需的知识，不机械增加目标、误解清单或证据地图。
- `2026-07-16` — 第一版不做局部润色，而是按“等待 → 保存状态 → poll → Pending/wake → 重新 poll → 分层职责”的因果链重写；源码地图移到机制解释之后。
- `2026-07-16` — 把评审接受的章节作为证据纪律、概念完整性和读者清晰度的质量下限，而不是后续章节的固定结构；评审失败必须撤销完成状态并回写可复用教训。

## Validation and Acceptance

- `node scripts/check-agent-workflow-review.mjs` — 2026-07-15 运行通过，下一次复审为 2026-07-29。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/system-boundaries.md` — 2026-07-15 对第一版正文运行，14 个外部链接全部通过；不作为新版验收结果。
- `make ci` — 2026-07-15 对第一版正文运行通过；不作为新版验收结果。
- `.tools/bin/lychee --no-progress --max-retries 3 docs/src/rust-async/system-boundaries.md research/notes/async-model/designing-futures-for-rust.md research/notes/async-model/why-async-rust.md docs/engineering-standards.md` — 2026-07-16 检查 37 个链接，全部通过。
- `make ci` — 2026-07-16 对新版正文、研究笔记和写作约束运行通过；Markdown、拼写、链接、doctest、Mermaid 与 Agent 工作流检查均通过。
- Mermaid 目视检查 — 2026-07-16 使用固定 Mermaid CLI 和 Chrome 检查进展时序图与标准库分层图，参与者、标签和箭头方向正确。
- 用户评审 — 2026-07-16 接受新版作为当前迭代基线，里程碑 1 可以完成。

## Idempotence and Recovery

标准库源码只读自 `$(rustc --print sysroot)/lib/rustlib/src/rust/library`，不修改工具链文件，也不需要克隆完整 `rust-lang/rust` 仓库。
恢复时先核对分支、Git 状态、固定工具链和本计划，再从 `Next Step` 继续；文档生成物不作为事实源。
跨机器演练使用干净 clone、`make tools` 和固定 `rust-src` 恢复环境，不复制 `.tools/`、`node_modules/`、`target/` 或 `docs/book/`。

## Next Step

提交并合并当前 `docs/async-system-boundaries` PR 后，停止使用原对话并开启全新的 Codex 对话恢复里程碑 2。
新对话只提供仓库，并要求：“读取 `AGENTS.md` 和唯一的 active ExecPlan；修改文件前，先按 `PLANS.md` 的恢复核验回答五个问题。”

## Outcomes and Retrospective

任务完成前保留为空。
