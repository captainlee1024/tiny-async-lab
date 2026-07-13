# 贡献与 Git 协作规范

本文档是本项目分支、commit、Pull Request（PR）与合并规则的唯一事实来源。规范的目标是让历史便于阅读、让 PR 便于审查，并让源码研究中的证据与实现决策能够长期追溯。文档、证据、代码设计和变更规模的质量要求见 [`docs/engineering-standards.md`](docs/engineering-standards.md)。

## 语言约定

- 面向读者的项目文档统一使用中文；代码标识符、文件名和需要与上游对应的术语保留英文。
- Commit subject 和 PR 标题统一使用英文。
- Commit body、PR 正文与 review 讨论默认使用中文；引用上游原文时保留必要的英文术语。

## 分支规范

`master` 是默认主分支。除仓库奠基期的微小维护外，有实质内容的变更使用短生命周期分支并通过 PR 合入。

分支名采用以下格式：

```text
<type>/<short-kebab-description>
```

例如：

```text
docs/future-poll-contract
feat/tiny-mio-epoll-poller
fix/runtime-duplicate-wakeup
```

分支名应简短、全小写，并描述目标而不是实现过程；不要加入姓名、日期或 `wip` 等无长期价值的信息。

## Commit 与 PR 标题格式

Commit subject 与 PR 标题共享同一套格式：

```text
<type>(<scope>): <summary>
<type>: <summary>
```

`scope` 是可选项。存在不兼容变更时，在冒号前添加 `!`：

```text
feat(runtime)!: change the task shutdown contract
```

格式细节：

- 使用英文半角冒号，冒号后恰好保留一个空格。
- `type` 和 `scope` 使用小写；`scope` 使用稳定的领域或组件名。
- `summary` 使用简洁的英文祈使表达，例如 `add`、`fix`、`document`、`remove`。
- `summary` 首词小写，末尾不加句号；避免 `update stuff`、`some fixes` 等模糊描述。
- 标题应尽量短，commit 首行以约 50 个字符为目标；信息清晰比机械截断更重要。

本项目采用与 Conventional Commits 兼容的括号 scope：使用 `feat(runtime): ...`，不使用 `feat: runtime: ...`。性能改进统一使用生态中更常见的 `perf`，不使用 `opt`。

## Type

| Type | 使用场景 |
| --- | --- |
| `feat` | 新增用户、学习者或其他组件可以观察到的能力 |
| `fix` | 修复错误行为 |
| `perf` | 不改变既有语义的性能改进 |
| `refactor` | 不增加能力、不修复错误的内部重构 |
| `test` | 新增或修改测试，不改变生产代码行为 |
| `docs` | 只修改文档或注释 |
| `build` | Cargo、构建脚本、工具链或打包相关变更 |
| `ci` | CI workflow 与自动化检查相关变更 |
| `chore` | 无法更准确归入其他类型的仓库维护工作 |
| `style` | 只改变格式、空白或排版，不改变语义 |
| `revert` | 回滚已有变更 |
| `backport` | 将已有变更移植到维护分支；建立维护分支后再使用 |

优先选择能够描述实际效果的最具体类型，不要把 `chore` 当作默认选项。仅有依赖版本变化时通常使用 `build(deps)`；如果升级依赖是为了获得明确的新能力或修复，应根据最终效果选择 `feat` 或 `fix`，并在正文说明依赖变化。

## Scope

`scope` 表示变更主要影响的稳定领域，而不是当前碰巧修改的文件名。常用 scope 包括但不限于：

- 学习对象：`std`、`compiler`、`tokio`、`mio`；
- 自研组件：`tiny-mio`、`runtime`、`scheduler`、`reactor`、`timer`、`net`；
- 实验与知识：`labs`、`practices`、`research`、`docs`；
- 仓库工程：`workspace`、`ci`、`deps`。

一个标题明显跨越多个领域时省略 `scope`，不要堆叠 scope。新增 scope 前先确认已有名称不能准确表达它。

## Commit 规范

每个 commit 应形成一个可解释的原子变更：它只服务于一个目的，在可行时能够独立编译和测试，也不会混入无关格式化或顺手重构。

只有 subject 已足以解释变更时才可以省略 body。需要正文时，subject 后空一行，正文重点说明：

- 为什么需要这项变更；
- 关键约束、不变量和没有选择的方案；
- 行为变化、兼容性影响或验证方式；
- 对应的上游 repository、tag/commit、path 和 symbol（源码研究类变更）。

正文可以使用中文，建议按约 72 个字符换行。关联问题使用 footer，例如：

```text
Refs: #12
BREAKING CHANGE: task shutdown now waits for owned resources
```

不要把已经推送、准备审查的 commit 命名为 `wip`、`tmp` 或 `fix typo`。开发期间可以保留本地检查点，但在需要保留 commit 历史的 PR 中，应在 review 前整理为可独立理解的提交。

Commit 示例：

```text
docs(std): explain the Future polling contract
feat(labs): add a manual Future experiment
feat(tiny-mio): implement an epoll-backed poller
fix(runtime): prevent duplicate task scheduling
perf(scheduler): reduce ready-queue contention
test(runtime): cover wake-after-cancel behavior
chore(workspace): record upstream baselines
```

## PR 范围与 review 预算

每个 PR 只完成一个可验证目标。开始实现前，在 PR 正文中写清楚“目标、不做、依据、验收”，把明显属于下一阶段的能力留给后续 PR，不提前建立尚未被当前需求证明的层次和扩展点。

首先根据影响边界确定变更等级；一个 PR 命中多个等级时按照最高等级处理：

