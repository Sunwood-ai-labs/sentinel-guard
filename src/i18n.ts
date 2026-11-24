export type Locale = "ja" | "en";

export interface Messages {
  // Sidebar
  watchViews: string;
  navScanner: string;
  navLive: string;
  navVault: string;
  targetFolder: string;
  targetPlaceholder: string;
  scanFolderBtn: string;
  scanningBtn: string;
  trayStatus: string;
  trayHint: string;

  // Titlebar
  winMinimize: string;
  winMaximize: string;
  winHideToTray: string;

  // Topbar & Metrics
  titleScanner: string;
  titleLive: string;
  titleVault: string;
  rescan: string;
  rescanning: string;
  shortcutHint: string;
  metricScanned: string;
  metricHigh: string;
  metricHighSub: string;
  metricMedium: string;
  metricMediumSub: string;
  metricLive: string;

  // Scanner list & detail
  detectedApps: string;
  countUnit: (n: number) => string;
  scanningEmpty: string;
  noAppsEmpty: string;
  selectAppEmpty: string;
  endpointsShort: (n: number) => string;
  crossToolShort: (n: number) => string;
  blockAllHosts: string;
  exposureScore: string;
  breakdownCrossTool: string;
  breakdownUnblocked: string;
  breakdownOptOut: string;
  appliedSuffix: string;

  // Detail sections
  secCreds: string;
  noCredsFound: string;
  badgeCrossTool: string;
  badgeSelfTool: string;
  secEndpoints: string;
  noEndpointsFound: string;
  badgeBlocked: string;
  btnBlock: string;
  secEnvVars: string;
  noEnvVarsFound: string;
  currentValLabel: string;
  unsetVal: string;
  badgeApplied: string;
  btnApply: string;
  btnRemoveEnvTitle: string;

  // Live tab
  liveTcpTitle: string;
  autoRefresh10s: string;
  btnRefresh: string;
  thProcess: string;
  thPid: string;
  thRemoteIp: string;
  thPort: string;
  thTelemetry: string;
  liveDnsTitle: string;
  noDnsHits: string;
  thDomain: string;
  thVendor: string;
  thResolved: string;
  thTtl: string;
  thStatus: string;

  // Vault tab
  vaultTitle: string;
  vaultExistsRatio: (exists: number, total: number) => string;
  thCategory: string;
  thTarget: string;
  thPath: string;
  thState: string;
  thModified: string;
  badgeProtectedTarget: string;
  statusNotFound: string;

  // Errors
  scanFailed: (err: string) => string;
  liveFailed: (err: string) => string;
}

