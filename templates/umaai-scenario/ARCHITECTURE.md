# 架构概览（ARCHITECTURE）

本仓是 `<剧本名>` 的 Rust 五 crate workspace 模板，按 `path` 或 `git` 依赖引用引擎仓稳定面（`umaai-core` / `umasim` / `umaai`）。领域术语见 [GLOSSARY.md](GLOSSARY.md)，机制公式与落点见 [docs/mechanism.md](docs/mechanism.md)，通用引擎规范见 `<引擎仓>/docs/scenario/{repo_form,contracts}.md`。

## 1. Crate 拓扑与依赖边界

```text
scenario-cli   ──▶ scenario-ai ──┐
scenario-tools ──▶ scenario-sim ─┼─▶ scenario-core ──▶ <引擎仓> (umaai-core / umasim / umaai)
```

- **单向依赖，禁止反向依赖**：`scenario-core` 不得依赖任何业务 crate；`scenario-sim`（离线模拟）与 `scenario-ai`（在线决策）互不依赖且不得依赖 `scenario-cli`。
- **不碰引擎仓**：接入与维护剧本时引擎仓 `git diff` 应为空；确需新能力时回引擎仓按「带默认实现、非破坏」加钩子。
- **按需加依赖**：重依赖各归其位（终端依赖只在 `scenario-cli`），对应业务真正用上之前不向 `Cargo.toml` 提前引入依赖或 feature。

## 2. 核心数据流

- **离线模拟**：`data/`（`ScenarioData` + `ScenarioConfig`）→ `ScenarioGame`（`Stage` 阶段机驱动）→ `ScenarioView` 只读快照 → `ScenarioTrainer` 决策 → `scenario-sim`（基线裁判 / 批量跑局）/ `scenario-cli`（终端运行与 `manual` 核对面板）/ `scenario-tools`（测卡 / 复盘）。
- **在线决策**：帧文件解析 → 局面还原 → `Session` 决策循环 → `scenario-cli` 在线外壳或 Web / Android 宿主。

## 3. 各 Crate 与关键落点

### `crates/scenario-core`（宿主稳定面：剧本机制、训练员与核心装配）

- **入口与配置**：`lib.rs`（仅模块声明、常量与重导出，**不含实现**）、`data.rs`（剧本资产 schema / 校验）、`config.rs`（剧本配置段，覆盖层语义）、`testing.rs`（仓根与数据根定位件）。
- **局面与阶段（`src/game/`）**：
  - `mod.rs`：局面 `ScenarioGame`（字段 / `Deref` / 建局 / 快照 `ScenarioView`）+ 装配 `build_game`。
  - `stage.rs`：阶段机 `Stage`（含显式赋号的 `StageId` 投影）。
  - `action.rs`：`Action` + `Display` + `ActionEnum` + 动作结算入口。
  - `traits.rs`：引擎契约 `impl`（`Game` / `TrainingValueSource` / `FlatSearchGame`）。
- **训练员**：`trainer.rs`（样例训练员 `ScenarioTrainer`，真实剧本替换为手写 / 搜索训练员）。

### `crates/scenario-sim`（离线模拟 lib，骨架）

- 负责跑一局、批量跑局、固定种子基线裁判与测卡测量函数；只出 lib，终端入口由 `scenario-cli` 承载。

### `crates/scenario-ai`（在线决策 lib，骨架）

- 负责帧段解析、`Session` 接线与决策循环；只出 lib，入口由 `scenario-cli` 或宿主承载。

### `crates/scenario-cli`（终端入口，bin `scenario`）

- `main.rs`：命令行入口（切换 `scenario` / `random` / `manual` 训练员）与静态 `DATA_ROOT` 注入。
- `manual.rs`：无颜色手动训练员（人工核对机制；消费 `ScenarioView`，不引终端重依赖）。

### `crates/scenario-tools`（外围工具，骨架）

- 单 bin 多子命令，负责测卡与复盘等外围离线工具。

## 4. 常见改动快速定位

| 改动诉求 | 优先查看的文件 |
|---|---|
| 剧本资产 schema 与配置覆盖层 | `scenario-core/src/{data.rs, config.rs}` |
| 局面状态、建局装配与 `ScenarioView` | `scenario-core/src/game/mod.rs` |
| 阶段流转与 `StageId` 投影 | `scenario-core/src/game/stage.rs` |
| 动作定义与结算入口 | `scenario-core/src/game/action.rs` |
| 引擎契约接缝（`Game` / `FlatSearchGame`） | `scenario-core/src/game/traits.rs` |
| 训练员决策逻辑 | `scenario-core/src/trainer.rs` |
| 终端运行与手动核对面板 | `scenario-cli/src/{main.rs, manual.rs}` |

## 5. 架构硬约束

- **契约 trait impl 不可拆文件**：同一个 trait（如 `Game`）的 `impl` 必须整体落在同一文件（`traits.rs`）；可拆的只有 inherent `impl`。
- **`Deref` 底座不得与契约同名**：`BaseGame` 的公开方法不得与 `Game` 契约方法同名。
- **可见性分级**：crate 内跨模块调用标 `pub(crate)`；跨 crate 消费面标 `pub`（宿主稳定面 = `core` 的装配 / `ScenarioView` / 训练员）。
- **数据与配置注入**：lib 侧不写死路径（`ScenarioData::load` / `ScenarioConfig::load*` / `Data::init` 的根由调用方注入）；bin 数据根为代码级常量（`scenario-cli/src/main.rs` 的 `DATA_ROOT`），工作目录存在同名 `ai_config.toml` / `scenario_config.toml` 时字段级覆盖合并；未知键严格报错。
- **`StageId` 显式赋号**：对外投影枚举值只增不改，修改已有编号会导致旧种子失效。
- **呈现与决策分离**：`ScenarioView` 为纯只读值快照，不混入呈现文本，不进契约 trait；`manual.rs` 与前端只消费 `ScenarioView` 与静态表。
