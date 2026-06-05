<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="TokenOwl" width="128" />
  <h1>TokenOwl</h1>
  <p><strong>AI 编码成本追踪器</strong> — CC Switch 数据分析伙伴</p>
  <p>
    <a href="README.md">English</a> | <a href="README.zh-CN.md">中文</a>
  </p>
  <p>
    <img src="https://img.shields.io/badge/Tauri-v2-blue" alt="Tauri v2" />
    <img src="https://img.shields.io/badge/React-19-blue" alt="React 19" />
    <img src="https://img.shields.io/badge/Rust-2021-orange" alt="Rust 2021" />
    <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License" />
  </p>
</div>

---

## 简介

TokenOwl 是一款桌面应用，通过分析 [CC Switch](https://github.com/anthropics/cc-switch) 代理数据来追踪 AI 编码助手的使用成本。提供详细的使用分析、预算管理和多供应商成本归因。

## 功能特性

- **仪表盘** — 实时使用摘要：成本、Token 数量、请求统计
- **成本归因** — 按模型、供应商、Token 类型（输入/输出/缓存）拆分成本
- **预算管理** — 设置每日/每周/每月消费限额，支持自定义告警阈值
- **供应商分析** — 追踪各 AI 供应商的使用分布和健康状态
- **CC Switch 集成** — 一键同步 CC Switch 代理日志数据
- **托盘迷你模式** — 紧凑的置顶悬浮窗，显示今日 Token 用量和预算进度
- **数据导出** — 支持 CSV 和 JSON 格式导出
- **国际化** — 完整的中英文支持，自动检测系统语言
- **自动更新** — 内置版本检查和更新提醒

## 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | [Tauri v2](https://tauri.app/) |
| 前端 | React 19 + TypeScript |
| UI | TailwindCSS 4 + Radix UI + Lucide Icons |
| 状态管理 | Zustand 5 |
| 图表 | Recharts |
| 国际化 | i18next |
| 后端 | Rust (rusqlite, tokio, reqwest) |
| 数据库 | SQLite (bundled) |

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.75
- [pnpm](https://pnpm.io/) >= 9

### 启动

```bash
# 克隆仓库
git clone https://github.com/bluvenr/tokenowl.git
cd tokenowl

# 安装依赖
pnpm install

# 启动开发
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

## 项目结构

```
app/
├── src/                    # 前端源码
│   ├── components/         # React 组件
│   │   ├── dashboard/      # 仪表盘组件
│   │   └── settings/       # 设置标签页
│   ├── pages/              # 页面组件 (App, TrayPopup)
│   ├── stores/             # Zustand 状态管理
│   ├── i18n/               # 翻译文件 (en, zh-CN)
│   ├── lib/                # 工具函数和 API 封装
│   └── styles/             # 全局样式
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── commands/       # Tauri 命令处理
│   │   ├── storage/        # SQLite 数据库层
│   │   ├── ccswitch/       # CC Switch 集成
│   │   ├── pricing/        # 价格注册表
│   │   └── ...
│   ├── capabilities/       # Tauri v2 权限配置
│   └── tauri.conf.json     # Tauri 配置
└── public/                 # 静态资源
```

## 许可证

[MIT](LICENSE)
