# tiny-async-lab

`tiny-async-lab` 是一个面向深入学习的 Rust 异步实验仓库。目标不是尽快做出一个可用运行时，而是建立从语言契约、编译器模型、成熟运行时实现到自研实现和工程实践的完整理解。

## 学习主线

1. 研究 Rust 标准库中的异步契约与基础类型，包括 `Future`、`Poll`、`Context`、`Waker`、`Pin` 和 `Unpin`。
2. 研究 `async`/`.await` 的语义、近似脱糖和编译器状态机。
3. 学习 Tokio 的公共使用模型、内部架构和实现取舍，并研究 Mio 提供的底层 I/O 抽象。
4. 实现独立于运行时的 `tiny-mio`，再基于它实现 `tiny-runtime`。
5. 使用 Tokio 和 `tiny-runtime` 分别实现同一批异步最佳实践案例。

## 两条并行学习线

源码研究同时关注两类问题，但分两遍完成：

- **原理与正确性**：契约、状态、所有权、不变量、唤醒、并发、取消和性能约束。
- **代码与工程质量**：命名、模块边界、类型设计、错误处理、文档、测试、平台隔离和可维护性。

第一遍优先理解“为什么正确、如何运行”，并随手记录值得复查的设计；完成一个机制单元后进行第二遍工程质量复盘，再决定哪些模式适合应用到自研代码。详细方法见 [`docs/source-reading-method.md`](docs/source-reading-method.md)。

## 仓库分区

- `upstream/`：固定版本的上游源码清单，以及由 Git 忽略的本地 checkout。
- `research/`：历史文章、PDF、个人笔记及其摘要和主题映射。
- `docs/`：经过源码或实验验证后形成的正式知识。
- `labs/`：独立、可运行、可冻结的学习实验。
- `crates/`：持续演进的 `tiny-mio`、`tiny-runtime` 及后续配套 crate。
- `practices/`：按场景组织的 Tokio/自研运行时成对实现。

这些目录将在进入相应阶段时逐步创建，不预先生成大量空目录。

## 阅读路径

仓库提供两条互相连接、但不复制完整结论的学习路径：

- **机制优先**：从 `docs/` 的定义、契约和源码证据开始，再运行链接的 `labs/` 验证行为。
- **实验优先**：先运行 `labs/` 观察现象，再回到链接的 `docs/` 建立完整解释。

crate rustdoc 只承担公共 API 契约和使用入口，持久且难以撤销的设计决定记录在按需创建的 `docs/adr/`。
历史文章和个人材料保留在 `research/`，验证前不作为项目结论。

## 工具链

根目录使用固定的 Rust `1.91.1`，并安装 `rust-src`、Rustfmt 和 Clippy。标准库源码由 `rust-src` 提供；完整的 `rust-lang/rust` 仓库不属于默认环境。

编译器 lowering 实验将在独立目录中使用按日期固定的 nightly，不改变根 workspace 的稳定工具链。

## 文档与协作约定

面向读者的项目文档统一使用中文，Rust 标识符、文件名和需要与上游对应的术语保留英文。Commit subject 与 Pull Request 标题统一使用英文，讨论背景、实现取舍和验证过程的正文可以使用中文。

分支、commit、Pull Request 和合并规则见 [`CONTRIBUTING.md`](CONTRIBUTING.md)。

知识证据、文档表达、代码设计和小步变更的要求见 [`docs/engineering-standards.md`](docs/engineering-standards.md)。

## 当前状态

项目处于仓库奠基阶段。后续工作和完成条件以 [`ROADMAP.md`](ROADMAP.md) 为准。
