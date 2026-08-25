# Agent 工作流复审基线

本文档记录长任务工作流所依据的 OpenAI/Codex 与 Anthropic/Claude Code 资料、当前接受的结论和复审状态。
它不证明某项机制永远有效；每次复审都要重新检查当前产品行为、工程证据和本项目实际失败。

## 复审状态

| 字段 | 值 |
| --- | --- |
| `last-reviewed` | `2026-08-25` |
| `review-interval-days` | `30` |
| `review-timezone` | `Asia/Shanghai` |
| `next-review` | `2026-09-24` |

`next-review` 是便于人工阅读的派生值，权威计算输入是 `last-reviewed` 与 `review-interval-days`。
当天日期由 `review-timezone` 指定的 IANA 时区确定。
到达 `next-review` 当天即视为应复审。

## 来源边界

深度复审只接受以下来源：

1. OpenAI 或 Anthropic 当前官方产品文档、changelog、规范、官方仓库和官方工程文章；
2. 能够从一手页面核实作者身份及其在发表时与 OpenAI 或 Anthropic 关系的工程师文章、演讲或访谈，并且内容来自亲历实践。

“知名”不是可复现的证据条件，因此使用可核实身份、任职关系、一手经历和问题相关性代替名气判断。
个人来源只能支持作者陈述的设计动机、历史和经验，不能单独证明当前产品契约或普遍最佳实践。
聚合转载、搜索摘要、社区教程、营销比较和无法核实作者身份的内容不得改变本项目工作流；它们最多作为寻找一手来源的线索。

## 证据等级

| 等级 | 来源 | 可以支持什么 |
| --- | --- | --- |
| A | 当前官方产品文档、changelog 和规范 | 当前公开能力、配置和行为边界 |
| B | 公司官方发布的工程实验或案例 | 该版本、任务和环境下观察到的效果与取舍 |
| C | 身份与任职关系可核实的工程师一手资料 | 作者的设计动机、历史和实践经验 |
| D | 本项目可重复的恢复演练、失败记录和维护成本 | 某项机制是否适合 tiny-async-lab |

外部产品变化必须由 A 或 B 支持；某项机制是否适合本项目必须由 D 验证，本项目已经复现的失败也可以独立触发改进实验。
官方工程案例不是产品保证，个人经验也不能因作者声誉直接升级为规范。

## 固定入口

### OpenAI 与 Codex

