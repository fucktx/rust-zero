# rsctl workspace

该目录是 `rsctl` 的独立 Rust workspace，用于实现“描述文件 -> 解析 -> 语义归一 -> Spec(IR) -> 代码生成 -> 写盘”的完整流水线。

## crates

- `cli`：命令行入口（薄层）
- `core`：流水线编排（对外主 API）
- `spec`：稳定的 IR/Spec 定义
- `parse`：输入解析（api/model/rpc）
- `semantic`：语义分析与归一化（parse -> spec）
- `gen`：代码生成（spec -> artifacts -> write）


