# 计划：将 `dag_in_context` 的 `.egg` 规则增量迁移到 `eggplant` Rust 代码

## 目标描述
- 将 `dag_in_context/src/**/*.egg`（当前共 32 个）从 `include_str!` + egglog 文本程序，逐步迁移为依赖 `eggplant` 的 Rust 规则/DSL 定义。
- 迁移过程必须可增量推进：一次只改一个（或一小组强依赖）`.egg` 文件，并且保留旧 `.egg` 版本作为对照；通过 Cargo feature 在旧实现与新实现之间切换，便于定位差异。
- 保持现有默认行为不变：不开启新后端时，`make nits`、`make test` 与现有 CLI 功能应全部通过。

## 验收标准

以下每条验收标准都包含正向与反向测试用例，用于确定性验证与回归保护。

- AC-1：`eggplant` 版本与 `.egg` 版本一样通过所有测试（最终验收）
  - 正向测试（应 PASS）：
    - `make nits`（默认 feature 集）通过
    - `make test`（默认 feature 集）通过
    - `cd dag_in_context && cargo test --features eggplant`（或等价命令）通过
  - 反向测试（应 FAIL）：
    - 在未启用 `eggplant` feature 的情况下，任何尝试走 `eggplant` 代码路径都应编译期失败或清晰报错（而不是静默回退到 `.egg` 文本路径）

- AC-2：通过 feature 机制在旧 `.egg` 与新 `eggplant` 实现之间切换（便于小步迁移）
  - 正向测试（应 PASS）：
    - 默认仍使用现有 `.egg` 文本路径（例如 `prologue()`）
    - 启用 `eggplant` feature 后，走 `eggplant` Rust 路径（例如 `prologue_eggplant()` / `build_rules_eggplant()`），无需运行时开关
  - 反向测试（应 FAIL）：
    - 如果 `eggplant` 版本目前无法覆盖某个 `.egg` 文件的全部语义，不应悄悄“忽略规则导致语义变化”；要么显式标注并在对比/回归测试中暴露差异，要么把缺口记录到 `feature_request.md`

- AC-3：完成首个 `.egg` 文件的端到端迁移并建立可重复的对比测试（一次只迁移一个文件）
  - 正向测试（应 PASS）：
    - 选择“编译流程中最先被用到且尚未迁移”的 `.egg` 文件翻译为 `eggplant` Rust 模块（从 `dag_in_context/src/schema.egg` 开始，按下方顺序推进）
    - 新旧两种实现对同一固定输入（例如一个最小化的 tree program 或现有 tests 里的小样例）产出一致的关键观测结果（例如提取结果/打印的中间程序/某个 invariants），并用 `insta` 快照或等价断言固定下来
  - 反向测试（应 FAIL）：
    - 人为引入一个明显不等价的规则差异时，对比测试应稳定失败并给出可定位的 diff（避免“偶发/不确定”失败）

- AC-4：形成“迁移工作流”文档与检查清单，保证后续每次迁移都是小步、可回滚、可定位
  - 正向测试（应 PASS）：
    - 文档说明：如何按编译流程顺序选择下一目标、如何用 `eggplant` 翻译技能（见下方参考）把 `.egg` 转成 `#[eggplant::dsl]` / `#[eggplant::func]` / `MyTx::add_rule`、如何用 feature 切换、如何运行对比测试、遇到 unsupported 语法如何记录
    - 每次迁移都能只动一个文件（或一组强耦合文件）并通过 `make test`
  - 反向测试（应 FAIL）：
    - 若未更新对比测试/缺少迁移注册，CI/本地 `make test` 应能在明确的位置失败，而不是运行时默默行为改变

## 路径边界

### 上界（最大可接受范围）
- 逐步把 `dag_in_context/src/**/*.egg` 全部迁移为 `eggplant` Rust 代码，并最终移除对 `.egg` 文本 prologue 的依赖（或将其仅保留为调试对照）
- 对关键 ruleset 建立稳定的回归/对比测试，确保迁移过程中语义等价或差异可解释且被批准

### 下界（最小可接受范围）
- 完成 `eggplant` 集成 + 混合装配点 + 迁移 1 个 `.egg` 文件 + 1 个稳定的端到端对比测试
- 默认路径（不启用 eggplant）保持完全不变并通过 `make`

### 允许的选择
- 可以使用：
  - Cargo feature（如 `eggplant`）来隔离依赖与逐步启用
  - `eggplant` 翻译技能中建议的 Rust 形态：`#[eggplant::dsl]`、`#[eggplant::func]`、`MyTx::add_rule`、`RunConfig`
  - 现有 `insta` 测试框架做快照对比
- 不可以使用：
  - 一次性大迁移（把 32 个 `.egg` 一口气全改）
  - 为了“对齐 eggplant API”而重写优化/分析的高层语义（除非是明确的等价重构并有对比测试证明）
  - 在没有对比/回归测试的情况下改变默认优化结果

## 可行性提示与建议

> 注意：本节仅用于帮助理解与降低实现风险，不是强制性实现要求。

### 概念性方案
- 先做“装配层”而不是立刻全量翻译：把当前 `prologue()` 的拼接逻辑抽象为两条后端路径，并用 feature 切换：
  - 默认后端：现有 `.egg` 文本（保持 `include_str!`）
  - `eggplant` 后端：Rust 规则/DSL（逐文件迁移）
