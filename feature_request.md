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

（暂空）

