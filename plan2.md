# 计划：将 `eggplant_backend` 从文本兼容迁移层推进到原生 eggplant backend

## 目标描述

- 将当前 `dag_in_context/src/eggplant_backend` 从“以 `fragment() -> String` 拼接 egglog 文本为主”的实现，逐步推进为“以 eggplant typed API 为主”的 Rust-native backend。
- 下一阶段不追求一次性替换全部模块；重点是建立一条真实可运行、可测试、可扩展的 eggplant-native 样板路径。
- 保持当前默认行为不变：默认后端仍然是现有 text backend，`eggplant` feature 仍然是显式选择项。

## 验收标准

以下每条验收标准都包含正向与反向测试，用于确认实现不是“看起来更像 eggplant”，而是确实把 backend 的主语义从文本迁移到 typed eggplant API。

- AC-1：至少一个代表性优化模块不再以 raw egglog rule 字符串作为主实现
  - 正向测试（应 PASS）：
    - 选择一个代表性模块，优先从 `peepholes`、`switch_rewrites`、`select`、`passthrough` 中选择
    - 该模块的主要规则改为 `MyTx::add_rule(...)` 实现，而不是 `const XXX: &str = r#"...(rule ...)..."#`
    - 该模块保留等价性回归测试，并在 `cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant` 下通过
  - 反向测试（应 FAIL）：
    - 若该模块仍只是“把原 `.egg` 文本搬进 Rust `&str`”，则不应视为完成
    - 若 typed 规则实现与 text backend 结果不一致，对比测试应稳定失败

- AC-2：建立一条真正的 eggplant-native 最小运行路径
  - 正向测试（应 PASS）：
    - 存在一条最小执行路径，直接使用 eggplant typed API：
      - `#[eggplant::dsl]`
      - 必要时的 `#[eggplant::func]`
      - `MyTx::add_rule(...)`
      - `MyTx::run_ruleset(..., RunConfig::Once | Times | Sat)`
    - 这条路径在固定输入上得到与当前 text backend 一致的关键观测结果
  - 反向测试（应 FAIL）：
    - 如果实现仍然必须通过 `prologue()` 拼接的 egglog 文本才能运行，则不应视为完成此标准

- AC-3：核心声明层继续从“文本注入”推进到“Rust DSL 为真源”
  - 正向测试（应 PASS）：
    - 至少一个核心声明域以 Rust 侧定义为主源，而不是先维护一份旧文本模板再做注入
    - 优先候选：
      - `schema.rs` / `schema_dsl.rs`
      - `type_analysis.rs`
      - `util.rs`
    - 对应测试应继续验证声明可解析、可执行、且语义不退化
  - 反向测试（应 FAIL）：
    - 若只是把 egglog 文本从 `.egg` 移到 Rust `&str`，仍不应视为完成此标准
    - 若仍高度依赖脆弱的 `inject_*_section()` 文本切片才能成立，则应在计划跟踪中保留为未完成

- AC-4：明确区分“已原生化模块”和“仍为文本兼容模块”
  - 正向测试（应 PASS）：
    - 文档或测试中明确标识：
      - 哪些模块是 eggplant-native
      - 哪些模块仍是 raw egglog text 包装
      - 哪些模块仍是 marker-only 占位
    - 后续新增迁移不应模糊这三类状态
  - 反向测试（应 FAIL）：
    - 若继续把纯字符串包装模块标注成“已完成 eggplant 迁移”，则视为未满足此标准

- AC-5：默认 backend 行为保持不变，`eggplant` feature 继续作为独立验证路径
  - 正向测试（应 PASS）：
    - `cd dag_in_context && cargo test` 继续通过
    - `cd dag_in_context && cargo test --features eggplant` 继续通过
    - 根仓库默认验证路径不因本阶段工作而被强制切换到 eggplant-native backend
  - 反向测试（应 FAIL）：
    - 如果本阶段工作导致默认 text backend 回归失败，或未显式开启 `eggplant` feature 就走到新实现路径，则视为未满足此标准

## 路径边界

### 上界（最大可接受范围）

- 至少完成一个规则模块的原生 eggplant 化
- 至少建立一条不依赖 `prologue()` 文本拼接的最小运行路径
- 至少让一个核心声明域从“文本注入”转向“Rust 定义为真源”
- 保持现有 feature 切换与测试通过

### 下界（最小可接受范围）