- 选择一个小 `.egg` 文件做样板迁移，验证：
  1) DSL/datatype 的映射方式是否可行
  2) ruleset/schedule 的映射方式是否可行
  3) 对比测试是否稳定
- 迁移顺序严格按编译流程中 `.egg` 被使用的顺序推进（见下方里程碑/清单）
- 遇到 eggplant/翻译技能暂不支持的语法：先跳过该部分，继续迁移其它部分，并把缺口记录到 `feature_request.md`

### 参考位置
- `.egg` 文件清单：`dag_in_context/src/**/*.egg`
- 文本 prologue 入口：`dag_in_context/src/lib.rs`（`prologue()`）
- 测试入口：`Makefile`（`make test` / `make nits`）
- eggplant 翻译技能（主参考）：`../eggplant_backup/docs/skills/egglog-to-eggplant-translation.md`
- eggplant 规则编写补充（可选）：`../eggplant_backup/docs/skills/eggplant-rule-authoring.md`

## 依赖与顺序

### 里程碑
1. 集成脚手架：为 `dag_in_context` 增加 `eggplant` feature，并用它在“旧 `.egg` 后端”和“新 eggplant 后端”之间切换
2. 翻译准备：阅读 `../eggplant_backup/docs/skills/egglog-to-eggplant-translation.md`，并约定每个 `.egg` 的翻译落点（模块/文件命名、注册方式、tests 约定）
3. 迁移顺序（严格按编译流程从前到后）：
   - `dag_in_context/src/schema.egg`
   - `dag_in_context/src/type_analysis.egg`
   - `dag_in_context/src/utility/util.egg`
   - `dag_in_context/src/utility/terms.egg`
   - `dag_in_context/src/optimizations/purity_analysis.egg`
   - `dag_in_context/src/utility/add_context.egg`
   - `dag_in_context/src/utility/context-prop.egg`
   - `dag_in_context/src/utility/term-subst.egg`
   - `dag_in_context/src/utility/context_of.egg`
   - `dag_in_context/src/utility/subst.egg`
   - `dag_in_context/src/utility/canonicalize.egg`
   - `dag_in_context/src/utility/expr_size.egg`
   - `dag_in_context/src/utility/drop_at.egg`
   - `dag_in_context/src/interval_analysis.egg`
   - `dag_in_context/src/optimizations/switch_rewrites.egg`
   - `dag_in_context/src/optimizations/select.egg`
   - `dag_in_context/src/optimizations/peepholes.egg`
   - `dag_in_context/src/optimizations/memory.egg`
   - `dag_in_context/src/optimizations/mem_simple.egg`
   - `dag_in_context/src/optimizations/loop_invariant.egg`
   - `dag_in_context/src/optimizations/loop_simplify.egg`
   - `dag_in_context/src/optimizations/loop_unroll.egg`
   - `dag_in_context/src/optimizations/swap_if.egg`
   - `dag_in_context/src/optimizations/rec_to_loop.egg`
   - `dag_in_context/src/optimizations/passthrough.egg`
   - `dag_in_context/src/optimizations/loop_strength_reduction.egg`
   - `dag_in_context/src/optimizations/ivt.egg`
   - `dag_in_context/src/optimizations/conditional_invariant_code_motion.egg`
   - `dag_in_context/src/optimizations/conditional_push_in.egg`
   - `dag_in_context/src/utility/debug-helper.egg`
   - `dag_in_context/src/optimizations/hackers_delight.egg`
   - `dag_in_context/src/optimizations/non_weakly_linear.egg`
4. 样板迁移：从顺序清单的第一个文件开始，保证“一次一文件 + 对比测试 + tests 全过”的节奏
5. Unsupported 处理：遇到暂不支持的语法，先跳过该部分并更新 `feature_request.md`（记录：原 `.egg` 片段、期望语义、阻塞原因、建议实现方向/最小复现）
6. 收尾：当 `eggplant` feature 下 `make test` 通过并稳定一段时间后，考虑切换默认后端与逐步移除 `.egg` 版本（以测试结果为准）

## 实施备注

### 代码风格要求
- 实现代码与注释中不要出现计划用语（例如 “AC-”、“里程碑”、“Phase/Step” 等）；这些只用于 `plan.md`
- 用领域语义命名（例如 `eggplant_backend`, `rules_registry`, `prologue_sources`），避免把流程术语写进代码里

--- Original Design Draft Start ---

进行本仓库的 `.egg` 重写，将没有编译报错支持的 `.egg` 文件转化成依赖 `eggplant` 库的 Rust 代码。只需要阅读 `../eggplant_backup` 里的 skills（尤其是 *Eggplant translation skill*），不需要通读 examples。为了方便 debug，你需要保证一次只重写一小部分，并且旧的 `.egg` 文件和 `eggplant` 驱动的 Rust 代码通过 feature 机制进行切换共同存在，方便定位错误。出现 unsupported 语法请先跳过这个文件的这个部分，并且编写 `./feature_request.md`。`.egg` 重写顺序应当是由编译流程从前到后用到的 `.egg` 文件顺序。

--- Original Design Draft End ---
