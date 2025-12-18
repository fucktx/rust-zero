# rsctl workspace

该目录是 `rsctl` 的独立 Rust workspace，用于实现“描述文件 -> 解析 -> 语义归一 -> Spec(IR) -> 代码生成 -> 写盘”的完整流水线。

## 总览（当前目录结构）

本仓库采用 **workspace 扁平组织**（不使用 `crates/` 目录）：

```text
rsctl/
  Cargo.toml                       # [workspace] + [workspace.dependencies]
  README.md

  cli/                             # 二进制：命令行入口（薄层）
  core/                            # 编排：Parse -> Semantic -> Gen -> Write（对外主 API）
  spec/                            # 稳定 IR：统一 Spec（最重要的稳定层）
  parse/                           # 输入解析：文本/文件 -> AST/中间结构
  semantic/                        # 语义分析：AST -> Spec（归一化/校验/默认值）
  gen/                             # 生成：Spec -> Artifacts（待写文件集合）

  templates/                       # 外置模板根（可被 CLI 的 --templates 覆盖）
    api/
      rs/
    model/
    rpc/
  examples/                        # 示例输入/示例工程（可选）
  docs/                            # 设计文档/规范（可选）
```

## 各 crate 职责（为什么要这样分层）

- `cli`：命令行入口（薄层）
- `core`：流水线编排（对外主 API）
- `spec`：稳定的 IR/Spec 定义
- `parse`：输入解析（api/model/rpc）
- `semantic`：语义分析与归一化（parse -> spec）
- `gen`：代码生成（spec -> artifacts -> write）

它们的调用关系（理想流水线）：

```text
输入文件/文本
  -> parse（解析）
  -> semantic（语义检查 + 归一化）
  -> spec（稳定 IR）
  -> gen（生成 artifacts）
  -> write（落盘策略：覆盖/跳过/合并）
```

## “完整形态”目录结构参考（带注释）

下面这份结构是你最初贴出来的“全链路 + 分层职责”的 **落盘版**（已按本仓库的实际约束做了两点调整：**扁平 crates**、**crate 不加 `rsctl-` 前缀**）。

```text
rsctl/                                              # workspace 根目录
  Cargo.toml                                        # [workspace]：成员管理/统一依赖版本
  README.md                                         # 总览说明/快速开始/目录解释（就是本文件）

  cli/                                              # 二进制 crate：命令行入口（最终产物）
    Cargo.toml
    src/
      main.rs                                       # 入口：解析参数 -> 调用 core
      cli.rs                                        # clap/子命令定义与分发（薄层）
      cli/
        commands/                                   # api/model/rpc 等子命令（gen/validate/format…）
        args/                                       # 参数结构/校验/配置加载

  core/                                             # 核心库 crate：流水线编排 + 公共能力（对外主 API）
    Cargo.toml
    src/
      lib.rs                                        # 对外接口：run_xxx/pipeline
      pipeline.rs                                   # Parse->Semantic->Spec->Gen->Write 编排（入口/调度）
      pipeline/
        api.rs                                      # api 流水线编排（可选：按域拆分）
        model.rs
        rpc.rs
      common.rs                                     # 通用：error/naming/fs/config/logging 等

  spec/                                             # IR/Spec crate：统一代码生成模型（稳定层）
    Cargo.toml
    src/
      lib.rs
      api.rs                                        # API Spec 根模块
      api/
        types.rs                                    # 类型系统：Request/Response/DTO/Enum…
        routes.rs                                   # 路由：service/route/method/path…
      model.rs                                      # Model Spec 根模块
      model/
        schema.rs                                   # 表/字段/索引/关系等
        types.rs                                    # 统一类型系统（吸收不同数据库的类型差异）
      rpc.rs                                        # RPC Spec 根模块
      rpc/
        service.rs                                  # service/method 定义
        message.rs                                  # message/field 定义

  parse/                                            # Parse crate：输入解析（文本/文件 -> AST/中间结构）
    Cargo.toml
    src/
      lib.rs
      api.rs                                        # API 输入解析入口（DSL/OpenAPI 等）
      api/
        dsl.rs                                      # 自定义 .api DSL 解析
        openapi.rs                                  # OpenAPI 输入解析
      model.rs                                      # Model 输入解析入口（DDL/Schema/Introspect…）
      model/
        mysql.rs                                    # MySQL 方言解析/导入点
        pg.rs                                       # PostgreSQL 方言解析/导入点
      rpc.rs                                        # RPC 输入解析入口（proto/thrift 等）
      rpc/
        proto.rs
        thrift.rs

  semantic/                                         # Semantic crate：语义分析与归一化（AST -> Spec）
    Cargo.toml
    src/
      lib.rs
      api.rs                                        # API 语义检查/默认值/命名归一/引用解析
      model.rs                                      # Model 语义检查/类型归一/约束归一（吸收大部分方言差异）
      rpc.rs                                        # RPC 语义检查/引用解析/包名归一等

  gen/                                              # Gen crate：生成层（Spec -> Artifacts/待写文件集合）
    Cargo.toml
    src/
      lib.rs
      api.rs                                        # API 生成器集合（领域优先；语言/框架放更深层）
      api/
        rs_axum.rs                                  # Rust/Axum 生成器（示例）
        go_zero.rs                                  # Go/go-zero 风格生成器（预留）
      model.rs                                      # Model 生成器集合
      model/
        mysql.rs                                    # 例如 sqlx/gorm/DDL 等输出策略
        pg.rs
      rpc.rs                                        # RPC 生成器集合
      rpc/
        grpc.rs
        thrift.rs
      template.rs                                   # 模板渲染封装（helpers/filters/loader）
      write.rs                                      # 写文件与增量更新策略（覆盖/跳过/合并/标记块）
      write/
        plan.rs                                     # 变更计划（新增/更新/删除/差异摘要）
        strategy.rs                                 # overwrite/merge/skip 等策略

  templates/                                        # 外置模板（可选；也可迁入 gen 内并内嵌）
    api/                                            # API 模板根
      rs/                                           # Rust API 模板（当前已有）
      go/                                           # Go API 模板（预留）
    model/                                          # Model 模板根（后续细分 mysql/pg/redis）
    rpc/                                            # RPC 模板根
      grpc/
      thrift/

  examples/                                         # 示例输入/示例工程（可选）
  docs/                                             # 设计文档/规范/模板变量说明（可选）
```

