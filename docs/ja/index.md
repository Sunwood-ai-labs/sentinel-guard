---
layout: home

hero:
  name: Sentinel Guard
  text: Windows 向けテレメトリ＆プライバシー監視
  tagline: 隠れテレメトリ送信先、他ツール認証ファイルの越境参照、アクティブ外部通信をローカル完結で高速検知・遮断します。
  image:
    src: /logo.svg
    alt: Sentinel Guard
  actions:
    - theme: brand
      text: はじめに
      link: /ja/guide/getting-started
    - theme: alt
      text: アーキテクチャ
      link: /ja/guide/architecture

features:
  - title: Rust 高速静的スキャナー
    details: SIMD リテラルアンカー検索で 275 MB 級の Electron .asar バンドルを約 0.4 秒で走査し、PostHog / Axiom / Sentry 等の送信先を抽出します。
  - title: 越境認証ファイル参照の検知
    details: ~/.codex/auth.json や ~/.claude.json、~/.aws/credentials、~/.ssh/id_* など他ツールの認証ファイル参照を自動検知してスコア化します。
  - title: ライブ監視＆ワンクリック対策
    details: 標準ユーザー権限のままアクティブ TCP 接続をプロセス単位で照合し、オプトアウト環境変数設定や UAC 付き hosts ブロックを 1 クリックで実行できます。
---
