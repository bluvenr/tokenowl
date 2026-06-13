<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="TokenOwl" width="128" />
  <h1>TokenOwl</h1>
  <p><strong>AI Coding Cost Tracker</strong> — CC Switch's Data Analysis Partner</p>
  <p>
    <a href="README.md">English</a> | <a href="README.zh-CN.md">中文</a>
  </p>
  <p>
    <img src="https://img.shields.io/badge/Tauri-v2-blue" alt="Tauri v2" />
    <img src="https://img.shields.io/badge/React-19-blue" alt="React 19" />
    <img src="https://img.shields.io/badge/Rust-2021-orange" alt="Rust 2021" />
    <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License" />
  </p>
  <p>
    <a href="https://bluvenr.github.io/tokenowl/">🌐 Project Website</a>
  </p>
</div>

---

## Overview

TokenOwl is a desktop application for tracking AI coding assistant costs by analyzing proxy data from [CC Switch](https://github.com/farion1231/cc-switch). It provides detailed usage analytics, budget management, and cost attribution across multiple AI providers.

## Features

- **Dashboard** — Real-time usage summary with cost, token count, and request statistics
- **Cost Attribution** — Break down costs by model, provider, and token type (input/output/cache)
- **Budget Management** — Set daily/weekly/monthly spending limits with configurable alert thresholds
- **Provider Analytics** — Track usage distribution and health across AI providers
- **CC Switch Integration** — One-click data sync from CC Switch proxy logs
- **Tray Mini Mode** — Compact always-on-top widget showing today's token usage and budget progress
- **Data Export** — Export usage data as CSV or JSON
- **i18n** — Full English and Chinese language support with system auto-detection
- **Auto Update** — Built-in update checker for new releases

## Platform Support

| Platform | Status |
|----------|--------|
| macOS (Apple Silicon / Intel) | ✅ Supported |
| Windows (x64) | ✅ Supported |
| Linux (x64) | ✅ Supported |

## Screenshots

| Dashboard | Cost Attribution | Budget Management |
|-----------|-----------------|------------------|
| ![Dashboard](pages/assets/screenshots/dashboard.png) | ![Cost Attribution](pages/assets/screenshots/cost-attribution.png) | ![Budget](pages/assets/screenshots/budget.png) |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://tauri.app/) |
| Frontend | React 19 + TypeScript |
| UI | TailwindCSS 4 + Radix UI + Lucide Icons |
| State | Zustand 5 |
| Charts | Recharts |
| i18n | i18next |
| Backend | Rust (rusqlite, tokio, reqwest) |
| Database | SQLite (bundled) |

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.75
- [pnpm](https://pnpm.io/) >= 9

Platform-specific system dependencies:

**macOS**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian)**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows**
- [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)

> See [Tauri Prerequisites](https://tauri.app/start/prerequisites/) for full details.

### Setup

```bash
# Clone the repository
git clone https://github.com/bluvenr/tokenowl.git
cd tokenowl/tokenowl-app

# Install dependencies
pnpm install

# Start development
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Project Structure

```
app/
├── .github/workflows/      # GitHub Actions (Pages deploy)
├── pages/                  # GitHub Pages landing page (not bundled)
├── src/                    # Frontend source
│   ├── components/         # React components
│   │   ├── dashboard/      # Dashboard widgets
│   │   └── settings/       # Settings tabs
│   ├── pages/              # Page components (Dashboard, Settings, TrayPopup)
│   ├── stores/             # Zustand state stores
│   ├── i18n/               # Translation files (en, zh-CN)
│   ├── lib/                # Utilities and API wrappers
│   └── styles/             # Global CSS
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   ├── storage/        # SQLite database layer
│   │   ├── ccswitch/       # CC Switch integration
│   │   ├── pricing/        # Price registry & calculator
│   │   ├── models/         # Data models (usage, budget, settings)
│   │   ├── remote/         # Remote data fetching
│   │   ├── updater/        # Auto-update logic
│   │   ├── crash/          # Crash logging
│   │   └── data/           # Bundled data files (model_prices.json)
│   ├── capabilities/       # Tauri v2 permissions
│   └── tauri.conf.json     # Tauri configuration
└── index.html              # App entry HTML
```

## License

[MIT](LICENSE)
