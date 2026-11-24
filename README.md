# Sentinel Guard

**Windows デスクトップ向けのテレメトリ＆他ツール認証ファイル越境アクセス検知アプリ（Tauri 2 + Rust + React）**

> Local-first privacy & telemetry watch for Windows desktop apps, Electron bundles, and AI developer tools.

---

## 特徴

- **高速バイナリ / `.asar` 静的スキャナー（Rust）**
  - `%LOCALAPPDATA%\Programs` 配下や任意フォルダの Electron アプリ（`resources/*.asar`）や JS バンドルを SIMD リテラルアンカー検索で高速走査（275MB 級バンドルを約 0.4 秒で解析）
  - 埋め込みテレメトリ送信先（PostHog / Axiom / Sentry / Segment / Mixpanel / Amplitude / Datadog / `ingest.*` / `telemetry.*` 等）を自動抽出
  - 他ツールの機密・認証ファイル（`~/.codex/auth.json`, `~/.claude.json`, `~/.aws/credentials`, `~/.ssh/id_*`）への **越境アクセス兆候** を検知し、自社ツール参照と区別してスコア化
  - コード内に隠されたオプトアウト環境変数（`*_TELEMETRY_ENABLED`, `*_OTEL_SDK_DISABLED`, `DO_NOT_TRACK` 等）を自動検出し、現在の Windows 環境変数と照合
- **ライブ通信監視（標準ユーザー権限対応）**
  - アクティブ TCP 接続（HTTPS / OTLP ポート）と既知テレメトリ IP マップをリアルタイム照合し、プロセス名・PID・送信先ベンダーを可視化
  - Windows DNS キャッシュのテレメトリドメイン解決履歴チェック
- **ワンクリック対策**
  - 検出したオプトアウト環境変数の永続適用／削除（`HKCU\Environment`）
  - 検出したテレメトリドメインの `hosts` ブロック（UAC 昇格付き・DNS キャッシュ自動フラッシュ）
- **完全ローカル・ゼロ外部通信 UI**
  - カスタム Graphite タイトルバー＋タスクトレイ常駐対応
  - フォント（`IBM Plex Sans JP` / `IBM Plex Mono`）と背景ビジュアルをアプリ内に完全ローカルバンドルし、UI からの外部 CDN 通信を行いません

---

## アーキテクチャ

```text
sentinel-guard/
├── src/                      # React 19 + TypeScript + Vite フロントエンド
│   ├── App.tsx               # ダッシュボード・Exposure Profile 計器バー・ライブ監視
│   ├── App.css               # Graphite デザインシステム（カスタムタイトルバー含む）
│   └── assets/               # ローカルバンドル背景ビジュアル
├── src-tauri/                # Tauri 2 + Rust バックエンド
│   └── src/
│       ├── scanner.rs        # .asar / JS バンドル高速静的スキャナー
│       ├── monitor.rs        # アクティブ TCP 照合・DNS キャッシュ・機密ファイル監視
│       ├── guard.rs          # ユーザー環境変数設定・hosts ブロック（UAC 昇格）
│       ├── models.rs         # IPC データモデル
│       └── lib.rs            # Tauri コマンド＆タスクトレイ常駐
└── test-fixtures/            # 検証用模擬アプリ（HIGH / MEDIUM / CLEAN）＆安全な TCP プローブ
```

---

## クイックスタート

### 前提環境（Windows）
- Node.js 20+
- Rust 1.85+（`cargo`）
- Microsoft Edge WebView2 Runtime（Windows 11 標準搭載）

### 開発モードで起動

```powershell
git clone https://github.com/Sunwood-ai-labs/sentinel-guard.git
cd sentinel-guard
npm install
npm run tauri dev
```

### インストーラー / 実行ファイルのビルド

```powershell
npm run tauri build
```

---

## テスト

ポータブルな模擬サンプルアプリ（`test-fixtures/`）を使った静的スキャンテストを実行できます：

```powershell
cd src-tauri
cargo test -- --nocapture
```

ローカル PC の実インストール環境やライブ TCP プローブを含む拡張テストを実行する場合：

```powershell
cd src-tauri
cargo test -- --ignored --nocapture
```

---

## ロードマップ

- [x] **Phase 1 (MVP)**: Electron / JS バンドル高速静的スキャン、露出スコア計器 UI、ライブ TCP テレメトリ照合、ワンクリック環境変数オプトアウト＆`hosts` ブロック、タスクトレイ常駐
- [ ] **Phase 2**: Windows ETW（`Microsoft-Windows-Kernel-File` / `DNS-Client`）ヘルパーサービスによるリアルタイム越境ファイル読み取り検知＆デスクトップ通知
- [ ] **Phase 3**: npm global / MCP サーバー自動スキャン、Windows ファイアウォールルール管理

---

## ライセンス

[MIT License](LICENSE) — Copyright (c) 2026 Sunwood AI Labs  
UI refinement methodology credit: *unslop* by Damir Sokolovsky ([@heydamir](https://x.com/heydamir)), CC BY 4.0.
