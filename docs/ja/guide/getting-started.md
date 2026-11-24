# はじめに

## 前提環境

- Windows 10 / 11 (x64)
- Node.js 20+
- Rust 1.85+（`cargo`）
- Microsoft Edge WebView2 Runtime（Windows 11 標準搭載）

## クローン＆起動

```powershell
git clone https://github.com/Sunwood-ai-labs/sentinel-guard.git
cd sentinel-guard
npm install
npm run tauri dev
```

## 初回スキャン

1. 起動すると `%LOCALAPPDATA%\Programs` を自動スキャンし、検出アプリを **露出スコア（Exposure Profile: 0–100）** 順に一覧表示します。
2. 任意のアプリを選択すると以下を確認できます：
   - 他ツール認証ファイルへの越境アクセス兆候
   - 埋め込みテレメトリ送信先 URL
   - コード内に隠されたオプトアウト環境変数
3. ワンクリック操作でユーザー環境変数（`HKCU\Environment`）への永続適用や、UAC 昇格付きでの Windows `hosts` 一括ブロックを行えます。
4. 任意のフォルダ（`test-fixtures/` やローカル CLI フォルダ等）を調べる場合は、左サイドバーの入力欄にパスを入れて **フォルダをスキャン** を押します。

## プロダクションビルド

```powershell
npm run tauri build
```
