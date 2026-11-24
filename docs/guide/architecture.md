# Architecture

Sentinel Guard is structured as a lightweight two-layer **Tauri 2** desktop application with zero external runtime dependencies.

## Layers

| Layer | Technology | Responsibility |
| :--- | :--- | :--- |
| **UI** | React 19 + TypeScript + Vite | Graphite dashboard, Exposure Profile 20-segment gauge, live tables, custom frameless titlebar |
| **Core** | Rust (`src-tauri/src/`) | High-speed `.asar`/JS static scanning, active TCP telemetry correlation, registry & `hosts` mitigations, system tray |

## Backend Modules

- `scanner.rs`: Uses SIMD literal-anchor patterns (`regex::bytes`) to scan Electron `resources/*.asar` archives and JS bundles. Distinguishes cross-tool credential references from self-references and computes a `0–100` risk score.
- `monitor.rs`: Resolves watched telemetry domains to IP maps and correlates them with active `Established` TCP connections (`Get-NetTCPConnection`) so live telemetry traffic is caught even under standard user privileges.
- `guard.rs`: Persists user opt-out environment variables in `HKCU\Environment` and applies `# BEGIN SENTINEL-GUARD` / `# END SENTINEL-GUARD` blocks to Windows `hosts` via UAC elevation with automatic DNS cache flush.
- `lib.rs`: Exposes async Tauri IPC commands and configures the Windows system tray (close hides to tray; tray menu exits cleanly).

## Zero External Requests Policy

- Fonts (`@fontsource/ibm-plex-sans-jp`, `@fontsource/ibm-plex-mono`) and background artwork are bundled locally inside `dist/`.
- No analytics, crash reporters, or external font CDNs are loaded by Sentinel Guard.
