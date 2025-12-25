# rust-zero

参考go-zero：包含基础库、Web 框架适配层，以及代码生成工具 `rsctl`。

## 技术栈

- Rust edition 2024
- Cargo（依赖管理与构建）
- 代码风格：`rustfmt` + `clippy`
- CI：GitHub Actions（规划/持续完善）

## 项目结构

```text
rust-zero/
  core/        # 基础核心库（被 rest 引用）
  rest/        # Web 适配层/中间件/DSL（axum/actix 等）
  rsctl/       # 代码生成器 workspace（cli/core/spec/parse/semantic/gen/utils + templates）
  LICENSE
  README.md
```

## 使用说明

- **rsctl（代码生成器）**：请看 `rsctl/README.md`
- **rest（Web 适配层）**：请看 `rest/` 下源码与示例（后续会补充文档）

## 许可证

本项目采用 Apache-2.0，详见 `LICENSE`。

## 版本记录

详见 `CHANGELOG.md`。
