# Agent 工作流复审基线

本文档记录长任务工作流所依据的 OpenAI/Codex 与 Anthropic/Claude Code 资料、当前接受的结论和复审状态。
它不证明某项机制永远有效；每次复审都要重新检查当前产品行为、工程证据和本项目实际失败。

## 复审状态

| 字段 | 值 |
| --- | --- |
| `last-reviewed` | `2026-07-15` |
| `review-interval-days` | `14` |
| `review-timezone` | `Asia/Shanghai` |
| `next-review` | `2026-07-29` |

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
- [Long-running work](https://learn.chatgpt.com/docs/long-running-work) — 长任务与相关 task 的组织方式。
- [Projects, chats, and tasks](https://learn.chatgpt.com/docs/projects) — project、task 与恢复边界。
- [Memories](https://learn.chatgpt.com/docs/customization/memories) — 产品 memory 的用途和边界。
- [AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md) — 仓库持久指令的发现与作用域。
- [Skills](https://learn.chatgpt.com/docs/build-skills)、[Hooks](https://learn.chatgpt.com/docs/hooks)、[Scheduled tasks](https://learn.chatgpt.com/docs/automations) 与 [Git worktrees](https://learn.chatgpt.com/docs/environments/git-worktrees) — 当前可替换增强层的产品入口。
- [Using PLANS.md for multi-hour problem solving](https://developers.openai.com/cookbook/articles/codex_exec_plans) — 可恢复 ExecPlan 的官方模板来源。
- [Harness engineering](https://openai.com/index/harness-engineering/)（2026-02-11，Ryan Lopopolo）— 仓库知识、active/completed plans、反馈循环和持续清理的内部工程案例。
- [Symphony](https://openai.com/index/open-source-codex-orchestration-symphony/)（2026-04-27，Alex Kotliarskyi、Victor Zhu、Zach Brock）— 将控制面从会话转移到任务和交付物的编排案例。
- [Codex changelog](https://learn.chatgpt.com/docs/changelog) — 产品变化的时间入口。

### Anthropic 与 Claude Code

- [Claude Code best practices](https://code.claude.com/docs/en/best-practices) — 上下文管理、计划、验证和会话组织的当前建议。
- [Memory](https://code.claude.com/docs/en/memory) — `CLAUDE.md`、自动 memory 与作用域。
- [Context window](https://code.claude.com/docs/en/context-window) — compaction 和上下文恢复边界。
- [Sessions](https://code.claude.com/docs/en/sessions)、[Skills](https://code.claude.com/docs/en/skills)、[Hooks](https://code.claude.com/docs/en/hooks-guide) 与 [Common workflows](https://code.claude.com/docs/en/common-workflows) — 当前会话恢复和可替换增强层的产品入口。
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
14 天间隔本身属于可替换增强参数；复审应比较新发现数量、漏检风险、中断成本和实际改进收益，证据支持时可以调整周期。

## 当前结论

| 状态 | 结论 |
| --- | --- |
| 保留 | 仓库中的版本化事实、明确验收和 Git/测试核验组成稳定内核 |
| 保留 | 多会话或多 PR 复杂任务使用 active ExecPlan，新会话从事实源恢复 |
| 保留 | 一个 task 对应一个清晰结果，项目和交付物不绑定永久会话 |
| 保留 | 14 天复审门禁，自动提醒只负责发现到期，工作流变化必须经人工 review |
| 条件启用 | L3、unsafe、关键并发和重要性能结论使用独立 review task |
| 暂不启用 | 固定 context reset、固定 sprint、多 agent、hook 和自动 orchestrator |
| 禁止 | 把 product memory、transcript、本机 task list 或单个聊天摘要作为唯一事实源 |
| 禁止 | 定时任务依据新文章自动编辑、提交或合并工作流规范 |

## 复审记录

| 日期 | 覆盖范围 | 结论 |
| --- | --- | --- |
| 2026-07-15 | Codex 当前文档、OpenAI ExecPlan/Harness Engineering/Symphony，以及 Claude Code 当前文档和 Anthropic 2025–2026 长任务研究 | 建立稳定内核与可替换增强层；采用可恢复 ExecPlan 和 14 天深度复审，暂不引入固定 context reset、多 agent 或 hook |
