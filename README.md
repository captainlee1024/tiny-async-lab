# tiny-async-lab

`tiny-async-lab` 是一个面向深入学习的 Rust 异步实验仓库。目标不是尽快做出一个可用运行时，而是建立从语言契约、编译器模型、成熟运行时实现到自研实现和工程实践的完整理解。

## 学习主线

1. 研究 Rust 标准库中的异步契约与基础类型，包括 `Future`、`Poll`、`Context`、`Waker`、`Pin` 和 `Unpin`。
2. 研究 `async`/`.await` 的语义、近似脱糖和编译器状态机。
3. 学习 Tokio 的公共使用模型、内部架构和实现取舍，并研究 Mio 提供的底层 I/O 抽象。
4. 实现独立于运行时的 `tiny-mio`，再基于它实现通用 `tiny-runtime` 基线。
5. 使用 Tokio 和通用 `tiny-runtime` 分别实现同一批异步最佳实践案例。
6. 在通用基线与配套生态稳定后，针对明确场景探索专用运行时变体，并通过公平基准和性能归因解释其收益与代价。

## 两条并行学习线

源码研究同时关注两类问题，但分两遍完成：

- **原理与正确性**：契约、状态、所有权、不变量、唤醒、并发、取消和性能约束。
- **代码与工程质量**：命名、模块边界、类型设计、错误处理、文档、测试、平台隔离和可维护性。

第一遍优先理解“为什么正确、如何运行”，并随手记录值得复查的设计；完成一个机制单元后进行第二遍工程质量复盘，再决定哪些模式适合应用到自研代码。详细方法见 [`docs/source-reading-method.md`](docs/source-reading-method.md)。

## 仓库分区

- `upstream/`：[工具与上游源码基线](upstream/BASELINES.md)，以及由 Git 忽略的本地 checkout。
- `research/`：[尚未验证的资料、阅读目录与主题地图](research/README.md)。
- `docs/`：经过源码或实验验证后形成的正式知识与项目约束；`docs/src/` 是单本学习书的源文件。
- `labs/`：独立、可运行、可冻结的学习实验。
- `crates/`：持续演进的 `tiny-mio`、`tiny-runtime` 及后续配套 crate。
- `practices/`：按场景组织的 Tokio/自研运行时成对实现。

这些目录将在进入相应阶段时逐步创建，不预先生成大量空目录。

## 阅读路径

仓库提供两条互相连接、但不复制完整结论的学习路径：

- **机制优先**：从[学习书导读](docs/src/introduction.md)进入定义、契约和源码证据，再运行链接的 `labs/` 验证行为。
- **实验优先**：先运行 `labs/` 观察现象，再回到学习书中链接的章节建立完整解释。

crate rustdoc 只承担公共 API 契约和使用入口，持久且难以撤销的设计决定记录在按需创建的 `docs/adr/`。
历史文章和个人材料保留在 `research/`，验证前不作为项目结论。

学习书使用 mdBook 组织，生成结果位于 Git 忽略的 `docs/book/`。

根目录的 Makefile 提供常用本地入口：

- `make tools`：将固定版本的辅助工具安装到 Git 忽略的仓库本地目录。
- `make upstream`：按固定 tag 和 commit 检出 Tokio 与 Mio 的完整 Git 仓库。
- `make book`：构建学习书。
- `make book-preview`：在 `127.0.0.1:3000` 启动学习书实时预览，不自动打开浏览器。
- `make rust`：执行 Rust 格式化、Clippy、测试、doctest 和 rustdoc 检查。
- `make docs`：执行全部文档检查。
- `make ci`：执行当前 PR 所需的全部本地检查；随着项目增加代码检查而扩展。

本机预览时在浏览器访问 <http://127.0.0.1:3000>；通过 SSH 预览远程服务器上的学习书时，先在本机执行 `ssh -L 3000:127.0.0.1:3000 user@server`，再在服务器运行 `make book-preview`。
按 `Ctrl+C` 停止预览。

> **新机器首次配置：** 项目不绑定 Node.js 版本管理器。使用 `fnm` 时，将 `eval "$(fnm env --use-on-cd --shell zsh)"` 加入 `~/.zshrc`；重新进入 zsh 后，在仓库根目录依次运行 `fnm install` 和 `fnm use`。确认 Node.js 与 npm 版本符合 [`upstream/BASELINES.md`](upstream/BASELINES.md) 后，再运行 `make tools`。

首次使用或工具基线更新后，先按照 `.node-version` 切换 Node.js，再运行 `make tools`。
其他命令只使用仓库本地辅助工具，不依赖全局安装或固定的临时目录。

上游 checkout 位于 Git 忽略的 `upstream/checkouts/`，用于源码和历史研究，不属于 Cargo workspace。
`make upstream` 会核对官方 remote、tag 和 commit，并拒绝覆盖含有未提交变更的 checkout；标准库源码仍由 `rust-src` 提供。

## 工具链

根目录使用固定的 Rust `1.91.1`，并安装 `rust-src`、Rustfmt 和 Clippy。标准库源码由 `rust-src` 提供；完整的 `rust-lang/rust` 仓库不属于默认环境。

编译器 lowering 实验将在独立目录中使用按日期固定的 nightly，不改变根 workspace 的稳定工具链。

## 文档与协作约定

面向读者的项目文档统一使用中文，Rust 标识符、文件名和需要与上游对应的术语保留英文。Commit subject 与 Pull Request 标题统一使用英文，讨论背景、实现取舍和验证过程的正文可以使用中文。

分支、commit、Pull Request 和合并规则见 [`CONTRIBUTING.md`](CONTRIBUTING.md)。

知识证据、文档表达、代码设计和小步变更的要求见 [`docs/engineering-standards.md`](docs/engineering-standards.md)。

## 当前状态

项目处于仓库奠基阶段。后续工作和完成条件以 [`ROADMAP.md`](ROADMAP.md) 为准。
