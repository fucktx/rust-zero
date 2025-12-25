# rest

`rest` 是本仓库的 Web 适配层与中间件集合，提供路由 DSL 与框架适配（如 axum/actix），目标是尽量保持零运行时抽象（zero-cost）。

## 技术栈

- Rust edition 2024
- feature 方式启用不同框架：`axum` / `actix`

## 使用说明（概览）

- **启用 axum 适配**：在依赖里开启 `features = ["axum"]`
- **启用 actix 适配**：在依赖里开启 `features = ["actix"]`

## 目录结构

```text
rest/
  Cargo.toml
  src/
    middleware.rs
    middleware/
```

## 许可证

Apache-2.0，详见仓库根目录 `LICENSE`。


