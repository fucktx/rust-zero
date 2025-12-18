# Rsctl templates 目录
_Exported on 12/16/2025 at 23:44:20 GMT+8 from Cursor (2.2.23)_

---

**User**

阅读rsctl下面的templates目录

---

**Cursor**

我已经阅读了 `rsctl/templates` 目录下的所有文件。这是一个代码脚手架模板目录，用于生成 Rust API 服务代码。以下是各文件的概述：

## 目录结构

```
rsctl/templates/rs/
├── api/           # API 服务模板
│   ├── api.txt    # 简单说明文件 "api层"
│   ├── config.tpl # 配置结构体模板
│   ├── etc.tpl    # 配置文件模板 (YAML格式)
│   ├── handler.tpl # HTTP 处理器模板
│   ├── logic.tpl  # 业务逻辑层模板
│   ├── main.tpl   # 主入口模板
│   ├── middleware.tpl # 中间件模板
│   ├── svc.tpl    # 服务上下文模板
│   └── types.tpl  # 类型定义模板
├── grpc/
│   └── demo       # 空文件 (占位符)
└── model/
    └── demo       # 空文件 (占位符)
```

## 各模板文件详解

### 1. `config.tpl` - 配置模块
定义了 `Config` 结构体，使用 serde 反序列化，内嵌 `RestConf` 公共配置。

### 2. `etc.tpl` - 配置文件
YAML 格式的配置文件模板，包含服务名、主机和端口。

### 3. `handler.tpl` - HTTP 处理器
基于 Axum 框架的处理器模板，支持：
- 从 `State` 提取服务上下文
- 可选的 JSON 请求体解析
- 调用 logic 层处理业务

### 4. `logic.tpl` - 业务逻辑层
业务逻辑实现模板，包含：
- 服务上下文引用
- 使用 `tracing::instrument` 进行链路追踪
- 异步方法签名

### 5. `main.tpl` - 主入口
完整的服务启动模板，包含：
- 配置文件加载（支持环境变量覆盖）
- Axum 服务器启动
- 优雅关闭信号处理（Ctrl+C 和 Unix SIGTERM）

### 6. `middleware.tpl` - 中间件
Tower 风格的中间件模板，包含：
- Layer 和 Service trait 实现
- 前置/后置处理逻辑占位

### 7. `svc.tpl` - 服务上下文
`ServiceContext` 结构体模板，用于存储配置和中间件实例。

### 8. `types.tpl` - 类型定义
自动生成的请求/响应类型定义，支持时间类型（chrono）。

---

这套模板借鉴了 go-zero 框架的设计理念，为 Rust 生成类似的分层架构（handler → logic → svc），使用 Axum 作为 Web 框架，Tower 作为中间件层。`grpc` 和 `model` 目录目前是空的占位符，可能是为将来的 gRPC 服务和数据模型生成预留的。

```plaintext
rsctl/templates/rs/
├── api/           # API 服务模板
│   ├── api.txt    # 简单说明文件 "api层"
│   ├── config.tpl # 配置结构体模板
│   ├── etc.tpl    # 配置文件模板 (YAML格式)
│   ├── handler.tpl # HTTP 处理器模板
│   ├── logic.tpl  # 业务逻辑层模板
│   ├── main.tpl   # 主入口模板
│   ├── middleware.tpl # 中间件模板
│   ├── svc.tpl    # 服务上下文模板
│   └── types.tpl  # 类型定义模板
├── grpc/
│   └── demo       # 空文件 (占位符)
└── model/
    └── demo       # 空文件 (占位符)
```

---

**User**

看来你对go-zero很熟悉，我这个是想借鉴go-zero的设计理念，你先帮规划模版目录，按照 api层，model层，rpc层，到时候model又分为mysql,pg等，rpc分为grpc,thrith等等，api分为rs语言，go语言等等

---

**Cursor**

我理解你的需求：要把 `rsctl/templates` 目录横向划分成 `api` / `model` / `rpc` 三大层级，每层再细分语言/驱动/协议，方便以后按照 go-zero 的分层思想生成代码。我的建议目录结构如下：

```
rsctl/templates/
├── api/
│   ├── rs/
│   │   ├── api.txt
│   │   ├── config.tpl
│   │   ├── etc.tpl
│   │   ├── handler.tpl
│   │   ├── logic.tpl
│   │   ├── main.tpl
│   │   ├── middleware.tpl
│   │   ├── svc.tpl
│   │   └── types.tpl
│   ├── go/
│   │   ├── ...（与 above 类似的 go-zero 风格模板）
│   └── ...（后续可加的语言，比如 ts、java）
├── model/
│   ├── mysql/
│   │   ├── model.tpl
│   │   └── query.tpl（例如带有 gorm / sqlx 的模板）
│   ├── pg/
│   │   ├── ...
│   ├── redis/
│   │   └── ...
│   └── ...（其他存储方式）
└── rpc/
    ├── grpc/
    │   ├── proto.tpl
    │   ├── server.tpl
    │   └── client.tpl
    ├── thrift/
    │   ├── ...
    └── http/
        └── ...
```

每个子目录下再细分按照服务角色（handler/logic/etc）或工具（proto、svc、middleware）来存放模板。你可以先以 `api/rs` 为基础再逐步扩展到其他语言和层。需要我帮你创建这个目录结构并移动/补充相应模板吗？

```plaintext
rsctl/templates/
├── api/
│   ├── rs/
│   │   ├── api.txt
│   │   ├── config.tpl
│   │   ├── etc.tpl
│   │   ├── handler.tpl
│   │   ├── logic.tpl
│   │   ├── main.tpl
│   │   ├── middleware.tpl
│   │   ├── svc.tpl
│   │   └── types.tpl
│   ├── go/
│   │   ├── ...（与 above 类似的 go-zero 风格模板）
│   └── ...（后续可加的语言，比如 ts、java）
├── model/
│   ├── mysql/
│   │   ├── model.tpl
│   │   └── query.tpl（例如带有 gorm / sqlx 的模板）
│   ├── pg/
│   │   ├── ...
│   ├── redis/
│   │   └── ...
│   └── ...（其他存储方式）
└── rpc/
    ├── grpc/
    │   ├── proto.tpl
    │   ├── server.tpl
    │   └── client.tpl
    ├── thrift/
    │   ├── ...
    └── http/
        └── ...
```

---

**User**

如果是这样，按照go-zero的思想，通过api描述文件，生成代码，这个功能是怎么实现

---

**Cursor**

## 生成流程（基于 go-zero 思想）

- **描述文件**：先写一个 `api` 描述文件（类似 go-zero 的 `.api`），描述服务、路由、请求/响应类型、依赖等；这个文件是整个代码生成的唯一来源。
- **解析器 + 模板引擎**：`rsctl` 读取描述文件，解析成抽象语法树/结构体（service、route、type 等），再通过已配置的模板（你刚规划的 `api/rs`、`model/mysql`、`rpc/grpc` 等）渲染各层代码。
- **分层输出**：针对每个路由：
  - 在 `handler.tpl` 生成入参验证、context 提取、调用 logic；
  - 在 `logic.tpl` 生成服务逻辑结构体；需要的 `model`/`rpc` 调用在逻辑层插入；
  - `svc.tpl` 管理依赖（比如 `Model`、外部 client、配置）；
  - `types.tpl` 按描述文件的请求/响应、枚举、模型结构生成 Rust/Go 类型。
- **模型层/RPC 层**：当接口需要数据库或外部 RPC 时，parser 会根据描述中的 `model`、`rpc` 声明生成对应的 `model` 文件（比如 `mysql` 的 `query.tpl`）或 `grpc` client/handler 等。
- **代码组织**：生成结果放到对应目录（`api/rs` 输出 handler/logic/main 等；`model/mysql` 输出 `model.rs`/`query.rs`；`rpc/grpc` 输出 proto + stub + server/client boilerplate），形成完整服务骨架。

如果你需要，我可以帮你：
1. 设计一个 `.api` 示例格式；
2. 实现解析器模板渲染流程；
3. 编写多层模板。

你想先从哪个步骤开始？

---

**User**

你帮我规划功能设计和对应的rust目录结构

---

**Cursor**

功能设计上，我会把生成器拆成三个核心阶段：

