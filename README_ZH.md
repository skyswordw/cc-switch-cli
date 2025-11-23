<div align="center">

# CC-Switch CLI

[![Version](https://img.shields.io/badge/version-4.0.0--cli-blue.svg)](https://github.com/farion1231/cc-switch/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/farion1231/cc-switch/releases)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Claude Code、Codex 与 Gemini CLI 的命令行管理工具**

统一管理 Claude Code、Codex 与 Gemini CLI 的供应商配置、MCP 服务器、Skills 扩展和系统提示词。

[English](README.md) | [中文](README_ZH.md)

</div>

---

## 📖 关于本项目

本项目是原版 [CC-Switch](https://github.com/farion1231/cc-switch) 的 **CLI 分支**。


**致谢：** 原始架构和核心功能来自 [farion1231/cc-switch](https://github.com/farion1231/cc-switch)

---

## 📸 截图预览

### 交互式主界面

<div align="center">
  <img src="assets/screenshots/main-ch.png" alt="主界面" width="800"/>
</div>

### 供应商管理

<div align="center">
  <img src="assets/screenshots/add-ch.png" alt="供应商管理" width="800"/>
</div>

---

## ✨ 功能特性

### 🎯 两种使用模式

**命令行模式**
```bash
cc-switch provider switch <id>       # 切换供应商
cc-switch mcp sync                   # 同步 MCP 服务器
cc-switch prompts activate <id>      # 激活提示词预设
```

**交互模式**
```bash
cc-switch                            # 启动菜单驱动界面
```

### 🔌 供应商管理

管理 **Claude Code**、**Codex** 和 **Gemini** 的 API 配置。

```bash
cc-switch provider list              # 列出所有供应商
cc-switch provider current           # 显示当前供应商
cc-switch provider switch <id>       # 切换供应商
cc-switch provider add               # 添加新供应商
cc-switch provider delete <id>       # 删除供应商
cc-switch provider speedtest <id>    # 测试 API 延迟
```

功能：一键切换、多端点支持、API 密钥管理、速度测试、供应商复制。

### 🛠️ MCP 服务器管理

跨 Claude/Codex/Gemini 管理模型上下文协议服务器。

```bash
cc-switch mcp list                   # 列出所有 MCP 服务器
cc-switch mcp enable <id> --app claude   # 为特定应用启用
cc-switch mcp sync                   # 同步所有已启用服务器
cc-switch mcp import --app claude    # 从配置导入
```

功能：统一管理、多应用支持、三种传输类型（stdio/http/sse）、自动同步、智能 TOML 解析器。

### 💬 Prompts 管理

管理 AI 编码助手的系统提示词预设。

```bash
cc-switch prompts list               # 列出提示词预设
cc-switch prompts activate <id>      # 激活提示词
cc-switch prompts show <id>          # 显示完整内容
cc-switch prompts delete <id>        # 删除提示词
```

跨应用支持：Claude (`CLAUDE.md`)、Codex (`AGENTS.md`)、Gemini (`GEMINI.md`)。

### ⚙️ 配置管理

```bash
cc-switch config show                # 显示配置
cc-switch config backup              # 创建备份
cc-switch config export <path>       # 导出配置
cc-switch config import <path>       # 导入配置
```

### 🌐 多语言支持

交互模式支持中英文切换，语言设置会自动保存。

- 默认语言：English
- 进入 `⚙️ 设置` 菜单切换语言

### 🔧 实用工具

```bash
cc-switch completions <shell>        # 生成 shell 补全（bash/zsh/fish/powershell）
cc-switch env check                  # 检查冲突
cc-switch app switch <app>           # 切换应用上下文
```

---

## 📥 安装

### 从源码构建

**前提条件：**
- Rust 1.85+（[通过 rustup 安装](https://rustup.rs/)）

**构建：**
```bash
git clone https://github.com/your-username/cc-switch-cli.git
cd cc-switch-cli/src-tauri
cargo build --release

# 二进制位置：./target/release/cc-switch
```

**安装到系统：**
```bash
# macOS/Linux
cp target/release/cc-switch /usr/local/bin/

# Windows
copy target\release\cc-switch.exe C:\Windows\System32\
```

---

## 🚀 快速开始

### 首次运行

**交互模式**（推荐）
```bash
cc-switch
```
按照屏幕菜单探索功能。

---

## 🏗️ 架构

### 核心设计

- **SSOT**：所有配置存于 `~/.cc-switch/config.json`，实时配置是生成的产物
- **原子写入**：临时文件 + 重命名模式防止损坏
- **服务层复用**：100% 复用原 GUI 版本
- **并发安全**：RwLock 配合作用域守卫

### 配置文件

**CC-Switch 存储：**
- `~/.cc-switch/config.json` - 主配置（SSOT）
- `~/.cc-switch/settings.json` - 设置
- `~/.cc-switch/backups/` - 自动轮换（保留 10 个）

**实时配置：**
- Claude: `~/.claude/settings.json`, `~/.claude.json` (MCP), `~/.claude/CLAUDE.md` (提示词)
- Codex: `~/.codex/auth.json`, `~/.codex/config.toml` (MCP), `~/.codex/AGENTS.md` (提示词)
- Gemini: `~/.gemini/.env`, `~/.gemini/settings.json` (MCP), `~/.gemini/GEMINI.md` (提示词)

---

## 🛠️ 开发

### 环境要求

- **Rust**：1.85+（[rustup](https://rustup.rs/)）
- **Cargo**：与 Rust 捆绑

### 开发命令

```bash
cd src-tauri

cargo run                            # 开发模式
cargo run -- provider list           # 运行特定命令
cargo build --release                # 构建 release

cargo fmt                            # 代码格式化
cargo clippy                         # 代码检查
cargo test                           # 运行测试
```

### 代码结构

```
src-tauri/src/
├── cli/
│   ├── commands/          # CLI 子命令（provider, mcp, prompts, config）
│   ├── interactive/       # 交互式 TUI 模式
│   └── ui.rs              # UI 实用工具（表格、颜色）
├── services/              # 业务逻辑
├── main.rs                # CLI 入口点
└── ...
```


## 🤝 贡献

欢迎贡献！本分支专注于 CLI 功能。

**提交 PR 前：**
- ✅ 通过格式检查：`cargo fmt --check`
- ✅ 通过代码检查：`cargo clippy`
- ✅ 通过测试：`cargo test`
- 💡 先开 issue 讨论

---

## 📜 许可证

- MIT © 原作者：Jason Young
- CLI 分支维护者：saladday

