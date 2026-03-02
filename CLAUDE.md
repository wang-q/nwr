# CLAUDE.md

此文件为 AI 助手在处理本仓库代码时提供指南与上下文。

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust

**语言约定**: 为了便于指导，本文件 (`CLAUDE.md`) 使用中文编写，且**与用户交流时请使用中文**。但项目代码中的**所有文档注释 (doc comments)**、**行内注释**以及**提交信息**必须使用**英文**。

**目录约定**: 任何被 `.gitignore` 完全忽略的目录，均仅作为参考资料，**不是本项目的一部分**。

`nwr` 是一个处理 NCBI 分类学数据、Newick 树文件和组装报告的命令行工具集。它旨在提供高效的工具来管理物种分类信息、操作系统发育树以及处理基因组组装元数据。

## 构建命令

### 构建

```bash
# 开发构建
cargo build

# 发布构建 (高性能)
cargo build --release
```

### 测试

```bash
# 运行所有测试
cargo test
```

## 架构

### 源代码组织

- **`src/nwr.rs`** - 主程序入口，负责命令行解析和分发。
    - 使用 `clap` 进行参数解析。
    - 在 `main` 函数中注册所有子命令模块。
- **`src/lib.rs`** - 库入口，导出模块。
- **`src/cmd_nwr/`** - 命令实现模块。按功能分组：
    - **Database**: `download`, `txdb` (Taxonomy DB), `ardb` (Assembly Report DB).
    - **Taxonomy**: `info`, `lineage`, `member`, `append`, `restrict`, `common`.
    - **Assembly**: `template` (Tera templates), `kb`, `seqdb`.
    - **Newick Data**: `data` (包含 `label`, `stat`, `distance`).
    - **Newick Operations**: `ops` (包含 `order`, `rename`, `replace`, `topo`, `subtree`, `prune`, `reroot`).
    - **Newick Visualization**: `viz` (包含 `indent`, `comment`, `tex`).
    - **Build Tree**: `build` (包含 `nj`, `upgma`).
    - **Plots**: `plot` (包含 `hh`, `venn`, `nrps`).
    - **Pipeline**: `pl_condense`.
- **`src/libs/`** - 共享工具库和核心逻辑。
  - **`io.rs`** - I/O 辅助函数。
  - **`newick.rs`** - Newick 树处理逻辑。
  - **`taxonomy.rs`** - 分类学数据处理逻辑。

### 命令结构 (Command Structure)

每个命令在 `src/cmd_nwr/` 下作为一个独立的模块实现，通常包含两个公开函数：

1.  **`make_subcommand`**: 定义命令行接口。
    -   返回 `clap::Command`。
    -   使用 `.about(...)` 设置简短描述。
2.  **`execute`**: 命令执行逻辑。
    -   接收 `&clap::ArgMatches`。
    -   返回 `anyhow::Result<()>`。

### 关键依赖

- **`clap`**: 命令行参数解析。
- **`anyhow`**: 错误处理。
- **`rusqlite`**: SQLite 数据库操作 (用于 `txdb`, `ardb`)。
- **`phylotree`**: 系统发育树处理。
- **`petgraph`**: 图数据结构。
- **`tera`**: 模板引擎 (用于生成脚本)。
- **`intspan`**: 整数区间操作。
- **`regex`**: 正则表达式。

## 开发工作流

### 添加新命令

1.  在 `src/cmd_nwr/` 下相应的类别目录中创建新文件 (或新建目录)。
2.  在 `src/cmd_nwr/mod.rs` (或子目录的 `mod.rs`) 中声明该模块。
3.  在 `src/nwr.rs` 中注册该子命令。
4.  实现 `make_subcommand` 和 `execute`。
5.  添加测试文件 `tests/cli_<command>.rs`。

### 测试约定

- 集成测试位于 `tests/` 目录下，通常命名为 `cli_<category>.rs` (如 `cli_nwr_taxonomy.rs`, `cli_newick_ops.rs`)。
- 测试数据通常放在 `tests/` 下的相关子目录中 (如 `tests/newick/`, `tests/nwr/`)。
- **推荐使用 `assert_cmd`** 来编写集成测试，以验证二进制文件的行为。
- **稳定性原则 (Zero Panic)**: 任何用户输入（包括畸形数据）都不应导致程序 Panic。必须捕获所有错误并返回友好的错误信息。

## 代码规范

- 使用 `cargo fmt` 格式化代码。
- 使用 `cargo clippy` 检查潜在问题。
- 优先使用标准库和项目中已引入的 crate。
- 保持代码简洁，注重性能。

## 帮助文本规范 (Help Text Style Guide)

- **About**: 第三人称单数动词 (e.g., "Downloads...", "Converts...").
- **Args**:
    - Input: `infile` / `infiles`.
    - Output: `outfile` (`-o`).
- **Description**: 简明扼要，解释命令的核心功能。
- **Examples**: 提供典型的使用示例。
