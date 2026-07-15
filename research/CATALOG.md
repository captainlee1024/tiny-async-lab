# 研究资料目录

本文档是研究资料的唯一登记入口，不是推荐榜单或项目结论。
首批入口于 2026-07-14 核验；开始阅读前先从 [`TOPICS.md`](TOPICS.md) 选择问题。

## 来源入口

| ID | 来源 | 形式 | 用途 | 状态 |
| --- | --- | --- | --- | --- |
| `WB-PROFILE` | [Without Boats 的 Rust 官方成员记录](https://rust-lang.org/governance/people/withoutboats/) | 官方资料 | 核对其历史项目角色，不用于证明技术主张 | 按需核验 |
| `WB-BLOG` | [Without Boats 博客](https://without.boats/blog/) | 博客合集 | 异步、pinning、并发和语言设计的历史与取舍 | 按主题阅读 |
| `WB-GITHUB` | [withoutboats](https://github.com/withoutboats) | 代码仓库入口 | 查找文章对应原型；使用实现结论前固定 repository 与 commit | 按需核验 |
| `AT-BLOG` | [Aaron Turon 的博客](https://aturon.github.io/blog/) | 博客合集 | Future、poll/task 模型与零成本抽象的早期设计记录 | 按主题阅读 |
| `NM-PROFILE` | [Niko Matsakis 的 Rust 官方成员记录](https://rust-lang.org/governance/people/nikomatsakis/) | 官方资料 | 核对当前与历史项目角色，不用于证明技术主张 | 按需核验 |
| `NM-RUST-BLOG` | [baby steps：Rust](https://smallcultfollowing.com/babysteps/categories/rust/) | 博客合集 | 异步、借用检查和 trait system 的设计探索 | 按主题阅读 |
| `NM-INTERVIEWS` | [Async Interviews](https://smallcultfollowing.com/babysteps/categories/asyncinterviews/) | 访谈与视频合集 | 追踪 async/await 稳定后不同实现者看到的问题 | 按主题阅读 |
| `NM-GITHUB` | [nikomatsakis](https://github.com/nikomatsakis) | 代码仓库入口 | 查找文章对应原型和形式化工作；使用前固定 commit | 按需核验 |
| `RAIN-GITHUB` | [Rain](https://github.com/wzxzhuxi/Rain) | C++23 代码仓库 | 调研 thread-per-core、I/O driver 与调度取舍；使用实现结论前固定 commit | 入口已核验；按需固定 |

## 首批阅读队列

| ID | 资料 | 发布日期 | 主要主题 | 路线图阶段 | 状态 |
| --- | --- | --- | --- | --- | --- |
| `AT-FUTURES-DESIGN` | [Designing futures for Rust](https://aturon.github.io/blog/2016/09/07/futures-design/) | 2016-09-07 | callback 与 poll/task 模型的设计取舍 | 阶段 1 | [已摘录](notes/async-model/designing-futures-for-rust.md) |
| `WB-ASYNC-ORIGIN` | [Why async Rust?](https://without.boats/blog/why-async-rust/) | 2023-10-15 | stackless coroutine、Future 模型、async/await 历史 | 阶段 1 | [已摘录](notes/async-model/why-async-rust.md) |
| `WB-PIN` | [Pin](https://without.boats/blog/pin/) | 2024-07-19 | 自引用 Future、pinning 需求与历史替代方案 | 阶段 1 | 待读 |
| `WB-PINNED-PLACES` | [Pinned places](https://without.boats/blog/pinned-places/) | 2024-07-23 | `Pin` 的语言级改进方案与兼容性取舍 | 阶段 1 | 待读 |
| `WB-ASYNC-PLAN` | [A four year plan for async Rust](https://without.boats/blog/a-four-year-plan/) | 2023-11-07 | async 语言能力缺口、`Move` 与长期方向 | 阶段 1、6 | 待读 |
| `WB-FUTURES` | [Let futures be futures](https://without.boats/blog/let-futures-be-futures/) | 2024-02-03 | task、Future、并发组合与阻塞边界 | 阶段 2、6 | 待读 |
| `WB-SCOPE` | [The Scoped Task trilemma](https://without.boats/blog/the-scoped-task-trilemma/) | 2023-04-08 | concurrency、parallelizability 与 borrowing | 阶段 4 | 待读 |
| `WB-CLEANUP` | [Asynchronous clean-up](https://without.boats/blog/asynchronous-clean-up/) | 2024-02-24 | 取消、清理、async drop 与 scoped task | 阶段 4、6 | 待读 |
| `NM-WB-INTERVIEW` | [Async Interview #7: Withoutboats](https://smallcultfollowing.com/babysteps/blog/2020/03/10/async-interview-7-withoutboats/) | 2020-03-10 | async/await 与 `Pin` 的设计历史、后续问题 | 阶段 1 | 已归档；仅作历史线索 |
| `NM-CARL-INTERVIEW` | [Async Interview #3: Carl Lerche](https://smallcultfollowing.com/babysteps/blog/2019/12/23/async-interview-3-carl-lerche/) | 2019-12-23 | Tokio、Mio、I/O 栈与运行时边界 | 阶段 2 | 待读 |
| `NM-CANCEL-PANIC` | [Panics vs cancellation, part 1](https://smallcultfollowing.com/babysteps/blog/2022/01/27/panics-vs-cancellation-part-1/) | 2022-01-27 | panic 与 drop cancellation 的语义类比 | 阶段 1、4、6 | 待读 |
| `NM-CANCEL-CASE` | [Async cancellation: a case study of pub-sub in mini-redis](https://smallcultfollowing.com/babysteps/blog/2022/06/13/async-cancellation-a-case-study-of-pub-sub-in-mini-redis/) | 2022-06-13 | `select!`、细粒度取消与可靠性 | 阶段 1、4、6 | 待读 |
| `NM-SEND-BOUND` | [Send Bound Problem](https://smallcultfollowing.com/babysteps/series/send-bound-problem/) | 2023 | async trait 返回 Future 的 `Send` 约束 | 阶段 1、6 | 待读 |
| `NM-DYN-ASYNC` | [Dyn Async Traits](https://smallcultfollowing.com/babysteps/series/dyn-async-traits/) | 2021–2025 | dyn dispatch、分配透明度与 async trait | 阶段 1、6 | 待读 |
| `NM-MINPIN` | [MinPin](https://smallcultfollowing.com/babysteps/blog/2024/11/05/minpin/) | 2024-11-05 | pinning 设计公理及与其他方案的比较 | 阶段 1 | 待读 |
| `NM-MOVE` | [Move, Destruct, Forget, and Rust](https://smallcultfollowing.com/babysteps/blog/2025/10/21/move-destruct-leak/) | 2025-10-21 | `Move`、受控析构、async scope 与清理保证 | 阶段 1、4 | 待读 |
| `RUST-ASYNC-2026` | [Rust 2026：Just add async](https://rust-lang.github.io/rust-project-goals/2026/roadmap-just-add-async.html) | 2026 | 当前项目方向和候选时间线的官方核验入口 | 全程 | 按需核验 |
| `RAIN-DESIGN` | [Rain：从零开始构建异步库](https://linux.do/t/topic/1949965) | 2026-04-12 | thread-per-core、`SO_REUSEPORT` 与自述性能结果形成的候选假设 | 阶段 2、7 | 待读、待复现 |

阅读队列只建立检索入口。
某篇资料是否仍反映当前状态，必须在笔记中继续核对已接受 RFC、tracking issue、固定源码和当前官方项目目标。
