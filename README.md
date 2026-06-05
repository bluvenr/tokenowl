# TokenOwl

**[English](README.md)** | [中文](README.zh-CN.md)

> AI Coding Cost Tracker — Desktop Widget

TokenOwl is a lightweight desktop widget that tracks your AI coding tool costs in real-time. It monitors usage across 5 popular AI coding assistants and provides a unified dashboard with cost breakdowns, trend charts, and budget alerts.

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey)

## Supported AI Tools

| Tool | Data Source | Format |
|------|------------|--------|
| Claude Code | `~/.claude/projects/` | JSONL |
| Codex CLI | `~/.codex/sessions/` | JSONL |
| Gemini CLI | `~/.gemini/tmp/` | JSON |
| Kimi Code | `~/.kimi/sessions/` | JSONL |

## Features

- **System Tray Widget** — Always-visible cost summary in your menu bar / notification area
- **Tray Popup** — Click the tray icon for a compact cost overview panel
- **Interactive Dashboard** — Pie charts, area charts, and bar charts powered by Recharts
- **Real-time Tracking** — File watcher with 2-second debounce for instant updates
- **Budget Alerts** — Daily/weekly/monthly budgets with system notifications
- **Cost Engine** — Multi-tier price merge (remote CDN + user custom, with local cache)
- **Data Export** — CSV (Excel-compatible with UTF-8 BOM) and JSON formats
- **i18n** — Chinese (Simplified) and English, auto-detects system language
- **Remote Updates** — Automatic version checking and price sync from CDN
- **Crash Logging** — Local JSON crash logs with one-click GitHub Issue reporting
- **Custom Model Pricing** — Add your own models with custom input/output/cache prices
- **Custom Watch Paths** — Override default data source paths for non-standard setups

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri v2 (Rust backend) |
| Frontend | React 19 + TypeScript |
| UI | shadcn/ui + Tailwind CSS v4 |
| Charts | Recharts |
| State | Zustand |
| i18n | react-i18next |
| Database | SQLite (rusqlite) |
| File Watcher | notify (Rust) |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/) (recommended)

### Development

```bash
# Clone the repository
git clone https://github.com/bluvenr/tokenowl.git
cd tokenowl

# Install dependencies
pnpm install

# Start development server
pnpm tauri dev
```

### Build

```bash
# Build for production (no installer bundle)
pnpm tauri build --no-bundle

# Build with installer
pnpm tauri build
```

Build outputs are in `src-tauri/target/release/`.

## Project Structure

```
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── collectors/     # 5 data source collectors
│   │   ├── commands/       # Tauri IPC commands
│   │   ├── crash/          # Crash logger
│   │   ├── models/         # Data models
│   │   ├── pricing/        # Cost calculation engine
│   │   ├── remote/         # Remote config & price sync
│   │   ├── storage/        # SQLite database layer
│   │   ├── updater/        # Version update checker
│   │   └── watcher/        # File system watcher
│   └── capabilities/       # Tauri permissions
├── src/                    # React frontend
│   ├── components/         # UI components
│   │   ├── announcement/   # Remote announcement banner
│   │   ├── budget/         # Budget alerts + Toast
│   │   ├── crash/          # Crash log viewer
│   │   ├── dashboard/      # Main dashboard with charts
│   │   ├── settings/       # 6-tab settings page
│   │   ├── tray/           # Tray popup window
│   │   ├── ui/             # Shared UI components
│   │   └── update/         # Update dialog
│   ├── hooks/              # React hooks
│   ├── lib/                # Utilities & API wrappers
│   ├── locales/            # i18n translations
│   ├── stores/             # Zustand state management
│   └── styles/             # Global CSS
└── remote/                 # Remote config files (served via CDN)
    ├── prices.json         # Remote model prices
    ├── config.json         # Feature flags & announcements
    └── latest.json         # Latest version info
```

## Remote Services (Zero-Server Architecture)

TokenOwl uses a zero-server architecture — all remote services are static JSON files served via CDN:

| Service | File | CDN | Update Frequency |
|---------|------|-----|-----------------|
| Version Check | `remote/latest.json` | jsDelivr | On release |
| Price Sync | `remote/prices.json` | jsDelivr | Every 12h |
| Config/Announcements | `remote/config.json` | jsDelivr | Every 6h |

Fallback: GitHub Raw URLs if CDN is unavailable.

## Configuration

### Budget Alerts

Set daily/weekly/monthly spending limits in Settings > Budget. When usage exceeds the threshold (default 80%), you'll receive:
- In-app banner notification
- System notification (if enabled)
- Tray icon color change (if enabled)

### Custom Model Prices

Go to Settings > Model Pricing > "+ Add Custom Model" to add models not in the built-in price table.

### Custom Data Paths

If your AI tools store data in non-standard locations, go to Settings > Data Sources and set custom watch paths.

## Contributing

Contributions are welcome! Please read the following before submitting:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Rust: Run `cargo fmt` and `cargo clippy` before committing
- TypeScript: Follow existing code style and conventions

## License

This project is licensed under the Apache License 2.0 — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) — Lightweight desktop framework
- [shadcn/ui](https://ui.shadcn.com/) — Beautiful UI components
- [Recharts](https://recharts.org/) — React charting library
- All the AI coding tool teams for making cost data accessible
