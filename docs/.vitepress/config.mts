import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Sentinel Guard",
  description:
    "Local-first Windows desktop monitor for hidden telemetry and cross-tool credential access",
  base: "/sentinel-guard/",
  head: [["link", { rel: "icon", type: "image/svg+xml", href: "/sentinel-guard/logo.svg" }]],
  themeConfig: {
    logo: "/logo.svg",
    socialLinks: [
      { icon: "github", link: "https://github.com/Sunwood-ai-labs/sentinel-guard" },
    ],
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2026 Sunwood AI Labs",
    },
  },
  locales: {
    root: {
      label: "English",
      lang: "en-US",
      themeConfig: {
        nav: [
          { text: "Home", link: "/" },
          { text: "Getting Started", link: "/guide/getting-started" },
          { text: "Architecture", link: "/guide/architecture" },
          { text: "Testing", link: "/guide/testing" },
        ],
        sidebar: [
          {
            text: "Guide",
            items: [
              { text: "Getting Started", link: "/guide/getting-started" },
              { text: "Architecture", link: "/guide/architecture" },
              { text: "Testing & Verification", link: "/guide/testing" },
            ],
          },
        ],
      },
    },
    ja: {
      label: "日本語",
      lang: "ja-JP",
      link: "/ja/",
      title: "Sentinel Guard",
      description:
        "Windows 向けローカル完結型テレメトリ＆他ツール認証ファイル越境アクセス監視アプリ",
      themeConfig: {
        nav: [
          { text: "ホーム", link: "/ja/" },
          { text: "はじめに", link: "/ja/guide/getting-started" },
          { text: "アーキテクチャ", link: "/ja/guide/architecture" },
          { text: "テスト・検証", link: "/ja/guide/testing" },
        ],
        sidebar: [
          {
            text: "ガイド",
            items: [
              { text: "はじめに", link: "/ja/guide/getting-started" },
              { text: "アーキテクチャ", link: "/ja/guide/architecture" },
              { text: "テスト・検証", link: "/ja/guide/testing" },
            ],
          },
        ],
      },
    },
  },
});
