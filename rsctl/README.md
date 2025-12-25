# rsctl workspace

该目录是 `rsctl` 的独立 Rust workspace，用于实现“描述文件 -> 解析 -> 语义归一 -> Spec(IR) -> 代码生成 -> 写盘”的完整流水线。

## 快速开始

### 安装/构建 rsctl

#### 方法1：直接从 Git 仓库安装（推荐）

```bash
cargo install --git https://github.com/fucktx/rust-zero --branch api --bin rsctl --force
```

#### 方法2：克隆后本地安装

```bash
git clone -b api https://github.com/fucktx/rust-zero.git
cd rust-zero/rsctl
cargo install --path cli --force
```

#### 方法3：本地构建

```bash
git clone -b api https://github.com/fucktx/rust-zero.git
cd rust-zero/rsctl
make build
```

构建产物位于 `rsctl/dist/rsctl`（Windows 为 `rsctl/dist/rsctl.exe`）。你可以手动加入 PATH，或直接使用该文件。

### 模板管理（template）

模板会安装到当前用户目录下：`~/.rsctl/<VERSION>/`（Windows 示例：`C:\Users\xiaohan\.rsctl\<VERSION>\`）。

```bash
# 初始化：安装当前版本模板（已存在则不覆盖）
rsctl template init

# 清理：只删除当前版本模板目录（支持拼写兼容：celan）
rsctl template clean
rsctl template celan

# 更新：覆盖安装当前版本模板
rsctl template update
```

### rsctl：API 生成使用说明（Rust）

#### 1) 查看 rsctl 版本

```bash
rsctl --version
# 或
rsctl -v
```

#### 2) 生成 API 工程（示例：axum）

最常用命令（建议加 `-o` 覆盖已有文件）：

```bash
rsctl api rs \
  -a rsctl/tests/api.api \
  -d rsctl/tests/out \
  --web axum \
  -o
```

参数说明：
- `-a, --api`：`.api` 描述文件路径
- `-d, --dir`：输出目录
- `-w, --web`：目标框架（当前支持 `axum` / `actix`，默认 `axum`）
- `-o, --overwrite`：覆盖输出目录中已有文件
- `-m, --merge`：同组 handler 是否合并到一个文件（默认 `true`）
- `-s, --style`：生成的 `.rs` 文件命名风格：`rust_zero` / `rustZero` / `RustZero`
- `-r, --remote`：模板来源（可传本地目录或 git/http URL）

提示：默认模板根会优先使用 `~/.rsctl/<当前 rsctl 版本>/`；建议首次使用先执行一次 `rsctl template update`。

#### 3) 运行生成结果

```bash
cd rsctl/tests/out
cargo run
```

如果端口被占用，会看到类似 `failed to bind ... Address already in use` 的提示；修改 `rsctl/tests/out/etc/config.yaml` 的 `Port` 即可。

#### 4) 生成结果如何接入本仓库的 `rest`（零运行时抽象）

当前生成的 `handler/routes.rs` 会使用本仓库 `rest` crate 提供的 `rest::add_routes!`/路由 DSL 来构建路由：
- 写法统一（router/group/middleware 等 meta）
- 编译期展开为原生框架调用（axum/actix），避免 `dyn/BoxFuture` 等运行时抽象成本

默认生成（`--web axum`）会在生成工程的 `Cargo.toml` 里将 `rest` 依赖指向本仓库的 `rest` crate，并启用 `features = ["axum"]`。

## templates（外置模板）

模板目前已按分层归档（与你的目标一致：api/model/rpc）：

- `templates/api/rs/`：Rust API（axum）模板（已迁移完成）
- `templates/model/`：模型层模板（后续会细分 mysql/pg 等）
- `templates/rpc/grpc/`：RPC 层模板（后续可加 thrift 等）

## 许可证

rsctl 随仓库一起采用 Apache-2.0，详见仓库根目录 `LICENSE`。


