# Egglog `.egg` → Eggplant（Rust）迁移指南（增量）

目标：把 `src/**/*.egg` 逐个迁移到 `src/eggplant_backend/` 的 Rust 代码中，并通过 Cargo feature 在旧后端（`.egg` 文本）与新后端（Rust 生成/翻译）之间切换；最终两种后端通过同一套测试且行为一致。

## 后端切换

- 默认（不启用 feature）：走旧后端（`include_str!` 拼接 `.egg`）
  - 入口：`dag_in_context::prologue_egglog_text()`
- 启用 `eggplant` feature：走新后端（`src/eggplant_backend/`）
  - 入口：`dag_in_context::prologue()` → `eggplant_backend::prologue()`

本地常用命令：

```bash
# 旧后端（默认）
cargo insta test --release --unreferenced=reject

# 新后端（eggplant feature）
cargo insta test --release --unreferenced=reject --features eggplant
```

> 注意：本仓库需要系统依赖（README 提到的 LLVM 18、CBC）。未安装时可能出现 link 失败（例如 `CbcSolver`、`llvm-sys`）。

## 代码布局与注册约定

- `src/eggplant_backend/mod.rs`
  - 聚合器：`eggplant_backend::prologue() -> String`
  - 负责按顺序拼接各文件的 `fragment()`（或暂时继续 `include_str!` 未迁移的文件）
- 每个被迁移的 `.egg` 文件对应一个 Rust 模块：
  - 例：`src/schema.egg` → `src/eggplant_backend/schema.rs`
  - 模块必须提供：`pub(crate) fn fragment() -> String`

迁移过程要求：

- `.egg` 原文件保留不动，作为 reference 实现（便于 diff 与回归）。
- 新后端只替换“拼接来源”，不要一次性改动多个 `.egg`。

## 迁移顺序（按编译流程 / 现有 prologue 顺序）

以 `prologue_egglog_text()` 的拼接顺序为准；当前顺序从：

1. `src/schema.egg`
2. `src/type_analysis.egg`
3. `src/utility/util.egg`
4. `src/utility/terms.egg`
5. …（其余依次）

## 回归/对比测试

在 `--features eggplant` 下，应提供稳定的对比测试，确保迁移不引入语义漂移：

- 建议最小化：先对比 `prologue_egglog_text()` 与 `eggplant_backend::prologue()` 的差异（diff/snapshot），用于验证迁移的拼接正确性与“只改一个文件”原则。
- 随迁移推进，再升级为更语义层的对比（例如固定输入程序的提取结果/关键 invariants）。

## Unsupported 语法/能力的处理

遇到 eggplant/翻译技能暂不支持的语法或能力：

1. 不要静默丢弃语义（避免悄悄改变默认优化结果）
2. 先跳过该部分（保持可继续迁移其它部分）
3. 记录到仓库根目录的 `feature_request.md`

从 `dag_in_context/` 目录写入时，路径为：`../feature_request.md`。

