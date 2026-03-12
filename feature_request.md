# Feature Requests (Egglog → Eggplant Migration)

当把 `dag_in_context/src/**/*.egg` 迁移到 `eggplant` Rust 代码时，如果遇到当前翻译技能/eggplant API 暂不支持的语法或能力，请在这里追加记录，避免阻塞整条迁移链路。

## 记录模板（复制后填写）

### <简短标题>
- 发现日期：
- 影响文件：`dag_in_context/src/.../*.egg`
- 阻塞范围：该文件的哪一段/哪些规则（可给行号或小节名）

**原 `.egg` 片段（尽量最小化）**
```lisp
; paste minimal snippet here
```

**期望语义**
- 这段规则/声明在 `.egg` 版本里做了什么（用一句话 + 要点）
- 如果有：对应的测试/快照/命令（例如 `make test` 中哪个用例覆盖到）

**为什么当前无法翻译**
- 具体缺口：缺少的 eggplant API / 宏能力 / container / schedule 支持等
- 参考（如果有）：`../eggplant_backup/docs/skills/egglog-to-eggplant-translation.md` 哪一节相关

**建议的解决方向（可选）**
- 最小可行支持：
- 可能的替代实现（临时）：

**最小复现（可选但推荐）**
- 输入：
- 期望输出 / 不变量：
- 备注：

---

## 条目列表

### 支持 `function` 自定义 `:merge` / `:no-merge`（用于迁移 `LoopNumItersGuess`）
- 发现日期：2026-03-12
- 影响文件：`dag_in_context/src/schema.egg`
- 阻塞范围：`LoopNumItersGuess` 的 function 声明（文件末尾附近）

**原 `.egg` 片段（尽量最小化）**
```lisp
(function LoopNumItersGuess (Expr Expr) i64 :merge (max 1 (min old new)))
```

**期望语义**
- 声明一个函数表 `LoopNumItersGuess : (Expr, Expr) -> i64`，并使用自定义 merge 表达式把 `old/new` 合并成一个更保守的迭代次数猜测（`max 1 (min old new)`）。

**为什么当前无法翻译**
- crates.io `eggplant` 0.2.7 的 `#[eggplant::func]` 宏目前只支持 `output=...`，未暴露 `:merge` / `:no-merge` 的配置。
- `eggplant::wrap::EgglogTypeRegistry::collect_type_defs()` 目前对函数的 `merge` 固定输出为 `new`，无法表达上述 merge 逻辑。

**建议的解决方向（可选）**
- 最小可行支持：
  - 扩展 `#[eggplant::func]`：支持 `merge = "<egglog expr>"` 或 `no_merge`（并在 type registry 生成对应的 egglog `function` 声明）。
- 可能的替代实现（临时）：
  - 在迁移 `schema.egg` 时，先保留该条 `function` 的原始 egglog 文本（字符串）作为 fragment 的一部分，待 eggplant 支持补齐后再替换。