| 等级 | 典型范围 | 要求 |
| --- | --- | --- |
| L1：局部实现 | 单个组件内部，不改变 public API、持久不变量或依赖方向 | 在 PR 中写明最小契约并完成局部验证 |
| L2：契约与生命周期 | public API、错误、取消、资源生命周期、并发不变量或可观察行为 | 先明确契约、状态和替代方案，与无关重构分开，补齐契约和失败路径测试 |
| L3：边界与架构 | 影响生产实现的新外部依赖、跨组件依赖方向、新平台或 unsafe abstraction、难以撤销的架构决定 | 实现前完成简短 ADR 或等价设计评审，只实现当前已决定边界的最小切片 |

手写 diff 以不超过约 400 行为目标。
超过 400 行时，应在 PR 中说明为何继续拆分会破坏理解或验证；超过 800 行原则上应拆分。
lockfile、生成文件和不可拆分的机械变更不计入预算，但应与逻辑变更隔离。
行数只是预警信号：即使行数很少，L2/L3 变更或同时改变多个职责仍然需要更严格的拆分和 review。

## Pull Request 规范

一个 PR 可以包含多个 commit，因此 PR 标题描述整项变更合入后的净效果，而不是复用最后一个 commit 的标题。选择最能代表主要效果的 `type`；变更跨越多个领域时省略 `scope`。

例如，一个标题为：

```text
feat(runtime): add basic task execution
```

的 PR 可以包含：

```text
feat(runtime): define task state transitions
feat(runtime): implement the RawWaker lifecycle
test(runtime): cover repeated wakeups
docs(runtime): document scheduling invariants
```

PR 正文使用中文，并至少覆盖以下信息；不适用的章节可以注明“不适用”，不要静默删除重要检查项。

```markdown
## 背景与目标

说明要解决的问题以及这次变更的边界。

变更等级：L1 / L2 / L3，并说明判断依据。

## 本次不做

明确留给后续 PR 的内容，防止实现过程中扩大范围。

## 主要变更

- 列出能够帮助 review 的关键变化。

## 验证

- [ ] 记录实际运行过的测试、lint 或实验命令及结果。

## 依据与设计取舍

记录关键不变量、替代方案；源码研究类 PR 还要记录上游版本和关键位置。

## 风险与限制

说明已知风险、平台限制、后续工作或“不适用”。

## 关联 Issue

Closes #123
```

提交 PR 前确认：

- PR 只解决一个清晰问题，没有混入无关修改；
- 已写明本次目标、非目标、变更等级、依据和可验证的验收条件；
- diff 规模适合有效 review；超过建议预算时已经说明不可继续拆分的理由；
- 标题符合本规范且能够作为最终 commit subject；
- 已自行检查 diff，没有调试输出、秘密信息或意外生成文件；
- 已运行与风险相称的格式化、lint、测试和实验；
- 技术结论具有匹配其类型的权威依据，待验证推测没有作为正式结论；
- 新增抽象具有当前必要性，并且已经检查可复用的 Rust 标准库能力；
- public API 已记录适用的错误、panic、取消和 safety 契约；
- unsafe 与并发代码已经写明不变量，并使用能够回答对应问题的工具验证；
- 行为、公共 API 或设计决策发生变化时，相关中文文档已经同步；
- 完成路线图事项时，`ROADMAP.md` 已在验证通过后更新；
- 尚未准备好 review 的 PR 标记为 Draft。

在 PR 正文中使用 `Closes #123`、`Fixes #123` 或 `Resolves #123` 关联并在合入时关闭 issue；只建立关联但不应关闭时，使用普通链接或说明文字。

## 本地验证

文档变更使用 `.github/workflows/docs.yml` 固定的工具版本，并在提交 PR 前运行：

```text
markdownlint-cli2
typos
lychee --offline --include-fragments=full --no-progress .
mdbook build docs
mdbook test docs
scripts/check-mermaid.sh
```

前三项分别检查 Markdown、拼写和仓库内链接；`mdbook build` 同时检查学习书结构与书内链接，最后一项实际渲染仓库中的 Mermaid 图。
外部链接受网络状态影响，不作为 PR 的必需检查，由定时 CI 完整验证。
首个 Rust package 加入后，再补充 `rustdoc` warning、doctest、格式化、lint 和测试命令。

## Review 与合并

- Review 按影响从高到低进行：先检查问题和范围是否正确，再检查 public contract、安全与并发不变量、取消和资源生命周期，然后检查架构与模块边界、测试和文档，最后处理命名、格式与措辞。
- Review 意见明确区分 blocking issue 与非阻塞的 `nit` 或后续改进，不用大量低风险意见淹没正确性和设计问题。
- 当前 PR 已达到清晰、正确且可维护的增量目标时，独立的非阻塞改进可以进入后续 PR；不能用“以后再做”推迟契约、安全或数据损坏问题。
- 对 review 意见通过新增 commit 还是重写现有 commit，取决于是否需要保留独立历史；不要为了“历史整洁”隐藏已经讨论过的重要设计变化。
- 默认建议使用 **Squash and merge**，使符合规范的 PR 标题成为 `master` 上的最终 commit subject。
- 只有当每个 commit 都独立有意义、通过验证，并且保留学习演进过程确有价值时，才保留多个 commit；此时每个 commit 都必须符合本规范。
- 合入后删除已完成的短生命周期分支。

## 规范依据

本规范主要参考并适配了以下公开约定：

- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)
- [Git：git-commit 文档](https://git-scm.com/docs/git-commit)
- [Pro Git：Contributing to a Project](https://git-scm.com/book/en/v2/Distributed-Git-Contributing-to-a-Project)
- [GitHub：About pull request merges](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/about-pull-request-merges)
- [GitHub：Using keywords in issues and pull requests](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/using-keywords-in-issues-and-pull-requests)
