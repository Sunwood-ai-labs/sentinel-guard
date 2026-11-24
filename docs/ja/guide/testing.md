# テスト・検証

Sentinel Guard は `test-fixtures/` 配下に再現性の高い模擬サンプルアプリを同梱しており、オフライン環境や CI 上でも検知精度を検証できます。

## 模擬フィクスチャ一覧

| フィクスチャ | 期待判定 | 検証内容 |
| :--- | :--- | :--- |
| `test-fixtures/mock-sneaky-ai-ide/app.js` | `HIGH (100)` | PostHog + Sentry 送信先、`~/.aws/credentials`・`~/.ssh/id_ed25519` 越境参照、オプトアウト環境変数 |
| `test-fixtures/mock-analytics-cli/bundle.mjs` | `MEDIUM (55)` | Segment + Mixpanel + Amplitude 送信先のみ（越境参照なし） |
| `test-fixtures/mock-clean-notepad/main.js` | `CLEAN (0)` | 無害なローカルメモ帳（ドキュメント URL コメントによる誤検知ゼロを確認） |
| `test-fixtures/live-tcp-probe.mjs` | ライブ検知 | データ送信ゼロで `us.i.posthog.com:443` へ生 TCP 接続のみ張り、PID 単位のライブ検知を検証 |

## テスト実行コマンド

ポータブルな標準テスト（CI で自動実行）：

```powershell
cd src-tauri
cargo test -- --nocapture
```

ローカル PC 実環境・ライブ TCP プローブを含む拡張テスト（`#[ignore]`）：

```powershell
cd src-tauri
cargo test -- --ignored --nocapture
```