## 规划的细化目录（后续按功能补齐）

下面是“推荐的内部细分结构”（你下次回来能快速定位代码应放在哪里）。注意：这些子目录未必现在就全部存在，按实现逐步补齐即可。

### `cli/`（命令行：薄层）

- **目标**：只做参数解析/校验/分发，不承载业务逻辑；业务逻辑由 `core` 统一编排。
- **建议结构**：

```text
cli/
  Cargo.toml
  src/
    main.rs                        # 入口：解析参数 -> 调用 core
    cli.rs                         # 顶层命令路由（clap）
    cli/
      commands/                    # api/model/rpc 等子命令
      args/                        # 参数结构/校验/配置加载
```

### `core/`（流水线编排：对外主 API）

- **目标**：把各阶段串起来（Parse -> Semantic -> Spec -> Gen -> Write），并支持“只跑到某个阶段”便于调试。
- **建议结构**：

```text
core/
  Cargo.toml
  src/
    lib.rs                         # 对外接口：run_xxx/pipeline
    pipeline.rs                    # 分阶段执行编排
    pipeline/
      api.rs
      model.rs
      rpc.rs
    common.rs                      # error/naming/path/fs/config/logging 等（如需也可下沉到 spec）
```

### `spec/`（稳定 IR：统一生成模型）

- **目标**：生成器唯一依赖的“稳定数据结构”。新增输入方言/生成目标时，尽量只改 `parse/semantic/gen`，避免频繁动 `spec`。
- **建议结构**：

```text
spec/
  Cargo.toml
  src/
    lib.rs
    api.rs
    api/
      types.rs
      routes.rs
    model.rs
    model/
      schema.rs                    # 表/字段/索引/关系等
      types.rs                     # 统一类型系统（i32/string/datetime/json…）
    rpc.rs
    rpc/
      service.rs
      message.rs
```

### `parse/`（输入解析：文件/文本 -> AST/中间结构）

- **目标**：支持多输入格式/方言。方言差异尽量在 `semantic` 吸收，`parse` 只做“能读出来”。
- **建议结构**：

```text
parse/
  Cargo.toml
  src/
    lib.rs
    api.rs                         # .api DSL / OpenAPI 等入口
    api/
      dsl.rs
      openapi.rs
    model.rs                       # ddl/反射/连接数据库导出（可选）
    model/
      mysql.rs
      pg.rs
    rpc.rs                         # proto/thrift 等入口
    rpc/
      proto.rs
      thrift.rs
```

### `semantic/`（语义分析：AST -> Spec）

- **目标**：默认值、命名归一、引用解析、类型归一、跨文件/跨模块校验等；把“方言差异”收敛成统一 Spec。
- **建议结构**：

```text
semantic/
  Cargo.toml
  src/
    lib.rs
    api.rs
    model.rs
    rpc.rs
```

### `gen/`（生成：Spec -> Artifacts -> Write）

- **目标**：按领域拆生成器（api/model/rpc），语言/框架/协议放更深层；写盘策略独立出来。
- **建议结构**：

```text
gen/
  Cargo.toml
  src/
    lib.rs
    api.rs
    api/
      rs_axum.rs                   # Rust Axum 风格
      go_zero.rs                   # Go go-zero 风格
    model.rs
    model/
      mysql.rs
      pg.rs
    rpc.rs
    rpc/
      grpc.rs
      thrift.rs
    template.rs                    # 模板渲染封装（helpers/filters/loader）
    write.rs                       # 写出策略入口（覆盖/跳过/合并/标记块）
    write/
      plan.rs                      # 变更计划（diff/新增/更新）
      strategy.rs                  # overwrite/merge/skip
```

## templates（外置模板）

模板目前已按分层归档（与你的目标一致：api/model/rpc）：

- `templates/api/rs/`：Rust API（axum）模板（已迁移完成）
- `templates/model/`：模型层模板（后续会细分 mysql/pg 等）
- `templates/rpc/grpc/`：RPC 层模板（后续可加 thrift 等）


