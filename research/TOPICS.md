# 研究主题地图

本文档把开放问题连接到阅读资料和最终核验目标。
一次研究任务只选择一个主要问题；相邻主题只有在阻塞该问题时才展开。

| ID | 阶段 | 主要问题 | 候选资料 | 必须补充的核验 | 状态 |
| --- | --- | --- | --- | --- | --- |
| `ASYNC-MODEL` | 1 | Rust 为什么选择基于 `Future` 的 stackless coroutine；调用、poll 和 `.await` 分别发生什么 | `WB-ASYNC-ORIGIN`、`NM-WB-INTERVIEW` | Rust Reference、标准库契约、固定编译器输出与实验 | 未开始 |
| `PINNING` | 1 | 自引用状态机为什么需要 pinning；`Pin`、`Unpin`、pinned places 与 `Move` 分别解决什么 | `WB-PIN`、`WB-PINNED-PLACES`、`NM-MINPIN`、`NM-MOVE` | 标准库契约、RFC 2349、固定源码和最小实验 | 未开始 |
| `ASYNC-TRAITS` | 1、6 | async trait 的返回类型、`Send` 约束、dyn dispatch 与分配策略如何互相影响 | `NM-SEND-BOUND`、`NM-DYN-ASYNC` | 已稳定语法、当前项目目标、编译器行为和实验 | 未开始 |
| `TOKIO-MIO-STACK` | 2 | Mio、Tokio runtime、driver、task 与公共 I/O API 的职责边界如何形成 | `NM-CARL-INTERVIEW` | 固定 Tokio/Mio 公共契约、源码、测试和黑盒实验 | 未开始 |
| `FUTURE-TASK` | 2、4 | Future、task、spawn 和运行时之间有哪些不同并发能力与所有权代价 | `WB-FUTURES`、`NM-INTERVIEWS` | Tokio 固定源码、自研运行时实验与任务契约 | 未开始 |
| `CANCELLATION` | 1、4、6 | drop Future、`select!`、panic 和 shutdown 分别留下哪些部分效果与清理责任 | `NM-CANCEL-PANIC`、`NM-CANCEL-CASE`、`WB-CLEANUP` | 标准库与 Tokio 契约、固定源码、取消安全实验 | 未开始 |
| `SCOPED-TASKS` | 4、6、7 | scoped/structured concurrency 如何约束任务生命周期、借用、取消与错误传播；哪些契约独立于具体 scheduler | `WB-SCOPE`、`WB-CLEANUP`、`NM-MOVE` | 当前 Rust 类型能力、结构化并发契约与跨 scheduler 原型实验 | 未开始 |
| `SCHEDULER-VARIANTS` | 2、4、7 | work stealing、current-thread、thread-per-core、本地队列、优先级与 CPU 亲和性分别适合哪些 workload，代价是什么 | `RAIN-DESIGN`、`RAIN-GITHUB` | 固定 Tokio 与候选运行时源码、调度实验、公平性、负载均衡和跨核行为测量 | 未开始 |
| `IO-DRIVER-VARIANTS` | 2、3、7 | readiness/completion 与共享/per-core driver 如何影响所有权、取消、批处理、唤醒和性能 | `NM-CARL-INTERVIEW`、`RAIN-GITHUB` | 操作系统官方接口、固定实现、独立 driver 实验与资源生命周期分析 | 未开始 |
| `RUNTIME-PERF` | 7 | 如何公平比较 Tokio、通用基线与专用变体，并通过 profile、trace、计数器和消融解释差异 | `RAIN-DESIGN`、`RAIN-GITHUB` | 可复现 benchmark、固定环境、等价契约、统计方法与实现级证据 | 未开始 |
| `ASYNC-DIRECTION` | 全程 | 设计者提出的语言方向哪些已进入官方计划、实现或稳定版本，哪些仍是探索 | `WB-ASYNC-PLAN`、`RUST-ASYNC-2026` | RFC、tracking issue、Rust release notes 与固定工具链 | 未开始 |

## 相邻主题进入条件

- NLL 与 Polonius 只在借用跨越 `.await`、lending pattern 或 scoped task 的问题需要时进入当前笔记。
- GAT、RPITIT、TAIT、RTN 和 trait solver 只在 async trait 的具体表达能力需要时展开。
- `Move`、guaranteed destructors 和 async drop 必须明确区分历史方案、当前项目目标、已实现 feature 与稳定能力。
- 通用语言治理、其他编程语言和非异步文章默认不进入本仓库，除非它们回答当前主题地图中的明确问题。
