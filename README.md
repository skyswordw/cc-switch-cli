<div align="center">

# CC-Switch CLI

[![Version](https://img.shields.io/badge/version-4.0.0--cli-blue.svg)](https://github.com/farion1231/cc-switch/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/farion1231/cc-switch/releases)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Command-Line Management Tool for Claude Code, Codex & Gemini CLI**

Unified management for Claude Code, Codex & Gemini CLI provider configurations, MCP servers, Skills extensions, and system prompts.

[English](README.md) | [中文](README_ZH.md)

</div>

---

## 📖 About

This project is a **CLI fork** of [CC-Switch](https://github.com/farion1231/cc-switch).


**Credits:** Original architecture and core functionality from [farion1231/cc-switch](https://github.com/farion1231/cc-switch)

---

## 📸 Screenshots

### Interactive Main Menu

<div align="center">
  <img src="assets/screenshots/main-en.png" alt="Main Menu" width="800"/>
</div>

### Provider Management

<div align="center">
  <img src="assets/screenshots/add-en.png" alt="Provider Management" width="800"/>
</div>

---

## ✨ Features

### 🎯 Two Usage Modes

**Command-Line Mode**
```bash
cc-switch provider switch <id>       # Switch provider
cc-switch mcp sync                   # Sync MCP servers
cc-switch prompts activate <id>      # Activate prompt preset
```

**Interactive Mode**
```bash
cc-switch                            # Launch menu-driven interface
```

### 🔌 Provider Management

Manage API configurations for **Claude Code**, **Codex**, and **Gemini**.

```bash
cc-switch provider list              # List all providers
cc-switch provider current           # Show current provider
cc-switch provider switch <id>       # Switch provider
cc-switch provider add               # Add new provider
cc-switch provider delete <id>       # Delete provider
cc-switch provider speedtest <id>    # Test API latency
```

Features: One-click switching, multi-endpoint support, API key management, speed testing, provider duplication.

### 🛠️ MCP Server Management

Manage Model Context Protocol servers across Claude/Codex/Gemini.

```bash
cc-switch mcp list                   # List all MCP servers
cc-switch mcp enable <id> --app claude   # Enable for specific app
cc-switch mcp sync                   # Sync all enabled servers
cc-switch mcp import --app claude    # Import from config
```

Features: Unified management, multi-app support, three transport types (stdio/http/sse), automatic sync, smart TOML parser.

### 💬 Prompts Management

Manage system prompt presets for AI coding assistants.

```bash
cc-switch prompts list               # List prompt presets
cc-switch prompts activate <id>      # Activate prompt
cc-switch prompts show <id>          # Display full content
cc-switch prompts delete <id>        # Delete prompt
```

Cross-app support: Claude (`CLAUDE.md`), Codex (`AGENTS.md`), Gemini (`GEMINI.md`).

### ⚙️ Configuration Management

```bash
cc-switch config show                # Display configuration
cc-switch config backup              # Create backup
cc-switch config export <path>       # Export configuration
cc-switch config import <path>       # Import configuration
```

### 🌐 Multi-language Support

Interactive mode supports English and Chinese, language settings are automatically saved.

- Default language: English
- Go to `⚙️ Settings` menu to switch language

### 🔧 Utilities

```bash
cc-switch completions <shell>        # Generate shell completions (bash/zsh/fish/powershell)
cc-switch env check                  # Check for conflicts
cc-switch app switch <app>           # Switch application context
```

---

## 📥 Installation

### Build from Source

**Prerequisites:**
- Rust 1.85+ ([install via rustup](https://rustup.rs/))

**Build:**
```bash
git clone https://github.com/your-username/cc-switch-cli.git
cd cc-switch-cli/src-tauri
cargo build --release

# Binary location: ./target/release/cc-switch
```

**Install to System:**
```bash
# macOS/Linux
cp target/release/cc-switch /usr/local/bin/

# Windows
copy target\release\cc-switch.exe C:\Windows\System32\
```

---

## 🚀 Quick Start

### First Run

**Interactive Mode** (Recommended)
```bash
cc-switch
```
Follow on-screen menus to explore features.

**Command-Line Mode**
```bash
cc-switch provider list              # List providers
cc-switch provider switch <id>       # Switch provider
cc-switch config show                # View configuration
```

### Common Workflows

**Switch Provider:**
```bash
cc-switch provider list
cc-switch provider switch my-provider-id
# Restart Claude Code/Codex/Gemini to apply changes
```

**Manage MCP Servers:**
```bash
cc-switch mcp import --app claude    # Import existing servers
cc-switch mcp enable mcp-fetch --app codex
cc-switch mcp sync                   # Sync all
```

**Manage Prompts:**
```bash
cc-switch prompts list
cc-switch prompts activate coding-assistant
```

---

## 🏗️ Architecture

### Core Design

- **SSOT**: All config in `~/.cc-switch/config.json`, live configs are generated artifacts
- **Atomic Writes**: Temp file + rename pattern prevents corruption
- **Service Layer Reuse**: 100% reused from original GUI version
- **Concurrency Safe**: RwLock with scoped guards

### Configuration Files

**CC-Switch Storage:**
- `~/.cc-switch/config.json` - Main configuration (SSOT)
- `~/.cc-switch/settings.json` - Settings
- `~/.cc-switch/backups/` - Auto-rotation (keep 10)

**Live Configs:**
- Claude: `~/.claude/settings.json`, `~/.claude.json` (MCP), `~/.claude/CLAUDE.md` (prompts)
- Codex: `~/.codex/auth.json`, `~/.codex/config.toml` (MCP), `~/.codex/AGENTS.md` (prompts)
- Gemini: `~/.gemini/.env`, `~/.gemini/settings.json` (MCP), `~/.gemini/GEMINI.md` (prompts)

---

## 🛠️ Development

### Requirements

- **Rust**: 1.85+ ([rustup](https://rustup.rs/))
- **Cargo**: Bundled with Rust

### Commands

```bash
cd src-tauri

cargo run                            # Development mode
cargo run -- provider list           # Run specific command
cargo build --release                # Build release

cargo fmt                            # Format code
cargo clippy                         # Lint code
cargo test                           # Run tests
```

### Code Structure

```
src-tauri/src/
├── cli/
│   ├── commands/          # CLI subcommands (provider, mcp, prompts, config)
│   ├── interactive/       # Interactive TUI mode
│   └── ui.rs              # UI utilities (tables, colors)
├── services/              # Business logic
├── main.rs                # CLI entry point
└── ...
```


## 🤝 Contributing

Contributions welcome! This fork focuses on CLI functionality.

**Before submitting PRs:**
- ✅ Pass format check: `cargo fmt --check`
- ✅ Pass linter: `cargo clippy`
- ✅ Pass tests: `cargo test`
- 💡 Open an issue for discussion first

---

## 📜 License

- MIT © Original Author: Jason Young
- CLI Fork Maintainer: saladday