- **描述文件解析**：定义类 go-zero 的 `.api`（或 `.rsapi`）描述语言，声明服务、路由、请求/响应、依赖的 model/rpc，中间件、图示式注释、数据源等信息。解析器把描述转成 AST/结构体，供后续模板渲染使用。
- **模板渲染**：根据语言/层级（`api/rs`、`model/mysql`、`rpc/grpc`）选择对应模板，注入请求校验、中间件、逻辑调用、model/rpc 引用等，生成 handler/logic/types/svc 以及 model、rpc 相关文件。
- **输出管理**：按照 service 名、层级、语言组织文件到标准目录（`service_name/api/*`、`service_name/model/*`、`service_name/rpc/*`），并生成必要的配置（`etc/*.yaml`）、构建文件（`Cargo.toml`、`go.mod` 等）和 README，方便直接编译/启动。

对应的 Rust 项目结构（按 go-zero 分层扩展）建议如下：

```
service-name/
├── Cargo.toml
├── src/
│   ├── main.rs                   # 启动入口 (从模板生成)
│   ├── config/                   # 配置结构与 loader
│   │   └── mod.rs
│   ├── handler/                  # API 层，根据路由生成
│   │   └── mod.rs
│   ├── logic/                    # 业务逻辑
│   │   └── mod.rs
│   ├── svc/                      # ServiceContext + 中间件依赖
│   │   └── mod.rs
│   ├── types/                    # 请求/响应、DTO
│   │   └── mod.rs
│   ├── middleware/               # 可插拔 middleware 模板
│   │   └── mod.rs
│   ├── model/                    # 按驱动组织
│   │   ├── mysql/
│   │   │   ├── mod.rs
│   │   │   └── query.rs
│   │   ├── pg/
│   │   └── redis/
│   └── rpc/                      # 按协议组织
│       ├── grpc/
│       │   ├── proto/            # 生成的 proto/服务
│       │   └── client.rs
│       ├── thrift/
│       └── http/                 # 适配其他 HTTP RPC
├── etc/
│   └── service-name.yaml         # 模板生成的默认配置
├── proto/ (optional)              # gRPC proto 文件
└── docs/
    └── api.md                    # API 文档（从描述自动生成）
```

- 每个 `model/<driver>` 模板输出：`model.rs`、`query.rs`（含 SQL 构建、分页）。
- `rpc/<protocol>` 模板输出：`proto` + client/server bootstrap + mock。
- `api` 目录分语言（目前先支撑 `rs`，可扩展 `go`、`ts` 等），同级会输出 `handler`, `logic`, `types`, `svc` 等。
- 生成后的 `Cargo.toml` 会把 handler/logic 模块串联起来，`main.rs` 只负责启动和注册中间件/路由。

需要我继续帮你定义描述语法、模板变量或具体目录初始化脚本吗？

```plaintext
service-name/
├── Cargo.toml
├── src/
│   ├── main.rs                   # 启动入口 (从模板生成)
│   ├── config/                   # 配置结构与 loader
│   │   └── mod.rs
│   ├── handler/                  # API 层，根据路由生成
│   │   └── mod.rs
│   ├── logic/                    # 业务逻辑
│   │   └── mod.rs
│   ├── svc/                      # ServiceContext + 中间件依赖
│   │   └── mod.rs
│   ├── types/                    # 请求/响应、DTO
│   │   └── mod.rs
│   ├── middleware/               # 可插拔 middleware 模板
│   │   └── mod.rs
│   ├── model/                    # 按驱动组织
│   │   ├── mysql/
│   │   │   ├── mod.rs
│   │   │   └── query.rs
│   │   ├── pg/
│   │   └── redis/
│   └── rpc/                      # 按协议组织
│       ├── grpc/
│       │   ├── proto/            # 生成的 proto/服务
│       │   └── client.rs
│       ├── thrift/
│       └── http/                 # 适配其他 HTTP RPC
├── etc/
│   └── service-name.yaml         # 模板生成的默认配置
├── proto/ (optional)              # gRPC proto 文件
└── docs/
    └── api.md                    # API 文档（从描述自动生成）
```

---

**User**

不是这个代码的，是解析模版的功能的rust目录

---

**Cursor**

解析模板的 Rust 代码可以按功能拆成几个模块，目录结构参考下面这个：

