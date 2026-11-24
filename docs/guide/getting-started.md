# Getting Started

## Prerequisites

- Windows 10 / 11 (x64)
- Node.js 20+
- Rust 1.85+ (`cargo`)
- Microsoft Edge WebView2 Runtime (preinstalled on Windows 11)

## Clone & Run

```powershell
git clone https://github.com/Sunwood-ai-labs/sentinel-guard.git
cd sentinel-guard
npm install
npm run tauri dev
```

## First Scan

1. On startup, Sentinel Guard automatically scans `%LOCALAPPDATA%\Programs` and ranks detected apps by their **Exposure Profile** score (`0–100`).
2. Select any app to inspect:
   - Cross-tool credential file references
   - Embedded telemetry endpoints
   - Undocumented opt-out environment variables
3. Use the one-click controls to apply user environment variables (`HKCU\Environment`) or block telemetry domains via Windows `hosts` (UAC-elevated).
4. To scan a custom directory (such as `test-fixtures/` or a local CLI folder), enter the folder path in the left sidebar and click **Scan Folder**.

## Build Production Installer

```powershell
npm run tauri build
```