export const MESSAGES: Record<Locale, Messages> = {
  ja: {
    watchViews: "監視ビュー",
    navScanner: "アプリ静的スキャン",
    navLive: "ライブ監視",
    navVault: "機密ファイル対象",
    targetFolder: "対象フォルダ",
    targetPlaceholder: "空欄で Programs 全体",
    scanFolderBtn: "フォルダをスキャン",
    scanningBtn: "走査中",
    trayStatus: "トレイ常駐待機中",
    trayHint: "閉じるで格納 · 右クリックで終了",

    winMinimize: "最小化",
    winMaximize: "最大化 / 元に戻す",
    winHideToTray: "トレイに格納",

    titleScanner: "アプリ静的スキャン",
    titleLive: "ライブ通信監視",
    titleVault: "機密ファイル保護対象",
    rescan: "再スキャン",
    rescanning: "スキャン中",
    shortcutHint: "ショートカット: R",
    metricScanned: "スキャン済みアプリ",
    metricHigh: "高リスク検出",
    metricHighSub: "越境・送信",
    metricMedium: "中リスク検出",
    metricMediumSub: "テレメトリのみ",
    metricLive: "ライブ検知イベント",

    detectedApps: "検出アプリ",
    countUnit: (n) => `${n} 件`,
    scanningEmpty: "インストール済みアプリのバンドルを解析中",
    noAppsEmpty: "スキャン対象のアプリが見つかりませんでした",
    selectAppEmpty: "左のリストからアプリを選択してください",
    endpointsShort: (n) => `送信先 ${n}`,
    crossToolShort: (n) => `越境 ${n}`,
    blockAllHosts: "hosts で一括ブロック",
    exposureScore: "露出スコア (Exposure profile)",
    breakdownCrossTool: "越境認証ファイル参照",
    breakdownUnblocked: "未ブロック送信先",
    breakdownOptOut: "オプトアウト環境変数",
    appliedSuffix: "適用済",

    secCreds: "他ツール認証・機密ファイル参照",
    noCredsFound: "他ツール認証ファイルへの参照は検出されませんでした",
    badgeCrossTool: "越境アクセス兆候",
    badgeSelfTool: "自社ツール参照",
    secEndpoints: "埋め込みテレメトリ送信先",
    noEndpointsFound: "既知のテレメトリ送信先 URL は検出されませんでした",
    badgeBlocked: "ブロック済",
    btnBlock: "ブロック",
    secEnvVars: "オプトアウト環境変数",
    noEnvVarsFound: "明確なオプトアウト環境変数は見つかりませんでした",
    currentValLabel: "現在値:",
    unsetVal: "(未設定)",
    badgeApplied: "適用済",
    btnApply: "適用",
    btnRemoveEnvTitle: "環境変数を削除",

    liveTcpTitle: "アクティブ外部通信 (HTTPS / OTLP)",
    autoRefresh10s: "10秒自動更新",
    btnRefresh: "更新",
    thProcess: "プロセス",
    thPid: "PID",
    thRemoteIp: "接続先 IP",
    thPort: "ポート",
    thTelemetry: "テレメトリ判定",
    liveDnsTitle: "DNS キャッシュ・テレメトリ履歴",
    noDnsHits: "DNS キャッシュ上にテレメトリドメインの直近解決履歴はありません",
    thDomain: "ドメイン",
    thVendor: "種別",
    thResolved: "解決先",
    thTtl: "TTL",
    thStatus: "状態",

    vaultTitle: "越境アクセス監視対象の機密ファイル",
    vaultExistsRatio: (e, t) => `${e} / ${t} 存在`,
    thCategory: "種別",
    thTarget: "対象",
    thPath: "パス",
    thState: "状態",
    thModified: "最終更新",
    badgeProtectedTarget: "要保護",
    statusNotFound: "未検出",

    scanFailed: (err) => `スキャン失敗: ${err}`,
    liveFailed: (err) => `ライブ監視更新エラー: ${err}`,
  },
  en: {
    watchViews: "Monitor",
    navScanner: "Static App Scan",
    navLive: "Live Monitor",
    navVault: "Credential Vault",
    targetFolder: "Target Folder",
    targetPlaceholder: "Blank for all Programs",
    scanFolderBtn: "Scan Folder",
    scanningBtn: "Scanning",
    trayStatus: "Tray resident ready",
    trayHint: "Close to hide · Right-click to quit",

    winMinimize: "Minimize",
    winMaximize: "Maximize / Restore",
    winHideToTray: "Hide to tray",

    titleScanner: "Static App Scan",
    titleLive: "Live Network Monitor",
    titleVault: "Protected Credential Files",
    rescan: "Rescan",
    rescanning: "Scanning",
    shortcutHint: "Shortcut: R",
    metricScanned: "Scanned Apps",
    metricHigh: "High Risk",
    metricHighSub: "Cross-tool / Ingest",
    metricMedium: "Medium Risk",
    metricMediumSub: "Telemetry only",
    metricLive: "Live Detections",

    detectedApps: "Detected Apps",
    countUnit: (n) => `${n} apps`,
    scanningEmpty: "Analyzing installed application bundles",
    noAppsEmpty: "No scannable apps found in the target folder",
    selectAppEmpty: "Select an application from the list",
    endpointsShort: (n) => `Endpoints ${n}`,
    crossToolShort: (n) => `Cross-tool ${n}`,
    blockAllHosts: "Block all in hosts",
    exposureScore: "Exposure profile",
    breakdownCrossTool: "Cross-tool credential refs",
    breakdownUnblocked: "Unblocked endpoints",
    breakdownOptOut: "Opt-out env vars",
    appliedSuffix: "applied",

    secCreds: "Cross-Tool Credential References",
    noCredsFound: "No references to external credential files detected",
    badgeCrossTool: "Cross-tool access",
    badgeSelfTool: "Self-tool ref",
    secEndpoints: "Embedded Telemetry Endpoints",
    noEndpointsFound: "No known telemetry endpoints detected",
    badgeBlocked: "Blocked",
    btnBlock: "Block",
    secEnvVars: "Opt-Out Environment Variables",
    noEnvVarsFound: "No explicit opt-out environment variables detected",
    currentValLabel: "Current:",
    unsetVal: "(unset)",
    badgeApplied: "Applied",
    btnApply: "Apply",
    btnRemoveEnvTitle: "Remove environment variable",

    liveTcpTitle: "Active Outbound Connections (HTTPS / OTLP)",
    autoRefresh10s: "Auto-refresh 10s",
    btnRefresh: "Refresh",
    thProcess: "Process",
    thPid: "PID",
    thRemoteIp: "Remote IP",
    thPort: "Port",
    thTelemetry: "Telemetry Match",
    liveDnsTitle: "DNS Cache Telemetry History",
    noDnsHits: "No recent telemetry domain resolutions found in Windows DNS cache",
    thDomain: "Domain",
    thVendor: "Vendor",
    thResolved: "Resolved IP",
    thTtl: "TTL",
    thStatus: "Status",

    vaultTitle: "Monitored Sensitive Credential Files",
    vaultExistsRatio: (e, t) => `${e} / ${t} present`,
    thCategory: "Category",
    thTarget: "Target",
    thPath: "Path",
    thState: "Status",
    thModified: "Last Modified",
    badgeProtectedTarget: "Protected",
    statusNotFound: "Not found",

    scanFailed: (err) => `Scan failed: ${err}`,
    liveFailed: (err) => `Live monitor failed: ${err}`,
  },
};

export function translateCredDescription(
  target: string,
  fallback: string,
  locale: Locale
): string {
  if (locale === "ja") return fallback;
  if (target.includes(".codex")) {
    return "References OpenAI Codex CLI account and token credentials";
  }
  if (target.includes(".claude.json")) {
    return "References Claude Code user ID and OAuth account metadata";
  }
  if (target.includes(".aws")) {
    return "References AWS CLI credentials file";
  }
  if (target.includes(".ssh")) {
    return "References local SSH private key path";
  }
  return fallback;
}

export function detectInitialLocale(): Locale {
  if (typeof window !== "undefined") {
    const params = new URLSearchParams(window.location.search);
    const q = params.get("lang");
    if (q === "en" || q === "ja") return q;
    const saved = window.localStorage.getItem("sentinel-guard-locale");
    if (saved === "en" || saved === "ja") return saved;
    if (navigator.language.toLowerCase().startsWith("ja")) return "ja";
  }
  return "en";
}
