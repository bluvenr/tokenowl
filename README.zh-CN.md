# TokenOwl

[English](README.md) | **[中文](README.zh-CN.md)**

> AI 编程费用追踪器 — 桌面小组件

TokenOwl 是一款轻量级桌面小组件，实时追踪你的 AI 编程工具使用费用。支持 5 款主流 AI 编程助手的数据采集，提供统一的费用仪表盘，包含费用明细、趋势图表和预算提醒。

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey)

## 支持的 AI 工具

| 工具 | 数据源路径 | 格式 |
|------|-----------|------|
| Claude Code | `~/.claude/projects/` | JSONL |
| Codex CLI | `~/.codex/sessions/` | JSONL |
| Gemini CLI | `~/.gemini/tmp/` | JSON |
| Kimi Code | `~/.kimi/sessions/` | JSONL |
| Qwen Code | `~/.qwen/history/` | JSON |

## 功能特性

- **系统托盘小组件** — 在菜单栏/通知区域常驻显示费用摘要
- **托盘弹窗** — 点击托盘图标弹出紧凑的费用概览面板
- **交互式仪表盘** — 饼图、面积图、柱状图，基于 Recharts 实现
- **实时追踪** — 文件监听器 2 秒防抖，数据即时更新
- **预算提醒** — 支持日/周/月预算，超阈值触发系统通知
- **费用引擎** — 多级价格合并（远程 CDN + 用户自定义，本地缓存）
- **数据导出** — CSV（UTF-8 BOM，Excel 兼容）和 JSON 格式
- **多语言** — 简体中文 / English，自动跟随系统语言
- **远程更新** — 自动版本检查与 CDN 价格同步
- **崩溃日志** — 本地 JSON 崩溃记录，一键生成 GitHub Issue
- **自定义模型价格** — 添加自定义模型，设置输入/输出/缓存价格
- **自定义监听路径** — 为非标准安装配置自定义数据源路径

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri v2（Rust 后端） |
| 前端 | React 19 + TypeScript |
| UI 组件 | shadcn/ui + Tailwind CSS v4 |
| 图表 | Recharts |
| 状态管理 | Zustand |
| 国际化 | react-i18next |
| 数据库 | SQLite（rusqlite） |
| 文件监听 | notify（Rust） |

## 快速开始

### 环境要求

- [Rust](https://www.rust-lang.org/tools/install)（stable 版本）
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)（推荐）

### 开发

```bash
# 克隆仓库
git clone https://github.com/bluvenr/tokenowl.git
cd tokenowl

# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev
```

### 构建

```bash
# 仅构建（不打包安装程序）
pnpm tauri build --no-bundle

# 构建并打包安装程序
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/`。

## 项目结构

```
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── collectors/     # 5 个数据源采集器
│   │   ├── commands/       # Tauri IPC 命令
│   │   ├── crash/          # 崩溃日志
│   │   ├── models/         # 数据模型
│   │   ├── pricing/        # 费用计算引擎
│   │   ├── remote/         # 远程配置与价格同步
│   │   ├── storage/        # SQLite 数据库层
│   │   ├── updater/        # 版本更新检查
│   │   └── watcher/        # 文件系统监听器
│   └── capabilities/       # Tauri 权限配置
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   │   ├── announcement/   # 远程公告横幅
│   │   ├── budget/         # 预算提醒 + Toast
│   │   ├── crash/          # 崩溃日志查看器
│   │   ├── dashboard/      # 主仪表盘（图表）
│   │   ├── settings/       # 6 标签页设置页
│   │   ├── tray/           # 托盘弹窗
│   │   ├── ui/             # 共享 UI 组件
│   │   └── update/         # 更新对话框
│   ├── hooks/              # React Hooks
│   ├── lib/                # 工具函数 & API 封装
│   ├── locales/            # i18n 翻译文件
│   ├── stores/             # Zustand 状态管理
│   └── styles/             # 全局样式
└── remote/                 # 远程配置文件（通过 CDN 分发）
    ├── prices.json         # 远程模型价格
    ├── config.json         # 功能开关与公告
    └── latest.json         # 最新版本信息
```

## 远程服务（零服务器架构）

TokenOwl 采用零服务器架构 — 所有远程服务均为通过 CDN 分发的静态 JSON 文件：

| 服务 | 文件 | CDN | 更新频率 |
|------|------|-----|---------|
| 版本检查 | `remote/latest.json` | jsDelivr | 每次发版 |
| 价格同步 | `remote/prices.json` | jsDelivr | 每 12 小时 |
| 配置/公告 | `remote/config.json` | jsDelivr | 每 6 小时 |

当 CDN 不可用时，自动回退到 GitHub Raw / Gitee Raw 地址。

## 使用指南

### 预算提醒

在「设置 > 预算」中设置日/周/月消费上限。当用量超过阈值（默认 80%）时，将触发：
- 应用内横幅通知
- 系统通知（如已启用）
- 托盘图标颜色变化（如已启用）

### 自定义模型价格

前往「设置 > 模型价格 > + 添加自定义模型」，添加内置价格表中没有的模型。

### 自定义数据路径

如果你的 AI 工具数据存放在非标准位置，前往「设置 > 数据源」设置自定义监听路径。

## 参与贡献

欢迎贡献代码！提交前请阅读以下说明：

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/amazing-feature`）
3. 提交更改（`git commit -m 'Add amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 发起 Pull Request

### 代码规范

- Rust：提交前运行 `cargo fmt` 和 `cargo clippy`
- TypeScript：遵循现有代码风格和约定

## 开源协议

本项目基于 Apache License 2.0 开源 — 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [Tauri](https://tauri.app/) — 轻量级桌面框架
- [shadcn/ui](https://ui.shadcn.com/) — 精美 UI 组件
- [Recharts](https://recharts.org/) — React 图表库
- 感谢所有 AI 编程工具团队提供可访问的费用数据
