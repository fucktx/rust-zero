## rsctl：API 生成使用说明（Rust）

### 安装/构建 rsctl

在仓库内构建（推荐）：

```bash
cd rsctl
make build
```

构建产物：
- `rsctl/dist/rsctl`
- macOS 额外可用：`make mac` → `rsctl/dist/rsctl-darwin`

### 查看 RS 生成器版本号

（这是 Rust API 生成器/模板版本号，不等同于 `rsctl --version` 的 CLI 版本）

```bash
rsctl -v
```

### 生成 API 工程（axum）

最常用命令（推荐加 `-o` 覆盖已有文件）：

```bash
rsctl api rs \
  -a rsctl/test/api.api \
  -d rsctl/test/out \
  --web axum \
  -o
```

参数说明：
- `-a, --api`: `.api` 描述文件路径
- `-d, --dir`: 输出目录
- `-f, --web`（或 `--framework`）: 目标框架（当前支持 `axum`/`actix`，默认 `axum`）
- `-o, --overwrite`: 覆盖输出目录中已有文件
- `-m, --merge`: 同组 handler 是否合并到一个文件（默认 true）
- `-s, --style`: 生成的 `.rs` 文件命名风格：`rust_zero` / `rustZero` / `RustZero`
- `-r, --remote`: 模板来源（可传本地目录或 git/http URL）

### 运行生成结果

```bash
cd rsctl/test/out
cargo run main.rs
```

如果端口被占用，会看到类似 `failed to bind ... Address already in use` 的提示；修改 `rsctl/test/out/etc/config.yaml` 的 `Port` 即可。

### 生成结果如何接入本仓库的 `rest::router!`（零运行时抽象）

当前生成的 `handler.rs` 会使用本仓库 `rest` crate 提供的 `rest::router!{...}` DSL 来构建路由：
- **表面写法统一**（router/group/middleware 等语法）
- **编译期展开为原生框架调用**（axum/actix），避免 `dyn/BoxFuture` 等运行时抽象成本

默认生成（`--web axum`）会在生成工程的 `Cargo.toml` 里把 `rest` 依赖指向本仓库的 `rest` crate，并启用 `features = ["axum"]`。
