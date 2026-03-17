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
cd dag_in_context && cargo test

# 新后端（eggplant feature）
cd dag_in_context && cargo test --features eggplant
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

## Eggplant DSL 翻译模板（最小）

迁移目标是把 `.egg` 的声明与规则翻译成 eggplant 的 Rust 形态（必要时允许混合 raw egglog 文本）。

### `datatype` / `constructor` → `#[eggplant::dsl]`

```rust
#[eggplant::dsl]
enum Expr {
    Const { n: i64 },
    Add { l: Expr, r: Expr },
}
```

> 目前 `schema.egg` 的 Types 段已开始迁移：`src/eggplant_backend/schema_dsl.rs` 定义 DSL，`src/eggplant_backend/schema.rs` 负责注入到 `schema::fragment()` 中。

### `function` → `#[eggplant::func]`

```rust
#[eggplant::func(output = i64, no_merge)]
struct fib {
    x: i64,
}
```

若 `.egg` 里出现自定义 `:merge`（例如用 `old/new` 表达式实现 max/min），当前 eggplant 宏不一定支持：
- 先把该声明保留为最小 raw egglog 文本
- 同步记录到仓库根目录 `feature_request.md`

### `rule` / `rewrite` → `MyTx::add_rule`

```rust
#[eggplant::pat_vars]
struct ConstFoldAdd {
    a: Const,
    b: Const,
    root: Add,
}

MyTx::add_rule(
    "const_fold_add",
    ruleset,
    || {
        let a = Const::query();
        let b = Const::query();
        let root = Add::query(&a, &b);
        ConstFoldAdd::new(a, b, root)
    },
    |ctx, pat| {
        let a = ctx.devalue(pat.a.n);
        let b = ctx.devalue(pat.b.n);
        let folded = ctx.insert_const(a + b);
        ctx.union(pat.root, folded);
    },
);
```

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

当前已包含一个“固定输入 + 提取结果”的语义对比测试（在 `--features eggplant` 下运行），作为后续逐步替换文本 diff 的基础。

## Backend Status Map

为了避免把“Rust 文件里的 egglog 字符串”误记为“已经原生 eggplant 化”，当前 backend 状态按下面规则记录：

- `eggplant-native`
  - 主语义通过 typed DSL / typed rules 表达，不依赖 raw egglog 文本作为唯一实现来源。
- `text-wrapped`
  - 仍然以 raw egglog 文本或 Rust 生成的 egglog 文本驱动生产路径；可以带有辅助性 Rust 代码，但不能据此声称已经完成原生迁移。
- `marker-only`
  - 目前只有 generated marker 或占位实现，尚未形成可比对的真实规则迁移。

当前分类：

- `eggplant-native`
  - `src/eggplant_backend/schema_dsl.rs`
- `text-wrapped`
  - `src/eggplant_backend/schema.rs`
  - `src/eggplant_backend/type_analysis.rs`
  - `src/eggplant_backend/util.rs`
  - `src/eggplant_backend/terms.rs`
  - `src/eggplant_backend/purity_analysis.rs`
  - `src/eggplant_backend/add_context.rs`
  - `src/eggplant_backend/context_of.rs`
  - `src/eggplant_backend/subst.rs`
  - `src/eggplant_backend/canonicalize.rs`
  - `src/eggplant_backend/expr_size.rs`
  - `src/eggplant_backend/drop_at.rs`
  - `src/eggplant_backend/interval_analysis.rs`
  - `src/eggplant_backend/switch_rewrites.rs`
  - `src/eggplant_backend/select.rs`
  - `src/eggplant_backend/peepholes.rs`
  - `src/eggplant_backend/memory.rs`
  - `src/eggplant_backend/mem_simple.rs`
  - `src/eggplant_backend/loop_invariant.rs`
  - `src/eggplant_backend/loop_unroll.rs`
  - `src/eggplant_backend/swap_if.rs`
  - `src/eggplant_backend/rec_to_loop.rs`
  - `src/eggplant_backend/passthrough.rs`
  - `src/eggplant_backend/loop_strength_reduction.rs`
  - `src/eggplant_backend/ivt.rs`
  - `src/eggplant_backend/conditional_invariant_code_motion.rs`
  - `src/eggplant_backend/conditional_push_in.rs`
  - `src/eggplant_backend/debug_helper.rs`
  - `src/eggplant_backend/hackers_delight.rs`
  - `src/eggplant_backend/non_weakly_linear.rs`
- `marker-only`
  - `src/eggplant_backend/context_prop.rs`
  - `src/eggplant_backend/loop_simplify.rs`

说明：

- `src/eggplant_backend/peepholes.rs` 现在包含一个 feature-gated typed-rule prototype 和直接 `run_ruleset` 测试，但当前生产 backend 仍通过 `fragment()` 文本接入，因此它暂时继续归类为 `text-wrapped`，直到主运行路径切换为 typed execution。

## 迁移完成后的后端决策

当前编译流程中的 `.egg` 文件已经全部迁移到 `src/eggplant_backend/` 的 Rust fragment。
迁移完成后的结论如下：

- 暂不切换默认 backend。
  - 默认路径继续保留为 `prologue_egglog_text()`。
  - `--features eggplant` 继续显式选择 `eggplant_backend::prologue()`。
- 暂不移除 `.egg` 文本 backend。
  - 该路径仍然是 AC-2 所要求的对照实现，也是定位回归最直接的 reference。
  - 现有 `prologue_egglog_text()` 与 `eggplant_backend::prologue()` 的 diff / semantic tests 仍依赖这条路径。
- 切换默认 backend 的前提：
  1. 在非 sandbox 环境或 CI 中补齐 AC-1 证据，完成 repo root 的 `make nits` 与 `make test`。
  2. 确认默认路径切换后不会削弱旧文本 backend 作为 reference 的作用；如果切换默认，仍需保留可选的文本 backend 入口用于比较与回归定位。
  3. 处理已知 cleanup 项（例如仍通过 `include_str!` 承载原 `.egg` 内容的 fragment 模块），确保最终状态明确且一致。

换句话说，当前推荐状态是：

- `default` = 文本 backend（稳定 reference）
- `--features eggplant` = Rust backend（迁移后的实现）

待 AC-1 在完整环境中可验证后，再单独发起“是否切换默认 backend / 是否移除文本 prologue 路径”的收尾变更。

## Unsupported 语法/能力的处理

遇到 eggplant/翻译技能暂不支持的语法或能力：

1. 不要静默丢弃语义（避免悄悄改变默认优化结果）
2. 先跳过该部分（保持可继续迁移其它部分）
3. 记录到仓库根目录的 `feature_request.md`

从 `dag_in_context/` 目录写入时，路径为：`../feature_request.md`。
