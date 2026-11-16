# umaai-scenario（空剧本仓模板）

`umaai-next` 引擎仓的剧本五 crate workspace 模板：复制本目录到独立剧本仓，将 `scenario` 统一替换为剧本名（如 `ramen`）后开工。通用规范与契约见引擎仓 `<引擎仓>/docs/scenario/{README,repo_form,dev_process,perf_plan,contracts}.md`。

## 1. 上手四步

1. **复制 + 改名**：复制整个目录到剧本仓，把 5 个 package 名与 bin 名里的 `scenario` 统一替换为剧本名（如 `ramen-core` / `ramen-cli` / bin `ramen`），同步改 `use scenario_core::…` 与 `Cargo.toml` 引用。保留根 `[workspace]` 表使剧本仓可独立构建。
2. **改依赖路径**：开发期 `path` 指向引擎仓；稳定期换 git `tag`；下游宿主（Web / Android）加 `default-features = false`（见下「依赖三档」）。
3. **补数据**：把引擎仓 `gamedata/` 的六张静态表（`cardDB.json` / `umaDB.json` / `events.json` / `text_data_dict.json` / `constants.json` / `manifest.json`）与 `ai_config.toml` 拷入本仓 `data/`；剧本仓自此自持数据、拷仓即跑。引擎数据升级时重拷并重录基线。
4. **填留白**：`data.rs` / `config.rs` 换成自己的资产与配置段；`game/{stage,action,mod,traits}.rs` 换成自己的阶段、动作、结算与契约实现；`trainer.rs` 换成自己的手写 / 搜索训练员。强依赖剧本的形态**不在模板预置**，先在 [docs/todo.md](docs/todo.md) 或 Issue 列为待确认项，机制核对后再实现。

## 2. 依赖三档

```toml
# 1. 开发期：并列目录（引擎仓与剧本仓并列）
umaai-core = { path = "../umaai-next/crates/umaai-core" }
umasim     = { path = "../umaai-next/crates/umasim" }
umaai      = { path = "../umaai-next/crates/umaai" }
umaai-cli  = { path = "../umaai-next/crates/umaai-cli" }   # 仅终端入口需要（样例面）

# 2. 稳定期：同一 repo、同一 tag（cargo 只解析一份检出）
# umaai = { git = "<engine repo>", tag = "engine-v0.1.0" }

# 3. 下游（Web / Android）：自喂帧、无文件监听，并剪掉并行后端
# umaai = { git = "<engine repo>", tag = "engine-v0.1.0", default-features = false }
```

## 3. 运行与验收

```powershell
# 数据根是代码级常量 data/（crates/scenario-cli/src/main.rs 的 DATA_ROOT）
cargo run --release --bin scenario --            # 样例训练员
cargo run --release --bin scenario -- random     # 引擎随机基线（对照用）
cargo run --release --bin scenario -- manual     # 无颜色人工选择（机制核对用）

cargo fmt --all --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --release --offline
```

- `cargo check` / `clippy` / `test` 通过（**不改引擎任何一行**）；
- 接入新剧本时引擎仓 `git diff` 为空——确需新能力时回引擎仓按「带默认实现、非破坏」加钩子；
- 三条随机轴（局面 / 事件 / 策略）+ 决策流只喂 `Trainer`；阶段自持、对外只投影 `StageId`；同一 `master` 下两次运行 / 两支分支逐位一致；
- 剧本资源给前端走 `ScenarioView`，不进契约 trait。

## 4. 深入文档

| 文档 | 用途 |
|---|---|
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Crate 拓扑、模块落点与架构约束 |
| [`GLOSSARY.md`](GLOSSARY.md) | 领域术语与禁用别名 |
| [`docs/README.md`](docs/README.md) | 剧本机制、ADR、Agent 规范、测试与交接文档索引 |