- [Codex best practices](https://learn.chatgpt.com/guides/best-practices) — 任务描述、计划、验证和持久指导的当前建议。
- [Long-running work](https://learn.chatgpt.com/docs/long-running-work) — Goal mode 的会话内持续执行、验收条件和权限边界。
- [Projects and chats](https://learn.chatgpt.com/docs/projects) — project、chat、当前工作树与持久指导的边界。
- [Memories](https://learn.chatgpt.com/docs/customization/memories) — 产品 memory 的用途和边界。
- [AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md) — 仓库持久指令的发现与作用域。
- [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)、[Skills](https://learn.chatgpt.com/docs/build-skills)、[Hooks](https://learn.chatgpt.com/docs/hooks)、[Scheduled tasks](https://learn.chatgpt.com/docs/automations) 与 [Git worktrees](https://learn.chatgpt.com/docs/environments/git-worktrees) — 当前可替换增强层的产品入口。
- [Feature Maturity](https://learn.chatgpt.com/docs/feature-maturity) — 产品成熟度标签的当前含义。
- [Using PLANS.md for multi-hour problem solving](https://developers.openai.com/cookbook/articles/codex_exec_plans) — 可恢复 ExecPlan 的官方模板来源。
- [Harness engineering](https://openai.com/index/harness-engineering/)（2026-02-11，Ryan Lopopolo）— 仓库知识、active/completed plans、反馈循环和持续清理的内部工程案例。
- [Symphony](https://openai.com/index/open-source-codex-orchestration-symphony/)（2026-04-27，Alex Kotliarskyi、Victor Zhu、Zach Brock）— 将控制面从会话转移到任务和交付物的编排案例。
- [Codex changelog](https://learn.chatgpt.com/docs/changelog) — 产品变化的时间入口。

### Anthropic 与 Claude Code

- [Claude Code best practices](https://code.claude.com/docs/en/best-practices) — 上下文管理、计划、验证和会话组织的当前建议。
- [Memory](https://code.claude.com/docs/en/memory) — `CLAUDE.md`、自动 memory 与作用域。
- [Context window](https://code.claude.com/docs/en/context-window) — compaction 和上下文恢复边界。
- [Sessions](https://code.claude.com/docs/en/sessions)、[Goals](https://code.claude.com/docs/en/goal)、[Agents](https://code.claude.com/docs/en/agents)、[Skills](https://code.claude.com/docs/en/skills)、[Hooks](https://code.claude.com/docs/en/hooks-guide) 与 [Common workflows](https://code.claude.com/docs/en/common-workflows) — 当前会话恢复、持续执行和可替换增强层的产品入口。
- [Effective context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) — 有限上下文、渐进检索和结构化笔记的工程解释。
- [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)（2025-11-26，Justin Young）— 跨会话交接、增量进度和错误完成的实验。
- [Harness design for long-running application development](https://www.anthropic.com/engineering/harness-design-long-running-apps)（2026-03-24，Prithvi Rajasekaran）— 独立评估、成本和随模型升级删减脚手架的后续实验。
- [Managed Agents](https://www.anthropic.com/engineering/managed-agents)（2026-04-08，Lance Martin、Gabe Cemaj、Michael Cohen）— session、harness 与 sandbox 的稳定接口及可替换实现。
- [Claude Code changelog](https://code.claude.com/docs/en/changelog) 与 [What's new](https://code.claude.com/docs/en/whats-new) — 产品变化与周期摘要入口。

固定入口是复审起点，不表示每次只重新阅读这些页面。
changelog 或新文章命中 memory、compaction、task、plan、skill、hook、worktree、review、orchestration 或长任务恢复时，必须继续读取其直接链接的完整一手资料。

## 深度复审协议

复审不得只浏览标题、搜索摘要或 changelog 的一句概括。

1. 确定从上次 `last-reviewed` 到当前日期的时间窗口和本项目同期失败记录。
2. 检查两家的 changelog 和官方更新摘要，筛出影响稳定内核或增强层的变化。
3. 重新核对受影响的当前产品文档，确认功能名称、作用域、持久性、跨机器行为和已知限制。
4. 检索两家公司在时间窗口内发布的相关工程文章、官方仓库或规范，并沿直接引用追到必要的一手材料。
5. 使用个人资料前核实作者、发表时间、任职关系和亲历范围，并限制它能够支持的结论类型。
6. 为每项变化写明旧假设、新证据、本项目影响、迁移成本、验证办法和移除条件。
7. 对照本项目恢复演练和真实失败，分别给出 `保留`、`澄清`、`实验`、`替换` 或 `删除` 结论。
8. 一次只改变能够独立验证的最小机制；不因出现新产品能力就预先增加 skill、hook 或 orchestrator。
9. 使用独立 PR 更新工作流和本文件；即使结论是无需改变，也要记录覆盖范围和依据后更新复审日期。

复审应获得完成上述证据路径所需的时间和上下文预算，不设置迫使调研退化为快速摘要的人为小额度。
只有来源窗口已经覆盖、相关正文已经核验、结论边界已经说明且变更具有验证方案时，才算完成复审。
复审间隔本身属于可替换增强参数；复审应比较新发现数量、漏检风险、中断成本和实际改进收益，证据支持时可以调整周期。
已采用机制出现官方弃用或本项目复现恢复失败、范围漂移、错误完成等问题时应提前复审，不等待周期门禁到期。

## 当前结论

| 状态 | 结论 |
| --- | --- |
| 保留 | 仓库中的版本化事实、明确验收和 Git/测试核验组成稳定内核 |
| 保留 | 多会话或多 PR 复杂任务使用 active ExecPlan，新会话从事实源恢复 |
| 保留 | 一个 task 对应一个清晰结果，项目和交付物不绑定永久会话 |
| 保留 | 复审门禁和人工批准；自动提醒只负责发现到期，实际失败或已采用机制的官方弃用可以提前触发复审 |
| 实验 | 定期复审间隔从 14 天调整为 30 天，并按本轮记录的条件决定是否缩短 |
| 条件启用 | 产品 Goal 可以从当前里程碑抽取可验证终态来驱动同一会话，但完成状态仍须由仓库、Git 和检查核验，且 Goal 不承载跨会话唯一事实 |
| 条件启用 | L3、unsafe、关键并发和重要性能结论使用独立 review task |
| 暂不启用 | 固定 context reset、固定 sprint、默认多 agent、项目 hook 和自动 orchestrator |
| 禁止 | 把 product memory、transcript、本机 task list 或单个聊天摘要作为唯一事实源 |
| 禁止 | 定时任务依据新文章自动编辑、提交或合并工作流规范 |

## 2026-08-25 复审判断

本轮覆盖 `2026-07-15` 至 `2026-08-25`，并将外部产品变化与本项目在同一时段的恢复、评审和维护记录分开判断。

### 覆盖范围

- OpenAI 侧检查了窗口内的 [ChatGPT 与 Codex changelog](https://learn.chatgpt.com/docs/changelog)，重新核对当前 Best practices、Long-running work、Projects and chats、Memories、`AGENTS.md`、Subagents、Skills、Hooks、Scheduled tasks、Git worktrees、Feature Maturity 与 ExecPlan 页面，并读取窗口内的 [GPT-5.6 harness 效率说明](https://openai.com/index/gpt-5-6-frontier-intelligence-efficiency/)。
- Anthropic 侧检查了 Claude Code `2.1.211` 至 `2.1.241` 的 [changelog](https://code.claude.com/docs/en/changelog) 与窗口内 [What's new](https://code.claude.com/docs/en/whats-new)，重新核对当前 Best practices、Memory、Context window、Sessions、Goals、Agents、Skills、Hooks 与 Common workflows；[Anthropic Engineering](https://www.anthropic.com/engineering) 在本窗口内没有发布新的长任务 harness 研究，因此只复核既有三篇固定工程案例，没有把旧案例冒充本轮新证据。
- 本项目侧核对了 `master` 历史、[到期提醒 Issue #18](https://github.com/captainlee1024/tiny-async-lab/issues/18)、当前 active ExecPlan 的恢复与评审记录，以及窗口内由真实失败触发的工作流和学习文档规则变更。

### 变化与影响

| 变化面 | 旧假设 | 新证据 | 本项目影响与结论 |
| --- | --- | --- | --- |
| Goal | task、goal 和 transcript 只属于会话便利状态 | OpenAI 要求 outcome、constraints 与 verification，并明确 Goal 不扩大权限；Anthropic 的 evaluator 只判断对话中已经呈现的证据 | **澄清**：可以用当前里程碑的终态驱动一次持续执行，但不能替代 active ExecPlan、当前工作树或最终验证 |
| Memory 与 session 恢复 | memory 和 transcript 不能承载跨机器唯一事实 | Codex 当前文档把 memory 定位为可选、机器本地的生成状态；Claude Code 自动 memory 同样明确为机器本地且不跨 cloud environment | **保留**：继续把它们当定位和回忆层，不迁移仓库事实源 |
| Subagent、background session 与跨会话消息 | 不把多 agent 设为默认工作流 | 两家都显著增强了原生并行和后台执行；OpenAI 同时记录额外 token 与并行写冲突，Claude Code 窗口内仍持续修复 session、worktree、消息投递和恢复问题 | **保留**：没有项目 D 级失败要求默认启用，只在任务独立、权限明确且写入隔离时按需使用 |
| Hook、scheduled task 与 worktree | 只保留最小到期提醒，不让自动化修改规范 | 当前产品可以让 scheduled task 写入本地项目、让 hook 在生命周期内执行命令或工具，也都要求额外信任和隔离判断 | **保留**：现有 GitHub Issue reminder 已解决已观察需求，不新增项目 hook 或自动改写流程 |
| Context 管理 | 不采用固定 reset 或固定 sprint | 更大 context、Goal 和恢复能力降低了部分长任务摩擦，但两家当前资料仍强调 context bloat、选择性清理、渐进加载和独立调查上下文 | **保留**：继续由实际污染或纠偏失败触发新会话，不按固定轮次清空 |
| 持久指令体积 | `AGENTS.md` 应提供地图和必须自动生效的约束，而不是项目百科 | OpenAI 工程案例继续警告单个巨大指令文件会挤占上下文并腐化；Anthropic 当前文档建议单个 `CLAUDE.md` 以 200 行以内为目标；本项目 `AGENTS.md` 为 156 行，且尚无规则遗漏或冲突的 D 级失败 | **保留并观察**：本轮不凭通用案例删除项目特有约束；达到 200 行、出现重复规则或复现指令遗漏时，优先把可导航细节下沉到既有权威文档 |
| 产品弃用 | 固定入口可能随产品演进而变化 | `codex mcp-server` 已于 2026-08-24 弃用，但本项目没有采用该入口 | **保留并澄清触发器**：当前无迁移；未来已采用机制被弃用时提前复审 |
| 复审周期 | 14 天能够以合理成本捕获高频产品变化 | 本轮 41 天窗口包含大量产品变化但没有推翻稳定内核；Issue 在到期日准确创建并保持去重，而本项目产生实际收益的规则均由恢复或读者评审失败即时触发 | **替换**：改为 30 天，降低学习单元中途被日历门禁打断的频率，同时保留事件触发的提前复审 |

删减审计确认仓库没有项目级 skill、hook、orchestrator、固定 context reset、固定 sprint 或第二份 handoff 文件，现有自动组件只有到期检查脚本和创建去重 Issue 的 GitHub Actions reminder。
这些组件都仍对应已观察需求，因此本轮没有可删除的已安装组件；14 天复审参数是唯一证据支持替换的旧脚手架。

### 实际变更的验证与撤销条件

| 本轮变更 | 迁移成本 | 验证办法 | 撤销或替换条件 |
| --- | --- | --- | --- |
| 复审间隔从 14 天改为 30 天 | 只更新基线元数据和结论；检查脚本、daily reminder 与 Issue 去重机制不变 | 用 `--today 2026-09-23` 和 `--today 2026-09-24` 验证边界，并运行 `make ci` | 若后续复审发现一个已采用机制在 30 天窗口内失效并实际造成可复现损害，而 14 天门禁本可在任务开始前发现，则缩短周期 |
| 明确 Goal 的会话内边界 | 文档变化，不要求用户采用 Goal，也不复制 ExecPlan 到产品状态 | 下一次使用 Goal 时仍能仅凭仓库和 active ExecPlan 冷启动恢复，并在 Goal 报告完成后独立核对 Git 与检查结果 | 若产品契约改变，则更新术语和边界；若 Goal 被用作唯一事实源并导致恢复失败，则禁止该用法而不是扩充 Goal 脚手架 |
| 更新固定入口与本轮证据清单 | 文档变化，不引入新工具或配置 | 对修改文件运行外部链接检查并人工确认页面正文仍支持相邻结论 | 链接迁移、功能弃用或正文不再支持结论时替换入口，不保留仅为历史装饰的来源 |

### 本轮验证

- `node scripts/check-agent-workflow-review.mjs --validate` 返回 `0`，并报告当前日期 `2026-08-25`、下一次复审 `2026-09-24`。
- 使用 `--today 2026-09-23` 返回 `0`，使用 `--today 2026-09-24` 返回 `2`，证明新的到期边界包含到期当天。
- GitHub 当前只有一个带复审标记的未关闭 Issue，即 2026-07-29 创建的 Issue #18。
- `make ci` 通过 workspace Rust 检查、文档检查、mdBook test、Mermaid 渲染和复审元数据校验。
- 对两份修改文档运行在线链接检查时，39 个链接通过，三个 `openai.com/index/` 官方工程页面向命令行检查器返回 `403`；排除这三个已人工打开并核验正文的页面后，其余链接为 `0` 错误。

## 复审记录

| 日期 | 覆盖范围 | 结论 |
| --- | --- | --- |
| 2026-07-15 | Codex 当前文档、OpenAI ExecPlan/Harness Engineering/Symphony，以及 Claude Code 当前文档和 Anthropic 2025–2026 长任务研究 | 建立稳定内核与可替换增强层；采用可恢复 ExecPlan 和 14 天深度复审，暂不引入固定 context reset、多 agent 或 hook |
| 2026-08-25 | 两家 `2026-07-15` 至 `2026-08-25` changelog、受影响的当前产品文档、窗口内相关官方工程资料，以及 tiny-async-lab 的恢复、评审与提醒记录 | 稳定内核继续成立；澄清 Goal 只属于会话控制面，保留按需增强策略，把定期复审从 14 天调整为 30 天并增加事件触发的提前复审 |
