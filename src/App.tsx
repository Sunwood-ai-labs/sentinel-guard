import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  FolderSearch,
  KeyRound,
  Lock,
  Minus,
  RefreshCw,
  Shield,
  Square,
  Trash2,
  X,
} from "lucide-react";
import type {
  ActionResult,
  AppScanResult,
  LiveSnapshot,
  ScanReport,
} from "./types";
import "./App.css";

type TabId = "scanner" | "live" | "vault";

const ICON_SIZE = 16;
const ICON_STROKE = 1.5;

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(0)} KB`;
  }
  return `${bytes} B`;
}

function riskDotClass(level: AppScanResult["riskLevel"]): string {
  switch (level) {
    case "HIGH":
      return "danger";
    case "MEDIUM":
      return "warn";
    case "CLEAN":
      return "ok";
    default:
      return "muted";
  }
}

export function App() {
  const [activeTab, setActiveTab] = useState<TabId>("scanner");
  const [report, setReport] = useState<ScanReport | null>(null);
  const [selectedAppName, setSelectedAppName] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);

  const [live, setLive] = useState<LiveSnapshot | null>(null);
  const [liveLoading, setLiveLoading] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);

  const [customPath, setCustomPath] = useState("");
  const [toast, setToast] = useState<{ ok: boolean; text: string } | null>(
    null
  );
  const [busyAction, setBusyAction] = useState<string | null>(null);

  const showToast = useCallback((ok: boolean, text: string) => {
    setToast({ ok, text });
    window.setTimeout(() => {
      setToast((prev) => (prev?.text === text ? null : prev));
    }, 5000);
  }, []);

  const runScan = useCallback(
    async (roots?: string[]) => {
      setScanning(true);
      try {
        const res = await invoke<ScanReport>("scan_apps", {
          customRoots: roots && roots.length > 0 ? roots : null,
        });
        setReport(res);
        setSelectedAppName((prev) => {
          if (prev && res.apps.some((a) => a.appName === prev)) return prev;
          return res.apps[0]?.appName ?? null;
        });
      } catch (err) {
        showToast(false, `スキャン失敗: ${String(err)}`);
      } finally {
        setScanning(false);
      }
    },
    [showToast]
  );

  const refreshLive = useCallback(async () => {
    setLiveLoading(true);
    try {
      const snap = await invoke<LiveSnapshot>("get_live_snapshot");
      setLive(snap);
    } catch (err) {
      showToast(false, `ライブ監視更新エラー: ${String(err)}`);
    } finally {
      setLiveLoading(false);
    }
  }, [showToast]);

  useEffect(() => {
    void runScan();
    void refreshLive();
  }, [runScan, refreshLive]);

  useEffect(() => {
    if (!autoRefresh) return;
    const id = window.setInterval(() => {
      void refreshLive();
    }, 10000);
    return () => window.clearInterval(id);
  }, [autoRefresh, refreshLive]);

  // Keyboard shortcut: R to rescan when not typing in an input
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      if (e.key.toLowerCase() === "r" && !e.ctrlKey && !e.metaKey && !scanning) {
        void runScan(customPath.trim() ? [customPath.trim()] : undefined);
        void refreshLive();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [customPath, refreshLive, runScan, scanning]);

  const selectedApp: AppScanResult | null = useMemo(() => {
    if (!report || !selectedAppName) return null;
    return report.apps.find((a) => a.appName === selectedAppName) ?? null;
  }, [report, selectedAppName]);

  const handleApplyEnv = async (name: string, value: string) => {
    setBusyAction(`env:${name}`);
    try {
      const res = await invoke<ActionResult>("apply_env_opt_out", {
        name,
        value,
      });
      showToast(res.ok, res.message);
      if (res.ok) await runScan();
    } catch (err) {
      showToast(false, String(err));
    } finally {
      setBusyAction(null);
    }
  };

  const handleRemoveEnv = async (name: string) => {
    setBusyAction(`env:${name}`);
    try {
      const res = await invoke<ActionResult>("remove_env_opt_out", { name });
      showToast(res.ok, res.message);
      if (res.ok) await runScan();
    } catch (err) {
      showToast(false, String(err));
    } finally {
      setBusyAction(null);
    }
  };

  const handleBlockDomains = async (domains: string[]) => {
    const unique = Array.from(new Set(domains));
    if (unique.length === 0) return;
    setBusyAction(`hosts:${unique.join(",")}`);
    try {
      const res = await invoke<ActionResult>("block_domains_in_hosts", {
        domains: unique,
      });
      showToast(res.ok, res.message);
      if (res.ok) {
        await Promise.all([runScan(), refreshLive()]);
      }
    } catch (err) {
      showToast(false, String(err));
    } finally {
      setBusyAction(null);
    }
  };

  const handleCustomScan = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = customPath.trim();
    if (!trimmed) {
      await runScan();
      return;
    }
    await runScan([trimmed]);
  };

  const liveTelemetryConnCount =
    live?.activeConnections.filter((c) => Boolean(c.telemetryDomain)).length ??
    0;
  const liveAlertCount =
    (live?.dnsTelemetryHits.length ?? 0) + liveTelemetryConnCount;

  const pageTitle =
    activeTab === "scanner"
      ? "アプリ静的スキャン"
      : activeTab === "live"
      ? "ライブ通信監視"
      : "機密ファイル保護対象";

  const appWindow = useMemo(() => getCurrentWindow(), []);

  return (
    <div className="app-frame">
      {/* Custom Graphite Titlebar (draggable) */}
      <div className="titlebar" data-tauri-drag-region>
        <div className="titlebar-left" data-tauri-drag-region>
          <Shield size={14} strokeWidth={ICON_STROKE} />
          <span className="titlebar-name" data-tauri-drag-region>
            Sentinel Guard
          </span>
          <span className="titlebar-sep" data-tauri-drag-region>
            /
          </span>
          <span className="titlebar-ctx" data-tauri-drag-region>
            {pageTitle}
          </span>
        </div>

        <div className="titlebar-controls">
          <button
            type="button"
            className="win-btn"
            onClick={() => void appWindow.minimize()}
            title="最小化"
            aria-label="最小化"
          >
            <Minus size={14} strokeWidth={1.5} />
          </button>
          <button
            type="button"
            className="win-btn"
            onClick={() => void appWindow.toggleMaximize()}
            title="最大化 / 元に戻す"
            aria-label="最大化"
          >
            <Square size={12} strokeWidth={1.5} />
          </button>
          <button
            type="button"
            className="win-btn close"
            onClick={() => void appWindow.hide()}
            title="トレイに格納"
            aria-label="トレイに格納"
          >
            <X size={14} strokeWidth={1.5} />
          </button>
        </div>
      </div>

      <div className="shell">
      {/* Sidebar */}
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            <Shield size={ICON_SIZE} strokeWidth={ICON_STROKE} />
          </div>
          <div>
            <div className="brand-title">Sentinel Guard</div>
            <div className="brand-sub">Telemetry &amp; Privacy Watch</div>
          </div>
        </div>

        <nav className="nav-group">
          <div className="sidebar-label">監視ビュー</div>
          <button
            className={`nav-item ${activeTab === "scanner" ? "active" : ""}`}
            onClick={() => setActiveTab("scanner")}
          >
            <FolderSearch size={ICON_SIZE} strokeWidth={ICON_STROKE} />
            <span>アプリ静的スキャン</span>
            {report && (
              <span
                className={`nav-count ${
                  report.highRiskCount > 0 ? "danger" : ""
                }`}
              >
                {report.highRiskCount}
              </span>
            )}
          </button>

          <button
            className={`nav-item ${activeTab === "live" ? "active" : ""}`}
            onClick={() => setActiveTab("live")}
          >
            <Activity size={ICON_SIZE} strokeWidth={ICON_STROKE} />
            <span>ライブ監視</span>
            <span className={`nav-count ${liveAlertCount > 0 ? "warn" : ""}`}>
              {liveAlertCount}
            </span>
          </button>

          <button
            className={`nav-item ${activeTab === "vault" ? "active" : ""}`}
            onClick={() => setActiveTab("vault")}
          >
            <KeyRound size={ICON_SIZE} strokeWidth={ICON_STROKE} />
            <span>機密ファイル対象</span>
            <span className="nav-count">
              {live?.sensitiveFiles.filter((f) => f.exists).length ?? 0}
            </span>
          </button>
        </nav>

        <div className="sidebar-section">
          <div className="sidebar-label">対象フォルダ</div>
          <form className="custom-scan-form" onSubmit={handleCustomScan}>
            <input
              type="text"
              value={customPath}
              onChange={(e) => setCustomPath(e.target.value)}
              placeholder="空欄で Programs 全体"
            />
            <button type="submit" className="btn" disabled={scanning}>
              {scanning ? "走査中" : "フォルダをスキャン"}
            </button>
          </form>
        </div>

        <div className="sidebar-footer">
          <div className="tray-status">
            <span className="dot ok" />
            <span>トレイ常駐待機中</span>
          </div>
          <div className="tray-hint">閉じるで格納 · 右クリックで終了</div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="main">
        <header className="topbar">
          <div className="page-heading">
            <h1>{pageTitle}</h1>
            {report && (
              <span className="mono muted">
                {report.durationMs} ms · {report.scannedAt}
              </span>
            )}
          </div>

          <div className="topbar-actions">
            <span className="keycap" title="ショートカット: R">
              R
            </span>
            <button
              className="btn primary"
              onClick={() => {
                void runScan(
                  customPath.trim() ? [customPath.trim()] : undefined
                );
                void refreshLive();
              }}
              disabled={scanning}
            >
              <RefreshCw
                size={ICON_SIZE}
                strokeWidth={ICON_STROKE}
                className={scanning ? "spin" : ""}
              />
              <span>{scanning ? "スキャン中" : "再スキャン"}</span>
            </button>
          </div>
        </header>

        {/* Compact Stat Row */}
        <section className="metrics">
          <div className="metric-card">
            <span className="metric-label">スキャン済みアプリ</span>
            <div className="metric-value-row">
              <span className="metric-value">{report?.totalApps ?? "—"}</span>
              <span className="metric-sub">Apps</span>
            </div>
          </div>
          <div className="metric-card">
            <span className="metric-label">高リスク検出</span>
            <div className="metric-value-row">
              <span
                className={`metric-value ${
                  (report?.highRiskCount ?? 0) > 0 ? "danger" : ""
                }`}
              >
                {report?.highRiskCount ?? 0}
              </span>
              <span className="metric-sub">越境・送信</span>
            </div>
          </div>
          <div className="metric-card">
            <span className="metric-label">中リスク検出</span>
            <div className="metric-value-row">
              <span
                className={`metric-value ${
                  (report?.mediumRiskCount ?? 0) > 0 ? "warn" : ""
                }`}
              >
                {report?.mediumRiskCount ?? 0}
              </span>
              <span className="metric-sub">テレメトリのみ</span>
            </div>
          </div>
          <div className="metric-card">
            <span className="metric-label">ライブ検知イベント</span>
            <div className="metric-value-row">
              <span
                className={`metric-value ${liveAlertCount > 0 ? "danger" : ""}`}
              >
                {liveAlertCount}
              </span>
              <span className="metric-sub">
                TCP {live?.activeConnections.length ?? 0}
              </span>
            </div>
          </div>
        </section>

        {toast && (
          <div className={`toast ${toast.ok ? "ok" : "err"}`}>
            {toast.ok ? (
              <CheckCircle2 size={ICON_SIZE} strokeWidth={ICON_STROKE} />
            ) : (
              <AlertTriangle size={ICON_SIZE} strokeWidth={ICON_STROKE} />
            )}
            <span>{toast.text}</span>
          </div>
        )}

        {/* TAB 1: Static App Scanner */}
        {activeTab === "scanner" && (
          <section className="split-view">
            {/* App List */}
            <div className="panel">
              <div className="panel-header">
                <h2>検出アプリ</h2>
                <span className="mono muted">
                  {report ? `${report.apps.length} 件` : "—"}
                </span>
              </div>

              <div className="app-list">
                {!report && scanning && (
                  <div className="empty-state">
                    インストール済みアプリのバンドルを解析中
                  </div>
                )}
                {report && report.apps.length === 0 && (
                  <div className="empty-state">
                    スキャン対象のアプリが見つかりませんでした
                  </div>
                )}
                {report?.apps.map((app) => {
                  const isSelected = app.appName === selectedAppName;
                  const crossCount = app.credentialRefs.filter(
                    (c) => c.isCrossTool
                  ).length;
                  return (
                    <button
                      key={app.installDir}
                      className={`app-row ${isSelected ? "selected" : ""}`}
                      onClick={() => setSelectedAppName(app.appName)}
                    >
                      <div className="app-row-top">
                        <div className="app-name-wrap">
                          <span
                            className={`dot ${riskDotClass(app.riskLevel)}`}
                          />
                          <span className="app-name">{app.appName}</span>
                        </div>
                        <span
                          className={`app-score ${app.riskLevel.toLowerCase()}`}
                        >
                          {app.riskScore}
                        </span>
                      </div>
                      <div className="app-row-meta">
                        <span>送信先 {app.telemetryEndpoints.length}</span>
                        {crossCount > 0 && (
                          <span className="meta-alert">
                            越境 {crossCount}
                          </span>
                        )}
                        <span>{formatBytes(app.totalBytesScanned)}</span>
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>

            {/* App Details */}
            <div className="panel">
              {!selectedApp ? (
                <div className="empty-state">
                  左のリストからアプリを選択してください
                </div>
              ) : (
                <AppDetailView
                  app={selectedApp}
                  busyAction={busyAction}
                  onApplyEnv={handleApplyEnv}
                  onRemoveEnv={handleRemoveEnv}
                  onBlockDomains={handleBlockDomains}
                />
              )}
            </div>
          </section>
        )}

        {/* TAB 2: Live Monitor */}
        {activeTab === "live" && (
          <section className="single-scroll">
            <div className="panel">
              <div className="panel-header">
                <div>
                  <h2>アクティブ外部通信 (HTTPS / OTLP)</h2>
                </div>
                <div className="inline-actions">
                  <label className="toggle-label">
                    <input
                      type="checkbox"
                      checked={autoRefresh}
                      onChange={(e) => setAutoRefresh(e.target.checked)}
                    />
                    <span>10秒自動更新</span>
                  </label>
                  <button
                    className="btn-xs"
                    onClick={() => void refreshLive()}
                    disabled={liveLoading}
                  >
                    <RefreshCw
                      size={14}
                      strokeWidth={ICON_STROKE}
                      className={liveLoading ? "spin" : ""}
                    />
                    <span>更新</span>
                  </button>
                </div>
              </div>

              <div className="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>プロセス</th>
                      <th>PID</th>
                      <th>接続先 IP</th>
                      <th>ポート</th>
                      <th>テレメトリ判定</th>
                    </tr>
                  </thead>
                  <tbody>
                    {live?.activeConnections.map((c, idx) => (
                      <tr
                        key={`${c.pid}-${c.remoteAddress}-${idx}`}
                        className={c.telemetryDomain ? "row-alert" : ""}
                      >
                        <td>{c.processName}</td>
                        <td className="mono muted">{c.pid}</td>
                        <td className="mono">{c.remoteAddress}</td>
                        <td className="mono muted">{c.remotePort}</td>
                        <td>
                          {c.telemetryDomain ? (
                            <span className="status-tag danger">
                              {c.telemetryVendor} · {c.telemetryDomain}
                            </span>
                          ) : (
                            <span className="muted">—</span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            <div className="panel">
              <div className="panel-header">
                <h2>DNS キャッシュ・テレメトリ履歴</h2>
                <span className="mono muted">
                  {live?.timestamp ?? "—"} · {live?.dnsTotalEntries ?? 0} 件
                </span>
              </div>

              {!live || live.dnsTelemetryHits.length === 0 ? (
                <div className="quiet-box">
                  <span className="dot ok" />
                  <span>
                    DNS キャッシュ上にテレメトリドメインの直近解決履歴はありません
                  </span>
                </div>
              ) : (
                <div className="table-wrap">
                  <table>
                    <thead>
                      <tr>
                        <th>ドメイン</th>
                        <th>種別</th>
                        <th>解決先</th>
                        <th>TTL</th>
                        <th>状態</th>
                      </tr>
                    </thead>
                    <tbody>
                      {live.dnsTelemetryHits.map((hit) => (
                        <tr key={`${hit.domain}-${hit.data}`}>
                          <td className="mono">{hit.domain}</td>
                          <td className="secondary">
                            {hit.matchedVendor ?? "Telemetry"}
                          </td>
                          <td className="mono muted">{hit.data}</td>
                          <td className="mono muted">{hit.ttl}s</td>
                          <td>
                            {hit.isBlockedInHosts ? (
                              <span className="status-tag ok">ブロック済</span>
                            ) : (
                              <button
                                className="btn-xs danger"
                                onClick={() =>
                                  void handleBlockDomains([hit.domain])
                                }
                              >
                                ブロック
                              </button>
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </section>
        )}

        {/* TAB 3: Sensitive Credential Vault Watch */}
        {activeTab === "vault" && (
          <section className="single-scroll">
            <div className="panel">
              <div className="panel-header">
                <h2>越境アクセス監視対象の機密ファイル</h2>
                <span className="mono muted">
                  {live?.sensitiveFiles.filter((f) => f.exists).length ?? 0} /{" "}
                  {live?.sensitiveFiles.length ?? 0} 存在
                </span>
              </div>
              <div className="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>種別</th>
                      <th>対象</th>
                      <th>パス</th>
                      <th>状態</th>
                      <th>最終更新</th>
                    </tr>
                  </thead>
                  <tbody>
                    {live?.sensitiveFiles.map((sf) => (
                      <tr key={sf.path}>
                        <td>
                          <span className="status-tag neutral">
                            {sf.category}
                          </span>
                        </td>
                        <td>{sf.label}</td>
                        <td className="mono secondary">{sf.path}</td>
                        <td>
                          {sf.exists ? (
                            <span className="status-tag warn">要保護</span>
                          ) : (
                            <span className="muted">未検出</span>
                          )}
                        </td>
                        <td className="mono muted">{sf.modifiedAt ?? "—"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </section>
        )}
      </main>
      </div>
    </div>
  );
}

interface AppDetailProps {
  app: AppScanResult;
  busyAction: string | null;
  onApplyEnv: (name: string, value: string) => Promise<void>;
  onRemoveEnv: (name: string) => Promise<void>;
  onBlockDomains: (domains: string[]) => Promise<void>;
}

function AppDetailView({
  app,
  busyAction,
  onApplyEnv,
  onRemoveEnv,
  onBlockDomains,
}: AppDetailProps) {
  const levelClass = app.riskLevel.toLowerCase();
  const filledSegments = Math.round(app.riskScore / 5);
  const crossCount = app.credentialRefs.filter((c) => c.isCrossTool).length;
  const unblockedCount = app.telemetryEndpoints.filter(
    (e) => !e.isBlockedInHosts
  ).length;
  const appliedEnvCount = app.optOutEnvVars.filter((v) => v.isApplied).length;

  return (
    <div className="detail-scroll">
      <div className="detail-header">
        <div>
          <div className="detail-title-row">
            <h2>{app.appName}</h2>
            <span className={`status-tag ${riskDotClass(app.riskLevel)}`}>
              {app.riskLevel}
            </span>
          </div>
          <div className="mono path-line">{app.installDir}</div>
          {app.detectedSdks.length > 0 && (
            <div className="sdk-tags">
              {app.detectedSdks.map((sdk) => (
                <span key={sdk} className="sdk-tag">
                  {sdk}
                </span>
              ))}
            </div>
          )}
        </div>

        {unblockedCount > 0 && (
          <button
            className="btn danger"
            disabled={busyAction !== null}
            onClick={() =>
              void onBlockDomains(
                app.telemetryEndpoints
                  .filter((e) => !e.isBlockedInHosts)
                  .map((e) => e.domain)
              )
            }
          >
            <Lock size={ICON_SIZE} strokeWidth={ICON_STROKE} />
            <span>hosts で一括ブロック</span>
          </button>
        )}
      </div>

      {/* Moment: Exposure Profile Precision Meter */}
      <div className="exposure-instrument">
        <div className="exposure-top">
          <span className="exposure-label">露出スコア (Exposure profile)</span>
          <div className="exposure-score-readout">
            <span className={`exposure-big ${levelClass}`}>
              {app.riskScore}
            </span>
            <span className="mono muted">/ 100</span>
          </div>
        </div>

        <div className="segment-bar" aria-label={`Risk score ${app.riskScore}`}>
          {Array.from({ length: 20 }, (_, i) => (
            <span
              key={i}
              className={`segment-cell ${
                i < filledSegments ? `filled ${levelClass}` : ""
              }`}
            />
          ))}
        </div>

        <div className="exposure-breakdown">
          <div className="breakdown-item">
            <span className="breakdown-label">越境認証ファイル参照</span>
            <span
              className={`breakdown-val ${crossCount > 0 ? "danger" : "ok"}`}
            >
              {crossCount} 件
            </span>
          </div>
          <div className="breakdown-item">
            <span className="breakdown-label">未ブロック送信先</span>
            <span
              className={`breakdown-val ${
                unblockedCount > 0 ? "warn" : "ok"
              }`}
            >
              {unblockedCount} / {app.telemetryEndpoints.length} 件
            </span>
          </div>
          <div className="breakdown-item">
            <span className="breakdown-label">オプトアウト環境変数</span>
            <span className="breakdown-val">
              {appliedEnvCount} / {app.optOutEnvVars.length} 適用済
            </span>
          </div>
        </div>
      </div>

      {/* Cross-tool Credential References */}
      <div className="detail-block">
        <div className="block-header">
          <h3>他ツール認証・機密ファイル参照</h3>
          <span className="mono muted">{app.credentialRefs.length} 件</span>
        </div>
        {app.credentialRefs.length === 0 ? (
          <div className="quiet-box">
            <span className="dot ok" />
            <span>他ツール認証ファイルへの参照は検出されませんでした</span>
          </div>
        ) : (
          <div className="card-stack">
            {app.credentialRefs.map((cred) => (
              <div
                key={cred.target}
                className={`finding-card ${cred.isCrossTool ? "danger" : ""}`}
              >
                <div className="finding-top">
                  <span className="finding-title">{cred.target}</span>
                  <span
                    className={`status-tag ${
                      cred.isCrossTool ? "danger" : "neutral"
                    }`}
                  >
                    {cred.isCrossTool ? "越境アクセス兆候" : "自社ツール参照"}
                  </span>
                </div>
                <div className="finding-desc">{cred.description}</div>
                <div className="finding-src mono">{cred.sourceFile}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Telemetry Endpoints */}
      <div className="detail-block">
        <div className="block-header">
          <h3>埋め込みテレメトリ送信先</h3>
          <span className="mono muted">
            {app.telemetryEndpoints.length} 件
          </span>
        </div>
        {app.telemetryEndpoints.length === 0 ? (
          <div className="quiet-box">
            <span className="dot ok" />
            <span>既知のテレメトリ送信先 URL は検出されませんでした</span>
          </div>
        ) : (
          <div className="card-stack">
            {app.telemetryEndpoints.map((ep) => (
              <div key={ep.url} className="finding-card">
                <div className="finding-top">
                  <span className="finding-title">{ep.domain}</span>
                  <div className="inline-actions">
                    <span className="status-tag neutral">{ep.vendor}</span>
                    {ep.isBlockedInHosts ? (
                      <span className="status-tag ok">ブロック済</span>
                    ) : (
                      <button
                        className="btn-xs danger"
                        disabled={busyAction !== null}
                        onClick={() => void onBlockDomains([ep.domain])}
                      >
                        ブロック
                      </button>
                    )}
                  </div>
                </div>
                <div className="finding-src mono">{ep.url}</div>
                <div className="finding-src mono">{ep.sourceFile}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Opt-out Environment Variables */}
      <div className="detail-block">
        <div className="block-header">
          <h3>オプトアウト環境変数</h3>
          <span className="mono muted">{app.optOutEnvVars.length} 件</span>
        </div>
        {app.optOutEnvVars.length === 0 ? (
          <div className="quiet-box">
            <span>明確なオプトアウト環境変数は見つかりませんでした</span>
          </div>
        ) : (
          <div className="card-stack">
            {app.optOutEnvVars.map((ev) => (
              <div key={ev.name} className="env-row">
                <div>
                  <div className="mono">
                    {ev.name}={ev.recommendedValue}
                  </div>
                  <div className="muted" style={{ fontSize: "12px" }}>
                    現在値:{" "}
                    <span className="mono">
                      {ev.currentValue ?? "(未設定)"}
                    </span>
                  </div>
                </div>
                <div className="inline-actions">
                  {ev.isApplied ? (
                    <>
                      <span className="status-tag ok">適用済</span>
                      <button
                        className="btn-xs ghost"
                        disabled={busyAction !== null}
                        onClick={() => void onRemoveEnv(ev.name)}
                        title="環境変数を削除"
                      >
                        <Trash2 size={14} strokeWidth={ICON_STROKE} />
                      </button>
                    </>
                  ) : (
                    <button
                      className="btn-xs primary"
                      disabled={busyAction !== null}
                      onClick={() =>
                        void onApplyEnv(ev.name, ev.recommendedValue)
                      }
                    >
                      適用
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