- 选定一个代表性模块并证明其主要规则不再依赖 raw egglog rule 字符串
- 提供一个稳定测试证明 typed rule 路径可执行且与 text backend 对齐
- 不破坏默认 backend 和现有 `eggplant` feature 测试

### 允许的选择

- 可以使用：
  - `#[eggplant::dsl]`
  - `#[eggplant::func]`
  - `MyTx::add_rule(...)`
  - `MyTx::run_ruleset(..., RunConfig::...)`
  - 现有对照测试、语义等价测试、快照测试
  - 小步并存策略：旧 text backend 与新 typed backend 共存
- 不可以使用：
  - 把 `.egg` 内容机械挪到 Rust `&str` 后标记为“已原生 eggplant 化”
  - 一次性全面替换所有模块而不建立样板和测试模板
  - 在默认 backend 上直接切换到新实现

## 可行性提示与建议

> 注意：本节用于帮助理解与降低实现风险，不是强制实现要求。

### 概念性方案

- 第一步不要先全面处理 schema，也不要先全面删掉 `fragment() -> String`
- 先挑一个代表性规则模块，把它完整迁成 typed eggplant 规则：
  - pattern closure
  - action closure
  - typed `ctx` 操作
  - `run_ruleset` 执行
- 建立最小运行路径后，再反向推动声明层和调度层重构

### 推荐首个样板模块

建议优先级：

1. `peepholes`
2. `switch_rewrites`
3. `select`
4. `passthrough`

原因：

- 这些模块相对集中
- 规则语义较清晰
- 对照测试现成
- 比 `memory`、`type_analysis`、`schema` 全量重构更适合作为原生 eggplant 样板

### 参考位置

- `dag_in_context/src/eggplant_backend/mod.rs`
- `dag_in_context/src/eggplant_backend/schema.rs`
- `dag_in_context/src/eggplant_backend/schema_dsl.rs`
- `dag_in_context/src/eggplant_backend/type_analysis.rs`
- `dag_in_context/src/eggplant_backend/util.rs`
- `dag_in_context/src/lib.rs`
- `/Users/mineralsteins/Repos/egg_related/eggplant_backup/README.md`
- `/Users/mineralsteins/.codex/skills/egglog-to-eggplant-translation/SKILL.md`
- `/Users/mineralsteins/.codex/skills/eggplant-rule-authoring/SKILL.md`

## 依赖与顺序

### 里程碑

1. 确认迁移分类
   - 明确列出：
     - eggplant-native 模块
     - text-wrapped 模块
     - marker-only 模块

2. 建立首个 typed rule 样板
   - 选择一个代表性模块
   - 将其主要规则迁成 `MyTx::add_rule(...)`
   - 为该模块补齐对照测试

3. 建立最小原生运行路径
   - 用最小 DSL + 最小 ruleset + `run_ruleset` 跑通
   - 固定输入验证与 text backend 一致

4. 推进声明层主源迁移
   - 优先从 `schema` / `type_analysis` / `util` 中选择一个最值得推进的点
   - 减少对文本注入和切片逻辑的依赖

5. 扩展到第二个和第三个模块
   - 在样板稳定后，再迁第二个模块
   - 复用已建立的测试模板和运行模板

## 实施备注

### 代码风格要求

- 实现代码与注释中不要出现计划术语，如 `AC-`、`Milestone`、`Phase`、`Step`
- 这些术语只属于计划文档，不属于实现代码
- 代码命名应使用领域语义，例如：
  - `typed_ruleset`
  - `eggplant_native_runner`
  - `text_backend_compat`
  - `schema_registry`

--- Original Design Draft Start ---

# 计划草案 2：把当前 `eggplant_backend` 从“文本兼容迁移层”推进到“原生 eggplant backend”

当前仓库已经完成了大量 `eggplant` 迁移工作，并且默认后端与 `eggplant` feature 后端都可测试通过；但当前 `eggplant_backend` 仍主要是以 Rust 文件组织的 egglog 文本装配层，而不是以 `#[eggplant::dsl]`、`#[eggplant::func]`、`MyTx::add_rule(...)`、`MyTx::run_ruleset(..., RunConfig::...)` 为主的 typed eggplant-native backend。下一阶段的主要目标不是继续机械搬运 `.egg` 内容，而是选定一个代表性 ruleset 做成真正的 typed eggplant 样板，建立一条最小原生运行路径，并推进 schema / function 声明从“文本注入”转向“Rust DSL 为真源”。保持默认 backend 不变，`eggplant` feature 继续作为独立验证路径。

--- Original Design Draft End ---
