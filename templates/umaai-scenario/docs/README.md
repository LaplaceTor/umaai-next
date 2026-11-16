# 剧本仓文档索引

> 本目录随仓复制：真实剧本仓在这里记录**本剧本**的机制、决策、验证与待办；通用接入规范见 `<引擎仓>/docs/scenario/`。

## 1. 本仓文档

| 文档 | 职责 | 触发条件（何时读/改） |
|---|---|---|
| [`../ARCHITECTURE.md`](../ARCHITECTURE.md) | Crate 拓扑、核心数据流、模块落点与架构硬约束 | 新增模块、调整依赖、定位代码或处理引擎接缝时 |
| [`../GLOSSARY.md`](../GLOSSARY.md) | 领域术语与禁用别名（单一事实源） | 核对或新增领域概念、命名时 |
| [`mechanism.md`](mechanism.md) | 当前生效的机制公式、流程、实现落点与测试对应 | 核对或修改剧本机制时 |
| [`adr/`](adr/) | 架构与核心算法范式决策记录（规范见 [`agents/domain.md`](agents/domain.md)） | 探索架构边界、评估或推翻架构/算法范式决策时 |
| [`agents/`](agents/) | 仓内 Agent 协作规范（领域文档 / 交接 / Issue / 调研等） | 执行对应 Agent 协作流程时 |
| [`changelog.md`](changelog.md) | 按日期升序的口径裁定与基线位移流水账（非日常查阅入口） | 拍板新口径/重录基线时追加，或按关键词追溯历史裁定原因 |
| [`tests_overview.md`](tests_overview.md) | 分 crate 测试面、基线快照与验收说明 | 增删测试、重录基线或排查验收时 |
| [`intake.md`](intake.md) | S0 情报核对、数据表字典与资产溯源映射 | S0 核对或重导出剧本资产时 |
| [`stubs.md`](stubs.md) | 数值占位（`STUB` / `TBD`）与待真机核对项登记 | 新增、排查或替换占位参数时 |
| [`todo.md`](todo.md) | 跨阶段收拢的真实未完成功能待办清单 | 认领待办或规划下一轮开发时 |
| [`handoff.md`](handoff.md) | 长会话交接文档（一次性运行时产物） | 需交接给下一位 Agent 时 |

## 2. 引擎通用文档（权威版在引擎仓）

| 文档 | 内容 |
|---|---|
| `<引擎仓>/docs/scenario/README.md` | 接入指南：建仓 / 依赖 / 纪律 / 常见坑 |
| `<引擎仓>/docs/scenario/repo_form.md` | 剧本仓形态：crate 分工 / 数据配置 / 引擎钩子 / 文档 / 测试 / 分面 / 验收 |
| `<引擎仓>/docs/scenario/dev_process.md` | 开发流程：S0 机制核对 → S1–S9 → 扩展轨 / 性能优化轨 |
| `<引擎仓>/docs/scenario/perf_plan.md` | 性能优化：工具选择 / 测量协议 / 手段 / 阶段 |
| `<引擎仓>/docs/scenario/contracts.md` | 契约逐项清单（`Game` / `Person` / `ActionEnum` / `Trainer` / `Frontend` / 装配 / `StageId` / View） |

- 开发期可直接读引擎仓原文；稳定期用 git `tag` 钉住引擎版本，或把上述文档按同一版本带入本仓。

## 3. 文档总则

- **真值优先级**：`contracts.md` > [`../GLOSSARY.md`](../GLOSSARY.md) > [`adr/`](adr/) > [`mechanism.md`](mechanism.md)。
- **单一事实源**：按 §1 职责分工落盘，不互相抄录；未确认机制标 `❓`，数值占位进 [`stubs.md`](stubs.md)。
- **相对路径**：文档内链接统一使用仓内相对路径或 `<引擎仓>/...` 占位，不写本机绝对路径。
