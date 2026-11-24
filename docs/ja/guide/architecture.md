# アーキテクチャ

Sentinel Guard は外部ランタイム依存を持たない、軽量な 2 層構成の **Tauri 2** デスクトップアプリケーションです。

## レイヤー構成

| レイヤー | 技術 | 役割 |
| :--- | :--- | :--- |
| **UI** | React 19 + TypeScript + Vite | Graphite ダッシュボード、20 セグメント Exposure Profile 計器、ライブ監視テーブル、カスタムタイトルバー |
| **Core** | Rust (`src-tauri/src/`) | 高速 `.asar` / JS 静的スキャン、アクティブ TCP テレメトリ照合、レジストリ・`hosts` 対策、タスクトレイ常駐 |

## バックエンドモジュール

- `scanner.rs`: SIMD リテラルアンカー検索（`regex::bytes`）で Electron の `resources/*.asar` や JS バンドルを走査します。他ツール認証ファイルへの越境参照と自社ツール参照を区別し、`0–100` のリスクスコアを算出します。
- `monitor.rs`: 監視対象テレメトリドメインの IP マップを構築し、`Get-NetTCPConnection` の確立済み接続とリアルタイム照合することで、標準ユーザー権限のままライブテレメトリ通信を検知します。
- `guard.rs`: `HKCU\Environment` へのユーザー環境変数の永続設定・削除、および UAC 昇格付きでの Windows `hosts` ブロック（`# BEGIN/END SENTINEL-GUARD` マーカー管理＋DNS フラッシュ）を担当します。
- `lib.rs`: 非同期 Tauri IPC コマンドの登録と、Windows タスクトレイ常駐（× ボタンでトレイ格納、右クリックメニューから完全終了）を制御します。

## ゼロ外部通信ポリシー

- フォント（`@fontsource/ibm-plex-sans-jp`, `@fontsource/ibm-plex-mono`）と背景ビジュアルはすべて `dist/` にローカルバンドルされます。
- Sentinel Guard 自身はアナリティクス、クラッシュレポート、外部フォント CDN へ一切通信しません。