```
rsctl/
├── src/
│   ├── main.rs                // 入口，负责接收命令行参数（如 .api 文件路径、目标语言/层级）、初始化日志等
│   ├── config/                // 配置解析（比如模板根路径、默认输出目录）
│   │   └── mod.rs
│   ├── parser/                // 描述文件解析器
│   │   ├── mod.rs
│   │   ├── ast.rs             // 定义 Service/Route/Type/Model/Rpc 等 AST 结构
│   │   ├── lexer.rs           // 可选：如果实现自定义语法需要词法分析
│   │   └── parser.rs          // 解析 `.api` 文件语法，返回 AST
│   ├── renderer/              // 模板渲染层
│   │   ├── mod.rs
│   │   ├── template.rs        // 抽象模板接口（按语言 + 层级选择）
│   │   └── engine.rs          // 包装 `tera`/`handlebars` 等模板引擎
│   ├── generator/             // 生成器核心逻辑
│   │   ├── mod.rs
│   │   ├── api.rs             // API 层（handler/logic/types/etc）渲染策略
│   │   ├── model.rs           // 按 mysql/pg 等执行 model 模板
│   │   ├── rpc.rs             // 按 grpc/thrift 渲染 rpc 模板
│   │   └── utils.rs           // 文件写入/路径生成
│   ├── templates/             // 内置模板（可以打包进二进制）
│   │   └── ...                // 例如 `api/rs/*.tpl`
│   ├── fs.rs                  // 输出目录创建、文件存在检查、备份旧文件等
│   └── cli.rs                 // 命令行参数定义（可用 `clap`）
└── templates/                  // 可复写模板路径（与 `rsctl/templates` 对应）
```

流程：`main` 调用 `parser` 生成 AST，`generator` 根据 AST + 目标层级选择 `renderer` 渲染模板，再用 `fs` 写入指定目录。每个子模块负责单一职责，便于后续扩展 go/ts 输出或新增 model/rpc 类型。需要我补充具体模块接口或模板变量设计吗？

```plaintext
rsctl/
├── src/
│   ├── main.rs                // 入口，负责接收命令行参数（如 .api 文件路径、目标语言/层级）、初始化日志等
│   ├── config/                // 配置解析（比如模板根路径、默认输出目录）
│   │   └── mod.rs
│   ├── parser/                // 描述文件解析器
│   │   ├── mod.rs
│   │   ├── ast.rs             // 定义 Service/Route/Type/Model/Rpc 等 AST 结构
│   │   ├── lexer.rs           // 可选：如果实现自定义语法需要词法分析
│   │   └── parser.rs          // 解析 `.api` 文件语法，返回 AST
│   ├── renderer/              // 模板渲染层
│   │   ├── mod.rs
│   │   ├── template.rs        // 抽象模板接口（按语言 + 层级选择）
│   │   └── engine.rs          // 包装 `tera`/`handlebars` 等模板引擎
│   ├── generator/             // 生成器核心逻辑
│   │   ├── mod.rs
│   │   ├── api.rs             // API 层（handler/logic/types/etc）渲染策略
│   │   ├── model.rs           // 按 mysql/pg 等执行 model 模板
│   │   ├── rpc.rs             // 按 grpc/thrift 渲染 rpc 模板
│   │   └── utils.rs           // 文件写入/路径生成
│   ├── templates/             // 内置模板（可以打包进二进制）
│   │   └── ...                // 例如 `api/rs/*.tpl`
│   ├── fs.rs                  // 输出目录创建、文件存在检查、备份旧文件等
│   └── cli.rs                 // 命令行参数定义（可用 `clap`）
└── templates/                  // 可复写模板路径（与 `rsctl/templates` 对应）
```

---

**User**

那你生成是不是分为rpc,model,api，这几个要统一放到generator 下面吗，后面比如再加其他数据的，那这个怎么扩展

---

**Cursor**

对，**默认生成器里可以把 `rpc`/`model`/`api` 各自的 orchestrator 都放到 `generator` 模块下**，核心思想是让 `generator` 负责把解析出的描述抽象（AST）和模板映射起来，例如：

```
generator/
├── mod.rs          // 统一入口：接收 AST + 输出配置，调度具体子生成器
├── api.rs          // 专注 api 层（handler/logic/types）
├── model.rs        // 专注 model 层（mysql/pg/...）
├── rpc.rs          // 专注 rpc 层（grpc/thrift/...）
└── utils.rs        // 公共路径/文件写入 helpers
```

这样做的好处是每个子生成器只关注自己层的模板渲染、依赖注入、代码路径，`mod.rs` 负责把它们串起来（按服务描述调用）。

**后续加新层级（比如 data、scheduler、job）时**，只要在 `generator` 下新增一个子模块（`data.rs`），并在 `mod.rs` 里注册它即可。例如：

```rust
pub struct Generator {
    api: ApiGenerator,
    model: ModelGenerator,
    rpc: RpcGenerator,
    data: DataGenerator, // 新增
}
```

只需保证 AST 里也有对应的节点（如 `.data` 描述），渲染层能按需求选择模板（`templates/data/...`）。渲染引擎/模板目录的划分也同 `api/model/rpc` 保持一致，方便未来统一扩展。

需要我帮你把 `generator` 的具体接口草案或者目录结构写出来吗？

```plaintext
generator/
├── mod.rs          // 统一入口：接收 AST + 输出配置，调度具体子生成器
├── api.rs          // 专注 api 层（handler/logic/types）
├── model.rs        // 专注 model 层（mysql/pg/...）
├── rpc.rs          // 专注 rpc 层（grpc/thrift/...）
└── utils.rs        // 公共路径/文件写入 helpers
```

```rust
pub struct Generator {
    api: ApiGenerator,
    model: ModelGenerator,
    rpc: RpcGenerator,
    data: DataGenerator, // 新增
}
```

---

**User**

我是想说，这几个也是单独目录，generator 里面复杂生成处理逻辑

---

**Cursor**

你说得对：`api`、`model`、`rpc` 这些生成单元可以各自做成独立目录（比如 `generator/api/`、`generator/model/`、`generator/rpc/`），各目录里封装该层的“转换逻辑 + 按语言/协议的分支”，而 `generator/mod.rs` 负责：

- 注册每个子生成器（trait + impl）；
- 根据 AST 或描述文件，按顺序调用 `api::generate(...)`、`model::generate(...)`、`rpc::generate(...)`；
- 处理公共依赖（比如模板路径、输出目录、是否覆盖旧文件）。

扩展时只要再在 `generator/` 下新增目录（比如 `generator/data/`），实现同样的 trait，就能无缝插入 `mod.rs` 的调度流程。这样既保留了各层独立实现的清晰度，也让 `generator` 成为统一的 orchestration 层，你需要我具体写个 `generator` 目录的草图吗？

---

**User**

然后你帮我列出功能目录结构

---

**Cursor**

以下是我为你规划的 `rsctl` 生成器功能目录结构，按职责划分，既满足独立层级，也方便扩展：

```
rsctl/
├── src/
│   ├── main.rs                      # CLI 入口，解析参数、初始化日志
│   ├── cli.rs                       # 命令行（clap）定义
│   ├── config/
│   │   └── mod.rs                   # 配置解析（模板根、本地覆盖等）
│   ├── parser/                      # 描述文件解析
│   │   ├── mod.rs
│   │   ├── ast.rs                   # Service/Route/Model/Rpc 等结构体
│   │   ├── lexer.rs                 # 自定义语法词法分析（可选）
│   │   └── parser.rs                # .api/.rsapi 语法解析器
│   ├── renderer/                    # 模板引擎封装
│   │   ├── mod.rs
│   │   ├── template.rs              # 模板定位/变量映射
│   │   └── engine.rs                # 封装 tera/handlebars 渲染
│   ├── generator/                   # 生成器核心目录
│   │   ├── mod.rs                   # 统一调度：按 AST 依次调 api/model/rpc
│   │   ├── api/                     # API 层生成逻辑（按语言切换模板）
│   │   │   ├── mod.rs
│   │   │   ├── rs.rs
│   │   │   └── go.rs
│   │   ├── model/                   # Model 层（mysql/pg/redis 等）生成
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs
│   │   │   └── pg.rs
│   │   ├── rpc/                     # RPC 层（grpc/thrift/http）生成
│   │   │   ├── mod.rs
│   │   │   ├── grpc.rs
│   │   │   └── thrift.rs
│   │   └── utils.rs                 # 公共路径、文件写入、模板选择
│   ├── fs.rs                        # 输出目录管理、备份、覆盖策略
│   └── templates/                   # 内置模板（打包到二进制供 renderer 用）
│       ├── api/rs/
│       ├── model/mysql/
│       ├── rpc/grpc/
│       └── ...
├── templates/                       # 可自定义模板目录（对外配置）
└── assets/                          # 文档、说明、示例 api 文件
```

- `generator` 下的 `api/model/rpc` 是独立目录，各自实现对应的生成逻辑和语言/驱动分支，`mod.rs` 负责统一调用并处理全局配置。
- 扩展新的层（比如 `data`、`scheduler`）只需在 `generator/` 下再开一个目录并在 `mod.rs` 注册，`ast` 也同步新增节点。
- `renderer` 与 `templates` 负责把 AST 绑到具体模板。`fs` 模块保证写入安全。

需要我再细化每个子模块的接口/函数签名或模板变量吗？

```plaintext
rsctl/
├── src/
│   ├── main.rs                      # CLI 入口，解析参数、初始化日志
│   ├── cli.rs                       # 命令行（clap）定义
│   ├── config/
│   │   └── mod.rs                   # 配置解析（模板根、本地覆盖等）
│   ├── parser/                      # 描述文件解析
│   │   ├── mod.rs
│   │   ├── ast.rs                   # Service/Route/Model/Rpc 等结构体
│   │   ├── lexer.rs                 # 自定义语法词法分析（可选）
│   │   └── parser.rs                # .api/.rsapi 语法解析器
│   ├── renderer/                    # 模板引擎封装
│   │   ├── mod.rs
│   │   ├── template.rs              # 模板定位/变量映射
│   │   └── engine.rs                # 封装 tera/handlebars 渲染
│   ├── generator/                   # 生成器核心目录
│   │   ├── mod.rs                   # 统一调度：按 AST 依次调 api/model/rpc
│   │   ├── api/                     # API 层生成逻辑（按语言切换模板）
│   │   │   ├── mod.rs
│   │   │   ├── rs.rs
│   │   │   └── go.rs
│   │   ├── model/                   # Model 层（mysql/pg/redis 等）生成
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs
│   │   │   └── pg.rs
│   │   ├── rpc/                     # RPC 层（grpc/thrift/http）生成
│   │   │   ├── mod.rs
│   │   │   ├── grpc.rs
│   │   │   └── thrift.rs
│   │   └── utils.rs                 # 公共路径、文件写入、模板选择
│   ├── fs.rs                        # 输出目录管理、备份、覆盖策略
│   └── templates/                   # 内置模板（打包到二进制供 renderer 用）
│       ├── api/rs/
│       ├── model/mysql/
│       ├── rpc/grpc/
│       └── ...
├── templates/                       # 可自定义模板目录（对外配置）
└── assets/                          # 文档、说明、示例 api 文件
```

---

**User**

不用 mod.rs    ，用rust最新版本的包组织方式

---

**Cursor**

在 Rust 2021 里，可以不用 `mod.rs`，改用更平展的模块组织方式：  
每个目录下放一个与目录同名的 `.rs` 文件来声明子模块，保持清晰又省略 `mod.rs`。

你可以按下述结构组织 `rsctl`：

```
rsctl/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── parser/
│   │   ├── mod.rs          ← 只能保留一个入口，里面 `pub mod ast; pub mod parser;`
│   │   ├── ast.rs
│   │   ├── lexer.rs
│   │   └── parser.rs
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── template.rs
│   │   └── engine.rs
│   ├── generator/
│   │   ├── mod.rs
│   │   ├── api/
│   │   │   ├── mod.rs      ← 这里可以用 `pub mod rs; pub mod go;`，但目录下不再有 `mod.rs`，用 `api.rs` 也行
│   │   │   ├── rs.rs
│   │   │   └── go.rs
│   │   ├── model/
│   │   │   ├── mysql.rs
│   │   │   └── pg.rs
│   │   ├── rpc/
│   │   │   ├── grpc.rs
│   │   │   └── thrift.rs
│   │   └── utils.rs
│   ├── fs.rs
│   └── templates.rs
```

关键点：

- 根模块（比如 `parser/mod.rs`）仍需一个入口文件，但子模块可以用 `parser/ast.rs`、`parser/parser.rs`，并在入口 `mod.rs` 里 `pub mod ast; pub mod parser;`。
- `generator/api` 目录可以把 `rs.rs`、`go.rs` 等直接当成模块，不用 `mod.rs`，只需在 `generator/api.rs` 或 `generator/mod.rs` 里 `pub mod api;` 并在 `generator/api.rs` 内 `pub mod rs; pub mod go;`。
- 这种组织避免嵌套 `mod.rs`，更贴合最新风格。

需要我具体给出某个目录的 `pub mod` 声明示例吗？

```plaintext
rsctl/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── parser/
│   │   ├── mod.rs          ← 只能保留一个入口，里面 `pub mod ast; pub mod parser;`
│   │   ├── ast.rs
│   │   ├── lexer.rs
│   │   └── parser.rs
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── template.rs
│   │   └── engine.rs
│   ├── generator/
│   │   ├── mod.rs
│   │   ├── api/
│   │   │   ├── mod.rs      ← 这里可以用 `pub mod rs; pub mod go;`，但目录下不再有 `mod.rs`，用 `api.rs` 也行
│   │   │   ├── rs.rs
│   │   │   └── go.rs
│   │   ├── model/
│   │   │   ├── mysql.rs
│   │   │   └── pg.rs
│   │   ├── rpc/
│   │   │   ├── grpc.rs
│   │   │   └── thrift.rs
│   │   └── utils.rs
│   ├── fs.rs
│   └── templates.rs
```

---

**User**

都说了不用mod.rs，你还保留

---

**Cursor**

```plaintext
src/
  main.rs
  cli.rs
  config.rs
  parser.rs            // 作为parser模块入口，声明 pub mod ast; pub mod lexer; pub mod grammar;
  parser/ast.rs
  parser/lexer.rs
  parser/grammar.rs

  renderer.rs
  renderer/engine.rs
  renderer/template.rs

  generator.rs         // 主调度模块
  generator/api.rs     // generator::api模块入口，声明 pub mod rs; pub mod go;
  generator/api/rs.rs
  generator/api/go.rs
  generator/model.rs
  generator/model/mysql.rs

  rpc.rs
  rpc/service.rs
  rpc/client.rs
```

好的，完全去掉 `mod.rs` 的组织方式（用同名 `.rs` 文件作为入口），建议如下：

```
rsctl/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── parser.rs              // pub mod ast; pub mod lexer; pub mod grammar; ...
│   │   └── parser/            // 子模块文件放目录里
│   │       ├── ast.rs
│   │       ├── lexer.rs
│   │       └── grammar.rs
│   ├── renderer.rs            // pub mod engine; pub mod template;
│   │   └── renderer/
│   │       ├── engine.rs
│   │       └── template.rs
│   ├── generator.rs           // 统一调度：api/model/rpc/data...，在这里 pub mod api; model; rpc; utils;
│   │   └── generator/
│   │       ├── api.rs         // 负责 API 层调度，内部再 pub mod rs_api; go_api;
│   │       │   └── api/
│   │       │       ├── rs_api.rs
│   │       │       └── go_api.rs
│   │       ├── model.rs       // 负责 model 层调度，内部 pub mod mysql; pg; redis;
│   │       │   └── model/
│   │       │       ├── mysql.rs
│   │       │       └── pg.rs
│   │       ├── rpc.rs         // 负责 rpc 层调度，内部 pub mod grpc; thrift; http;
│   │       │   └── rpc/
│   │       │       ├── grpc.rs
│   │       │       └── thrift.rs
│   │       └── utils.rs
│   ├── fs.rs                  // 输出/覆盖/备份策略
│   └── templates.rs           // 内置模板读取/路径解析
├── templates/                 // 可覆盖/自定义模板（同结构）
│   ├── api/rs/
│   ├── api/go/
│   ├── model/mysql/
│   ├── model/pg/
│   ├── rpc/grpc/
│   └── rpc/thrift/
└── assets/                    // 示例 .api 描述、文档
```

要点：
- 每个目录的入口改成同名 `.rs` 文件（例如 `parser.rs`），再用 `pub mod xxx;` 声明子模块，子模块源码放在同名子目录下（如 `parser/ast.rs`）。
- `generator.rs` 只做调度（按 AST 调用 api/model/rpc 等），每个子层有自己的入口文件（`api.rs`、`model.rs`、`rpc.rs`），里面再细分语言/驱动文件。
- 扩展新层（如 `data.rs`/`job.rs`）时，只需在 `generator.rs` 里 `pub mod data;`，并新增 `generator/data.rs` + `generator/data/*.rs`。

```plaintext
rsctl/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── parser.rs              // pub mod ast; pub mod lexer; pub mod grammar; ...
│   │   └── parser/            // 子模块文件放目录里
│   │       ├── ast.rs
│   │       ├── lexer.rs
│   │       └── grammar.rs
│   ├── renderer.rs            // pub mod engine; pub mod template;
│   │   └── renderer/
│   │       ├── engine.rs
│   │       └── template.rs
│   ├── generator.rs           // 统一调度：api/model/rpc/data...，在这里 pub mod api; model; rpc; utils;
│   │   └── generator/
│   │       ├── api.rs         // 负责 API 层调度，内部再 pub mod rs_api; go_api;
│   │       │   └── api/
│   │       │       ├── rs_api.rs
│   │       │       └── go_api.rs
│   │       ├── model.rs       // 负责 model 层调度，内部 pub mod mysql; pg; redis;
│   │       │   └── model/
│   │       │       ├── mysql.rs
│   │       │       └── pg.rs
│   │       ├── rpc.rs         // 负责 rpc 层调度，内部 pub mod grpc; thrift; http;
│   │       │   └── rpc/
│   │       │       ├── grpc.rs
│   │       │       └── thrift.rs
│   │       └── utils.rs
│   ├── fs.rs                  // 输出/覆盖/备份策略
│   └── templates.rs           // 内置模板读取/路径解析
├── templates/                 // 可覆盖/自定义模板（同结构）
│   ├── api/rs/
│   ├── api/go/
│   ├── model/mysql/
│   ├── model/pg/
│   ├── rpc/grpc/
│   └── rpc/thrift/
└── assets/                    // 示例 .api 描述、文档
```

---

**User**

unigen/                                              # workspace 根目录
  Cargo.toml                                         # [workspace]：成员管理/统一依赖版本
  README.md                                          # 总览说明/快速开始

  crates/                                            # workspace 成员（crate 集合）
    cli/                                             # 二进制 crate：命令行入口（最终产物）
      Cargo.toml
      src/
        main.rs                                      # 入口：解析参数 -> 调用 core
        cli/                                         # clap/子命令定义与分发（薄层）
          commands/                                  # api/model/rpc 等子命令
          args/                                      # 参数结构/校验/配置加载

    core/                                            # 核心库 crate：流水线编排 + 公共能力（对外主 API）
      Cargo.toml
      src/
        lib.rs                                       # 对外接口：run_xxx/pipeline
        pipeline/                                    # Parse->Semantic->Spec->Gen->Write 编排
        common/                                      # 通用：error/naming/fs/config/logging 等

    spec/                                            # IR/Spec crate：统一代码生成模型（稳定层）
      Cargo.toml
      src/
        lib.rs
        api/                                         # API Spec/Descriptor
        model/                                       # Model Spec/Descriptor（表/字段/索引/关系等）
        rpc/                                         # RPC Spec/Descriptor（service/method/message 等）

    parse/                                           # Parse crate：输入解析（文本/文件 -> AST/中间结构）
      Cargo.toml
      src/
        lib.rs
        api/                                         # API 输入解析（DSL/OpenAPI 等）
        model/                                       # Model 输入解析（mysql/pg 等方言解析点更适合在这里）
        rpc/                                         # RPC 输入解析（proto/thrift 等）

    semantic/                                        # Semantic crate：语义分析与归一化（AST -> Spec）
      Cargo.toml
      src/
        lib.rs
        api/                                         # API 语义检查/默认值/命名归一
        model/                                       # Model 语义检查/类型归一/约束归一（吸收大部分方言差异）
        rpc/                                         # RPC 语义检查/引用解析/包名归一等

    gen/                                             # Gen crate：生成层（Spec -> Artifacts/待写文件集合）
      Cargo.toml
      src/
        lib.rs
        api/                                         # API 生成器集合（领域优先；语言/框架建议放更深层）
        model/                                       # Model 生成器集合（如 go/go-zero/gorm... 放更深层）
        rpc/                                         # RPC 生成器集合（如 grpc/thrift... 放更深层）
        template/                                    # 模板渲染封装（helpers/filters/loader）
        write/                                       # 写文件与增量更新策略（覆盖/跳过/合并/标记块）

  templates/                                         # 外置模板（可选；也可迁入 gen 内并内嵌）
    api/                                             # API 模板根
    model/                                           # Model 模板根（例如 go/go-zero 风格）
    rpc/                                             # RPC 模板根

  examples/                                          # 示例输入/示例工程（可选）
  docs/                                              # 设计文档/规范（可选）


你觉得这个结构咋样

---

**Cursor**

整体结构很清晰，按“输入→解析→语义→规范→生成→写出”链路分层，并用单独 crate 解耦，便于演进和测试。几点小建议供参考：

- workspace/crate 划分  
  - `spec` 做稳定 IR 很好；建议把跨层共用的类型/错误放 `spec` 或 `core::common`，避免在 `parse/semantic/gen` 间重复定义。  
  - `parse` 与 `semantic` 分开合理；如果早期功能少，可以先合并一个 crate，成熟后再拆。  
  - `gen` 里已分 `template`、`write`，建议再加一个 `lang/` 或在 `api/model/rpc` 下再分语言/框架（如 `api/rs/axum.rs`、`api/go/zero.rs`），保持可扩展。

- crate 职责边界  
  - `core` 仅做编排和公共能力，不要承载业务规则，规则留给 `parse/semantic/gen`。  
  - `cli` 薄层：参数校验→调用 `core::pipeline`，不要夹带逻辑。  
  - `pipeline` 建议支持阶段化执行（只 parse/semantic，不生成）方便调试。

- 模板与资源  
  - 外置 `templates/` OK，同时在 `gen::template` 支持内嵌（`include_bytes!`）+ 覆盖策略，便于单可执行分发。  
  - `write` 层考虑：覆盖/跳过/merge（标记块）策略配置化，并输出变更清单。

- 命名与依赖  
  - 统一 error/naming/fs/logging 放在 `core::common`，其他 crate 只依赖它和 `spec`。  
  - 约定好 `Spec` 的稳定接口后，`gen` 可以独立演进，不被 parser 方言影响。

- 测试与示例  
  - `examples/` 放最小 `.api`/`.proto`/`.ddl` 示例，确保 pipeline 跑通。  
  - 每个 crate 保留单元测试，`core` 增加集成测试（全链路）。

总体架构可行，利于扩展新层（如 data/job）。如果要进一步细化，我可以帮你列出各 crate 的 pub 接口草案或生成阶段的 config 约定。

---

**User**

按照你的理解，然后帮我优化整理下我刚才发你的文件结构目录，然后再rsctl目录下帮我组织下代码目录结构，按照rust的方式

---

**Cursor**

### 我帮你优化后的 workspace 目录结构（在 `rsctl/` 下落地）

你发的 `unigen/` 结构本身已经很接近“可长期演进”的形态了，我做的优化重点是：**命名更 Rust/workspace 化、职责边界更硬、扩展点更明确、模板与写入策略可插拔**。

```text
rsctl/                             # workspace 根（建议把生成器体系都放这里）
  Cargo.toml                       # [workspace] + [workspace.dependencies]
  README.md

  crates/
    rsctl-cli/                     # 二进制：命令行入口（薄层）
      Cargo.toml
      src/
        main.rs
        cli.rs                     # 顶层命令路由（clap）
        cli/
          api.rs                   # api 子命令：gen/validate/format 等
          model.rs
          rpc.rs
          common.rs                # 通用参数：--out/--templates/--overwrite...

    rsctl-core/                    # 编排：Parse -> Semantic -> Gen -> Write（对外主 API）
      Cargo.toml
      src/
        lib.rs                     # 对外：run()/pipeline()
        pipeline.rs                # 分阶段执行/可只跑到某一阶段
        pipeline/
          api.rs
          model.rs
          rpc.rs
        common.rs                  # error/naming/path/fs/logging/config（跨 crate 也可下沉到 spec）

    rsctl-spec/                    # 稳定 IR：统一 Spec（最重要的稳定层）
      Cargo.toml
      src/
        lib.rs
        api.rs
        api/
          types.rs
          routes.rs
        model.rs
        model/
          schema.rs                # 表/字段/索引/关系
          types.rs                 # 统一类型系统（i32/string/datetime/json…）
        rpc.rs
        rpc/
          service.rs
          message.rs

    rsctl-parse/                   # 输入解析：文本/文件 -> 解析结果（AST 或者轻量中间结构）
      Cargo.toml
      src/
        lib.rs
        api.rs                     # .api DSL / OpenAPI
        api/
          dsl.rs
          openapi.rs
        model.rs                   # ddl/反射/连接数据库导出（可选）
        model/
          mysql.rs
          pg.rs
        rpc.rs                     # proto/thrift
        rpc/
          proto.rs
          thrift.rs

    rsctl-semantic/                # 语义分析：Parse 输出 -> Spec（默认值/命名/引用/检查）
      Cargo.toml
      src/
        lib.rs
        api.rs
        model.rs
        rpc.rs

    rsctl-gen/                     # 生成：Spec -> Artifacts（待写文件集合）
      Cargo.toml
      src/
        lib.rs
        api.rs
        api/
          rs_axum.rs               # 语言/框架下沉到这里（可继续细分）
          go_zero.rs
        model.rs
        model/
          mysql.rs
          pg.rs
        rpc.rs
        rpc/
          grpc.rs
          thrift.rs
        template.rs                # 模板渲染封装（loader/helpers/filters）
        write.rs                   # 写出策略（覆盖/跳过/合并/标记块）
        write/
          plan.rs                  # 变更计划（diff/新增/更新）
          strategy.rs              # overwrite/merge/skip

  templates/                       # 外置模板根（可被 --templates 覆盖）
    api/
      rs/
      go/
    model/
      mysql/
      pg/
    rpc/
      grpc/
      thrift/

  examples/                        # 示例输入（.api/.proto/ddl）+ 期望输出（golden）
  docs/                            # 设计文档/规范/模板变量说明
```

- **关键优化点**：
  - **统一 crate 命名**：用 `rsctl-*` 前缀，workspace 里依赖关系更清楚，避免 `core/spec` 这种容易撞名。
  - **`spec` 作为稳定边界**：新增任何数据源/协议/语言，尽量只影响 `parse/semantic/gen`，而 `spec` 变化最少。
  - **扩展新“层”**（比如 `job/data/mq`）：按同样模式加 `spec::job`、`parse::job`、`semantic::job`、`gen::job`，`core::pipeline` 只需要注册编排即可。

---

### `rsctl` 各 crate 内部的 Rust 代码目录组织方式（完全不用 `mod.rs`）

你前面强调“不用 `mod.rs`”，那就用 Rust 推荐的 **“入口文件 + 子目录文件”** 组织：  
`foo.rs` 作为模块入口，子模块放 `foo/xxx.rs`。

下面给一个典型 crate（以 `rsctl-gen` 为例）的内部结构，你其他 crate 也照这个套路来：

```text
crates/rsctl-gen/
  src/
    lib.rs                 # pub mod api; pub mod model; pub mod rpc; pub mod template; pub mod write;
    api.rs                 # pub mod rs_axum; pub mod go_zero;
    api/
      rs_axum.rs
      go_zero.rs
    model.rs               # pub mod mysql; pub mod pg;
    model/
      mysql.rs
      pg.rs
    rpc.rs                 # pub mod grpc; pub mod thrift;
    rpc/
      grpc.rs
      thrift.rs
    template.rs
    write.rs               # pub mod plan; pub mod strategy;
    write/
      plan.rs
      strategy.rs
```

同理，`rsctl-parse` 也会是：

```text
crates/rsctl-parse/
  src/
    lib.rs
    api.rs
    api/
      dsl.rs
      openapi.rs
    model.rs
    model/
      mysql.rs
      pg.rs
    rpc.rs
    rpc/
      proto.rs
      thrift.rs
```

---

### 迁移你当前仓库的最小建议

你现在仓库里已有 `rsctl/api` + `rsctl/templates`，我建议：

- **把 `rsctl/api` 未来演进成 `crates/rsctl-cli`**（或者短期直接保留，后面再搬）
- **把现有 `rsctl/templates/rs/...` 迁到新结构的 `rsctl/templates/api/rs/...`**（路径更符合你要的分层）

如果你确认要按这个结构“真的落盘”，我可以直接在你的仓库里创建这些目录与空的 crate 骨架（含 `Cargo.toml`/`lib.rs`/`main.rs`），并把现有模板移动到新位置。你希望 workspace 根就叫 `rsctl/`，还是改成你写的 `unigen/`？

```plaintext
rsctl/                             # workspace 根（建议把生成器体系都放这里）
  Cargo.toml                       # [workspace] + [workspace.dependencies]
  README.md

  crates/
    rsctl-cli/                     # 二进制：命令行入口（薄层）
      Cargo.toml
      src/
        main.rs
        cli.rs                     # 顶层命令路由（clap）
        cli/
          api.rs                   # api 子命令：gen/validate/format 等
          model.rs
          rpc.rs
          common.rs                # 通用参数：--out/--templates/--overwrite...

    rsctl-core/                    # 编排：Parse -> Semantic -> Gen -> Write（对外主 API）
      Cargo.toml
      src/
        lib.rs                     # 对外：run()/pipeline()
        pipeline.rs                # 分阶段执行/可只跑到某一阶段
        pipeline/
          api.rs
          model.rs
          rpc.rs
        common.rs                  # error/naming/path/fs/logging/config（跨 crate 也可下沉到 spec）

    rsctl-spec/                    # 稳定 IR：统一 Spec（最重要的稳定层）
      Cargo.toml
      src/
        lib.rs
        api.rs
        api/
          types.rs
          routes.rs
        model.rs
        model/
          schema.rs                # 表/字段/索引/关系
          types.rs                 # 统一类型系统（i32/string/datetime/json…）
        rpc.rs
        rpc/
          service.rs
          message.rs

    rsctl-parse/                   # 输入解析：文本/文件 -> 解析结果（AST 或者轻量中间结构）
      Cargo.toml
      src/
        lib.rs
        api.rs                     # .api DSL / OpenAPI
        api/
          dsl.rs
          openapi.rs
        model.rs                   # ddl/反射/连接数据库导出（可选）
        model/
          mysql.rs
          pg.rs
        rpc.rs                     # proto/thrift
        rpc/
          proto.rs
          thrift.rs

    rsctl-semantic/                # 语义分析：Parse 输出 -> Spec（默认值/命名/引用/检查）
      Cargo.toml
      src/
        lib.rs
        api.rs
        model.rs
        rpc.rs

    rsctl-gen/                     # 生成：Spec -> Artifacts（待写文件集合）
      Cargo.toml
      src/
        lib.rs
        api.rs
        api/
          rs_axum.rs               # 语言/框架下沉到这里（可继续细分）
          go_zero.rs
        model.rs
        model/
          mysql.rs
          pg.rs
        rpc.rs
        rpc/
          grpc.rs
          thrift.rs
        template.rs                # 模板渲染封装（loader/helpers/filters）
        write.rs                   # 写出策略（覆盖/跳过/合并/标记块）
        write/
          plan.rs                  # 变更计划（diff/新增/更新）
          strategy.rs              # overwrite/merge/skip

  templates/                       # 外置模板根（可被 --templates 覆盖）
    api/
      rs/
      go/
    model/
      mysql/
      pg/
    rpc/
      grpc/
      thrift/

  examples/                        # 示例输入（.api/.proto/ddl）+ 期望输出（golden）
  docs/                            # 设计文档/规范/模板变量说明
```

```plaintext
crates/rsctl-gen/
  src/
    lib.rs                 # pub mod api; pub mod model; pub mod rpc; pub mod template; pub mod write;
    api.rs                 # pub mod rs_axum; pub mod go_zero;
    api/
      rs_axum.rs
      go_zero.rs
    model.rs               # pub mod mysql; pub mod pg;
    model/
      mysql.rs
      pg.rs
    rpc.rs                 # pub mod grpc; pub mod thrift;
    rpc/
      grpc.rs
      thrift.rs
    template.rs
    write.rs               # pub mod plan; pub mod strategy;
    write/
      plan.rs
      strategy.rs
```

```plaintext
crates/rsctl-parse/
  src/
    lib.rs
    api.rs
    api/
      dsl.rs
      openapi.rs
    model.rs
    model/
      mysql.rs
      pg.rs
    rpc.rs
    rpc/
      proto.rs
      thrift.rs
```

---

**User**

不用加rsctl这个前缀，下面就是core，cli,parse这样的，，然后把现有的迁移把现有 rsctl/templates/rs/... 迁到新结构的 rsctl/templates/api/rs/
然后罗盘

---

**Cursor**

我来把这件事“真正落盘”到你的仓库里：先看清 `rsctl/` 现状与哪些地方引用了旧模板路径，然后按你要的 crate 命名（`core/cli/parse/...` 不加前缀）创建 workspace 骨架，最后把现有 `rsctl/templates/rs/...` 迁移到 `rsctl/templates/api/rs/`（并顺手把 grpc/model 也归位到 `rpc/`、`model/`）。

很好：目前 `rsctl/api` 还是个空白示例 crate（没引用模板路径），所以迁移模板目录不会牵扯改代码。接下来我会在 `rsctl/` 下创建你要的 workspace 骨架（`crates/core|cli|parse|semantic|spec|gen`），并把 `rsctl/templates/rs/...` 落盘迁移到新结构的 `rsctl/templates/api/rs/...`（同时把 grpc/model 归类到 `rpc/`、`model/`）。

```plaintext
[workspace]
resolver = "2"
members = [
  "crates/cli",
  "crates/core",
  "crates/spec",
  "crates/parse",
  "crates/semantic",
  "crates/gen",
]

[workspace.package]
edition = "2024"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }

```

```markdown
# rsctl workspace

该目录是 `rsctl` 的独立 Rust workspace，用于实现“描述文件 -> 解析 -> 语义归一 -> Spec(IR) -> 代码生成 -> 写盘”的完整流水线。

## crates

- `crates/cli`：命令行入口（薄层）
- `crates/core`：流水线编排（对外主 API）
- `crates/spec`：稳定的 IR/Spec 定义
- `crates/parse`：输入解析（api/model/rpc）
- `crates/semantic`：语义分析与归一化（parse -> spec）
- `crates/gen`：代码生成（spec -> artifacts -> write）


```

我已经把 `rsctl/` 变成一个独立 workspace（不加 rsctl 前缀的 crates），接下来会把 `crates/cli|core|spec|parse|semantic|gen` 的最小骨架落盘，并把旧模板目录 `templates/rs/...` 迁移到 `templates/api/rs/...`（grpc/model 也会分别归到 `templates/rpc/*`、`templates/model/*`）。

```plaintext
[package]
name = "cli"
version.workspace = true
edition.workspace = true

[dependencies]
clap.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true
core = { path = "../core" }


```

```rust
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rsctl", version, about = "Code generator CLI")]
struct Args {
    /// 模板根目录（默认：rsctl/templates）
    #[arg(long)]
    templates: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _args = Args::parse();
    Ok(())
}


```

```plaintext
[package]
name = "core"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
spec = { path = "../spec" }
parse = { path = "../parse" }
semantic = { path = "../semantic" }
gen = { path = "../gen" }


```

```rust
//! Pipeline orchestration: parse -> semantic -> spec -> gen -> write.

pub fn ping() -> &'static str {
    "core"
}


```

```plaintext
[package]
name = "spec"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true


```

```rust
//! Stable IR (Spec) used by generators.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
}


```

```plaintext
[package]
name = "parse"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true


```

```rust
//! Input parsing (api/model/rpc) -> AST/lightweight structures.

pub fn ping() -> &'static str {
    "parse"
}


```

```plaintext
[package]
name = "semantic"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
spec = { path = "../spec" }


```

```rust
//! Semantic analysis: parse output -> spec normalization/validation.

pub fn ping() -> &'static str {
    "semantic"
}


```

```plaintext
[package]
name = "gen"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
spec = { path = "../spec" }


```

```rust
//! Code generation: spec -> artifacts.

pub fn ping() -> &'static str {
    "gen"
}


```

我已经把 `rsctl/` workspace 的 `crates/{cli,core,spec,parse,semantic,gen}` 最小骨架落盘了（crate 名称就是你要的 `core/cli/...`，不加前缀）。下一步开始迁移模板：把原 `rsctl/templates/rs/api/*` 迁到 `rsctl/templates/api/rs/*`，并把 `rsctl/templates/rs/grpc`、`rsctl/templates/rs/model` 分别归到 `rsctl/templates/rpc/grpc`、`rsctl/templates/model/`。

```plaintext
api层


```

```plaintext
// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use serde::Deserialize;
use rest::RestConf; // 提前实现好的公共配置结构体
{{.extraImports}}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub rest: RestConf
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rest: RestConf::default()
        }
    }
}

pub fn load_config() -> Config {
    // TODO: 之后可以改成从环境变量 / 配置文件加载
    Config::default()
}


```

```plaintext
// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

Name: {{.serviceName}}
Host: {{.host}}
Port: {{.port}}


```

```plaintext
use std::sync::Arc;
use axum::{
    extract::{State{{if .HasRequest}}, Json{{end}}},
    response::{IntoResponse, Response},
};
use crate::svc::ServiceContext;
{{.ImportPackages}}

{{if .HasDoc}}{{.Doc}}{{end}}
pub async fn {{.HandlerName}}(
    State(svc_ctx): State<Arc<ServiceContext>>,
    {{if .HasRequest}}Json(req): Json<crate::types::{{.RequestType}}>,{{end}}
) -> Response {
    let logic = crate::logic::{{.LogicName}}::{{.LogicType}}::new(svc_ctx);

    {{if .HasResp}}
    match logic.{{.Call}}({{if .HasRequest}}req{{end}}).await {
        Ok(resp) => resp.into_response(),
        Err(err) => err.into_response(),
    }
    {{else}}
    match logic.{{.Call}}({{if .HasRequest}}req{{end}}).await {
        Ok(()) => ().into_response(), // 这里你可以自定义一个 OK 的 IntoResponse 实现
        Err(err) => err.into_response(),
    }
    {{end}}
}


```

```plaintext
// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use std::sync::Arc;
use tracing::instrument;
use crate::svc::ServiceContext;
{{.imports}}

pub struct {{.logic}} {
    /// 请求级别的 tracing span / 日志上下文
    // 这里不直接存 context，通常从 handler 的参数里获取；如需可改成存 `Span` 或自定义 Context。
    svc_ctx: Arc<ServiceContext>,
}

impl {{.logic}} {
    pub fn new(svc_ctx: Arc<ServiceContext>) -> Self {
        Self { svc_ctx }
    }

    {{if .hasDoc}}{{.doc}}{{end}}
    #[instrument(skip(self{{if .request}}, req{{end}}))]
    pub async fn {{.function}}(
        &self{{if .request}},
        {{.request}}{{end}}
    ) {{.responseType}} {
        // todo: add your logic here and delete this line

        {{.returnString}}
    }
}


```

```plaintext
// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use std::{env, net::SocketAddr, sync::Arc};
use tokio::signal;
use axum::Router;

{{.importPackages}}

/// 配置文件路径，默认：`etc/{{.serviceName}}.yaml`
/// 支持通过环境变量覆盖：`CONFIG_FILE=/path/to/config.yaml`
const DEFAULT_CONFIG_FILE: &str = "etc/{{.serviceName}}.yaml";

#[tokio::main]
async fn main() {
    // 解析配置文件路径：优先环境变量，其次默认值
    let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());

    // 加载配置：这里假设你有一个 config 模块和 Config 结构体，类似 go-zero 的 Config
    // 你可以在 config.tpl 里定义具体结构，这里只用占位符形式描述。
    let c = match config::load(&config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("failed to load config from {}: {}", config_file, e);
            std::process::exit(1);
        }
    };

    // 组合监听地址：等价 Go 的 c.Host, c.Port
    let addr: SocketAddr = format!("{}:{}", c.host, c.port)
        .parse()
        .expect("invalid host/port in config");

    // 初始化全局 ServiceContext
    let svc_ctx = Arc::new(svc::ServiceContext::new(c.clone()));

    // 构建路由：等价 Go 里的 handler.RegisterHandlers(server, ctx)
    let app: Router = handler::register_handlers(svc_ctx);

    println!("Starting server at {}:{}...", c.host, c.port);

    // 启动服务，并支持 ctrl+c 优雅退出
    if let Err(err) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("server error: {}", err);
    }
}

async fn shutdown_signal() {
    // Ctrl+C
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term =
            signal(SignalKind::terminate()).expect("failed to install signal handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}


```

```plaintext
// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use std::task::{Context, Poll};
use std::{future::Future, pin::Pin};
use tower::{Layer, Service};
use axum::response::Response;
use axum::http::{Request};

pub struct {{.name}};

impl {{.name}} {
    pub fn new() -> Self {
        {{.name}}
    }

    /// 返回一个可用于 axum 路由的 Layer：
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/path", get(handler))
    ///     .layer({{.name}}::new().layer());
    /// ```
    pub fn layer<S>(&self) -> {{.name}}Layer<S> {
        {{.name}}Layer::new()
    }
}

/// 中间件的 Layer 类型，用于包裹 Service。
pub struct {{.name}}Layer<S> {
    inner: std::marker::PhantomData<S>,
}

impl<S> {{.name}}Layer<S> {
    pub fn new() -> Self {
        Self {
            inner: std::marker::PhantomData,
        }
    }
}

impl<S> Layer<S> for {{.name}}Layer<S> {
    type Service = {{.name}}Middleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        {{.name}}Middleware { inner }
    }
}

/// 实际的中间件实现：在这里编写你的逻辑。
pub struct {{.name}}Middleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for {{.name}}Middleware<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // TODO generate middleware implement function, delete after code implementation

        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 这里写你的前置逻辑，例如：
            // - 记录日志
            // - 校验 header / token
            // - 注入 trace id 到 extensions
            //
            // 示例：
            // println!("request path: {}", req.uri().path());

            // Passthrough to next service / handler if need
            let response = inner.call(req).await?;

            // 这里可以写后置逻辑，例如修改响应头
            // let mut response = response;
            // response.headers_mut().insert(...);

            Ok(response)
        })
    }
}


```

```plaintext
// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

{{.configImport}}

pub struct ServiceContext {
    pub config: {{.config}},
    {{.middleware}}
}

impl ServiceContext {
    pub fn new(config: {{.config}}) -> Self {
        Self {
            config,
            {{.middlewareAssignment}}
        }
    }
}


```

```plaintext
// Code generated by rsctl. DO NOT EDIT.
// rsctl {{.version}}

{{if .containsTime}}
use chrono::{DateTime, Utc};
{{end}}

{{.types}}


```

---

**User**

有一个问题，rsctl下面为什么要jcreates,，然后才是core,cli那些

---

**Cursor**

### 为什么要 `crates/`，而不是直接 `rsctl/core`、`rsctl/cli`？

这是 Rust workspace 里最常见的一种组织方式，核心原因是**把“workspace 根目录的杂项”和“各个 crate 的源码”分开**，长期维护会更干净。

- **避免根目录混乱**：workspace 根通常还会放 `templates/`、`docs/`、`examples/`、CI 配置等；把所有 crate 统一放进 `crates/`，根目录不会被 `core/cli/parse/...` 填满。
- **扩展更自然**：以后加 `gen`、`semantic`、`spec`、`plugins` 等，直接在 `crates/` 下加一个目录，不会和 `templates/`、`examples/` 同级挤在一起。
- **Cargo 生态惯例**：很多大型仓库（Rust 官方/社区项目）都这么放，团队协作时一眼能看出“哪些是 crate”。

### 你不想要 `crates/` 也完全可以

如果你更喜欢扁平结构，直接这样也行（Cargo 完全支持）：

```text
rsctl/
  Cargo.toml
  cli/
  core/
  parse/
  semantic/
  spec/
  gen/
  templates/
```

我可以按你的偏好把现在的 `crates/*` **整体搬到根目录同级**，并同步更新 `rsctl/Cargo.toml` 的 `members` 路径。你倾向保留 `crates/` 还是改成扁平？

```plaintext
rsctl/
  Cargo.toml
  cli/
  core/
  parse/
  semantic/
  spec/
  gen/
  templates/
```

---

**User**

改成扁平

---

**Cursor**

好，我把 `rsctl/crates/*` 改成你要的**扁平结构**：把 `cli/core/parse/semantic/spec/gen` 都搬到 `rsctl/` 根目录同级，并同步更新 workspace 的 `members` 以及各 crate 的 `path` 依赖。接下来我会先改 `rsctl/Cargo.toml`，再复制文件到新路径，最后删除旧的 `crates/*` 里的文件。

```plaintext
[workspace]
resolver = "2"
members = [
  "cli",
  "core",
  "spec",
  "parse",
  "semantic",
  "gen",
]

[workspace.package]
edition = "2024"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }


```

我已经把 workspace 的 `members` 改成扁平路径了。下一步是把 `crates/cli` 等目录下的文件复制到 `rsctl/cli`、`rsctl/core`… 并校准各 crate 的 `path` 依赖，然后再删除旧文件，避免 workspace 里出现重复 crate。

```plaintext
[package]
name = "cli"
version.workspace = true
edition.workspace = true

[dependencies]
clap.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true
core = { path = "../core" }


```

```rust
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rsctl", version, about = "Code generator CLI")]
struct Args {
    /// 模板根目录（默认：rsctl/templates）
    #[arg(long)]
    templates: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _args = Args::parse();
    Ok(())
}


```

我已经把 `cli` 迁到了根目录同级（`rsctl/cli`）。接下来用同样方式把 `core/spec/parse/semantic/gen` 全部迁过去，并删除 `rsctl/crates/*` 下的旧文件，确保 workspace 只认新的扁平 crates。

```plaintext
[package]
name = "core"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
spec = { path = "../spec" }
parse = { path = "../parse" }
semantic = { path = "../semantic" }
gen = { path = "../gen" }


```

```rust
//! Pipeline orchestration: parse -> semantic -> spec -> gen -> write.

pub fn ping() -> &'static str {
    "core"
}


```

```plaintext
[package]
name = "spec"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true


```

```rust
//! Stable IR (Spec) used by generators.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
}


```

```plaintext
[package]
name = "parse"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true


```

```rust
//! Input parsing (api/model/rpc) -> AST/lightweight structures.

pub fn ping() -> &'static str {
    "parse"
}


```

```plaintext
[package]
name = "semantic"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
spec = { path = "../spec" }


```

```rust
//! Semantic analysis: parse output -> spec normalization/validation.

pub fn ping() -> &'static str {
    "semantic"
}


```

```plaintext
[package]
name = "gen"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
spec = { path = "../spec" }


```

```rust
//! Code generation: spec -> artifacts.

pub fn ping() -> &'static str {
    "gen"
}


```

```markdown
# rsctl workspace

该目录是 `rsctl` 的独立 Rust workspace，用于实现“描述文件 -> 解析 -> 语义归一 -> Spec(IR) -> 代码生成 -> 写盘”的完整流水线。

## crates

- `cli`：命令行入口（薄层）
- `core`：流水线编排（对外主 API）
- `spec`：稳定的 IR/Spec 定义
- `parse`：输入解析（api/model/rpc）
- `semantic`：语义分析与归一化（parse -> spec）
- `gen`：代码生成（spec -> artifacts -> write）



```

### 已完成：改成扁平结构 + 迁移模板（落盘）

#### 1) `rsctl/` workspace 改成扁平 crates
- **原来**：`rsctl/crates/{cli,core,spec,parse,semantic,gen}`
- **现在**：`rsctl/{cli,core,spec,parse,semantic,gen}`（同级扁平）
- 同时更新了 `rsctl/Cargo.toml` 的 `[workspace].members` 指向新路径。

#### 2) 迁移模板目录
- 已把原 `rsctl/templates/rs/api/*` 迁到 **`rsctl/templates/api/rs/*`**
- 并把原来的占位 `grpc/model` 归位到：
  - `rsctl/templates/rpc/grpc/`
  - `rsctl/templates/model/`

#### 3) 清理旧 `crates/*`
- 旧的 `rsctl/crates/*` 下文件已删除（目录还空着，之后你要彻底删目录可用 git/资源管理器直接删）。

#### 4) 当前 `rsctl/` 目录（关键部分）
- `rsctl/cli`
- `rsctl/core`
- `rsctl/spec`
- `rsctl/parse`
- `rsctl/semantic`
- `rsctl/gen`
- `rsctl/templates/api/rs/...`

如果你希望我下一步也把空的 `rsctl/crates/` 目录本身删掉（以及确认 `cargo check` 通过），你允许我跑一次 `cargo check` 我就直接验证。

---

**User**

rsctl/                             # workspace 根（建议把生成器体系都放这里）
  Cargo.toml                       # [workspace] + [workspace.dependencies]
  README.md

  crates/
    rsctl-cli/                     # 二进制：命令行入口（薄层）
      Cargo.toml
      src/
        main.rs
        cli.rs                     # 顶层命令路由（clap）
        cli/
          api.rs                   # api 子命令：gen/validate/format 等
          model.rs
          rpc.rs
          common.rs                # 通用参数：--out/--templates/--overwrite...

    rsctl-core/                    # 编排：Parse -> Semantic -> Gen -> Write（对外主 API）
      Cargo.toml
      src/
        lib.rs                     # 对外：run()/pipeline()
        pipeline.rs                # 分阶段执行/可只跑到某一阶段
        pipeline/
          api.rs
          model.rs
          rpc.rs
        common.rs                  # error/naming/path/fs/logging/config（跨 crate 也可下沉到 spec）

    rsctl-spec/                    # 稳定 IR：统一 Spec（最重要的稳定层）
      Cargo.toml
      src/
        lib.rs
        api.rs
        api/
          types.rs
          routes.rs
        model.rs
        model/
          schema.rs                # 表/字段/索引/关系
          types.rs                 # 统一类型系统（i32/string/datetime/json…）
        rpc.rs
        rpc/
          service.rs
          message.rs

    rsctl-parse/                   # 输入解析：文本/文件 -> 解析结果（AST 或者轻量中间结构）
      Cargo.toml
      src/
        lib.rs
        api.rs                     # .api DSL / OpenAPI
        api/
          dsl.rs
          openapi.rs
        model.rs                   # ddl/反射/连接数据库导出（可选）
        model/
          mysql.rs
          pg.rs
        rpc.rs                     # proto/thrift
        rpc/
          proto.rs
          thrift.rs

    rsctl-semantic/                # 语义分析：Parse 输出 -> Spec（默认值/命名/引用/检查）
      Cargo.toml
      src/
        lib.rs
        api.rs
        model.rs
        rpc.rs

    rsctl-gen/                     # 生成：Spec -> Artifacts（待写文件集合）
      Cargo.toml
      src/
        lib.rs
        api.rs
        api/
          rs_axum.rs               # 语言/框架下沉到这里（可继续细分）
          go_zero.rs
        model.rs
        model/
          mysql.rs
          pg.rs
        rpc.rs
        rpc/
          grpc.rs
          thrift.rs
        template.rs                # 模板渲染封装（loader/helpers/filters）
        write.rs                   # 写出策略（覆盖/跳过/合并/标记块）
        write/
          plan.rs                  # 变更计划（diff/新增/更新）
          strategy.rs              # overwrite/merge/skip

  templates/                       # 外置模板根（可被 --templates 覆盖）
    api/
      rs/
      go/
    model/
      mysql/
      pg/
    rpc/
      grpc/
      thrift/

  examples/                        # 示例输入（.api/.proto/ddl）+ 期望输出（golden）
  docs/                            # 设计文档/规范/模板变量说明
这个里面的解释都加到readme里面，到时候下次好理解

---

**Cursor**

我来把你贴的那份“目录结构 + 每个目录职责说明”整理进 `rsctl/README.md`，并且会按**当前已经落盘的扁平结构（`cli/core/spec/...`）**来写，同时在 README 里明确区分“**当前最小骨架**”和“**后续规划的扩展目录**”，下次你一眼就能看懂每层该放什么。

```markdown
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



```

### 已把你那份“结构解释”整理进 `rsctl/README.md`

我已更新 `rsctl/README.md`，内容包含：

- **当前已落盘的扁平 workspace 结构**（`cli/core/spec/parse/semantic/gen/templates`）
- **每个 crate 的职责解释**（为什么这样分层 + 流水线关系）
- **后续建议的细化目录**（比如 `core/pipeline/*`、`gen/write/*`、`parse/api/*` 等），并明确说明“这是规划结构，按实现逐步补齐即可”
- **templates 分层说明**（并标注 `templates/api/rs` 已迁移完成）

你下次只要打开 README，就能快速定位“某类逻辑该放在哪个 crate/哪个子目录”。

