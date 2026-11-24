use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use chrono::{DateTime, Local};
use serde::Deserialize;

use crate::guard::read_hosts_blocked_domains;
use crate::models::{
    ActiveConnectionInfo, DnsCacheEntry, LiveSnapshot, SensitiveFileStatus,
};
use crate::scanner::classify_vendor;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Deserialize)]
struct RawDnsRow {
    #[serde(rename = "Entry")]
    entry: Option<String>,
    #[serde(rename = "Data")]
    data: Option<String>,
    #[serde(rename = "TimeToLive")]
    ttl: Option<u32>,
}

#[derive(Deserialize)]
struct RawConnRow {
    #[serde(rename = "Pid")]
    pid: Option<u32>,
    #[serde(rename = "ProcessName")]
    process_name: Option<String>,
    #[serde(rename = "RemoteAddress")]
    remote_address: Option<String>,
    #[serde(rename = "RemotePort")]
    remote_port: Option<u16>,
    #[serde(rename = "State")]
    state: Option<String>,
}

pub fn collect_live_snapshot() -> LiveSnapshot {
    let hosts_blocked = read_hosts_blocked_domains();
    let (dns_total, dns_hits) = read_dns_cache_telemetry(&hosts_blocked);
    let sensitive_files = check_sensitive_files();
    let active_connections = read_active_connections();

    let mut blocked_list: Vec<String> = hosts_blocked.into_iter().collect();
    blocked_list.sort();

    LiveSnapshot {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        dns_total_entries: dns_total,
        dns_telemetry_hits: dns_hits,
        sensitive_files,
        hosts_blocked_domains: blocked_list,
        active_connections,
    }
}

fn read_dns_cache_telemetry(
    hosts_blocked: &std::collections::HashSet<String>,
) -> (usize, Vec<DnsCacheEntry>) {
    let ps = "Get-DnsClientCache | Select-Object Entry, Data, TimeToLive | ConvertTo-Json -Compress";
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let Ok(out) = output else {
        return (0, Vec::new());
    };
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return (0, Vec::new());
    }

    let rows: Vec<RawDnsRow> = if text.starts_with('[') {
        serde_json::from_str(&text).unwrap_or_default()
    } else if text.starts_with('{') {
        serde_json::from_str::<RawDnsRow>(&text)
            .map(|r| vec![r])
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let total = rows.len();
    let mut hits = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for row in rows {
        let Some(domain_raw) = row.entry else {
            continue;
        };
        let domain = domain_raw.to_ascii_lowercase();
        let is_tel = is_telemetry_domain(&domain);
        if !is_tel {
            continue;
        }
        let data = row.data.unwrap_or_default();
        let key = format!("{}|{}", domain, data);
        if !seen.insert(key) {
            continue;
        }
        hits.push(DnsCacheEntry {
            matched_vendor: Some(classify_vendor(&domain)),
            is_blocked_in_hosts: hosts_blocked.contains(&domain),
            domain,
            data,
            ttl: row.ttl.unwrap_or(0),
            is_telemetry: true,
        });
    }

    hits.sort_by(|a, b| a.domain.cmp(&b.domain));
    (total, hits)
}

fn is_telemetry_domain(domain: &str) -> bool {
    let d = domain.to_ascii_lowercase();
    d.contains("posthog")
        || d.contains("axiom.co")
        || d.contains("sentry.io")
        || d.starts_with("ingest.")
        || d.contains(".ingest.")
        || d.contains("telemetry.")
        || d.contains(".telemetry.")
        || d.contains("analytics.")
        || d.contains("segment.io")
        || d.contains("mixpanel.com")
        || d.contains("amplitude.com")
        || d.contains("datadoghq.com")
        || d.contains("events.data.microsoft.com")
}

fn check_sensitive_files() -> Vec<SensitiveFileStatus> {
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    let home_path = PathBuf::from(home);

    let targets = vec![
        (".codex\\auth.json", "Codex CLI 認証トークン", "AI CLI"),
        (".claude.json", "Claude Code 認証・設定", "AI CLI"),
        (".gemini\\oauth_creds.json", "Gemini CLI 認証情報", "AI CLI"),
        (".aws\\credentials", "AWS 認証情報", "Cloud"),
        (".ssh\\id_ed25519", "SSH 秘密鍵 (ed25519)", "SSH"),
        (".ssh\\id_rsa", "SSH 秘密鍵 (RSA)", "SSH"),
        (".npmrc", "npm トークン設定", "Dev"),
    ];

    targets
        .into_iter()
        .map(|(rel, label, category)| {
            let full = home_path.join(rel);
            let meta = std::fs::metadata(&full).ok();
            let exists = meta.is_some();
            let modified_at = meta
                .and_then(|m| m.modified().ok())
                .map(|st| {
                    let dt: DateTime<Local> = st.into();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                });
            SensitiveFileStatus {
                path: full.to_string_lossy().to_string(),
                label: label.to_string(),
                category: category.to_string(),
                exists,
                modified_at,
            }
        })
        .collect()
}

fn build_telemetry_ip_map() -> HashMap<String, (String, String)> {
    const WATCHED_DOMAINS: &[&str] = &[
        "ingest.t3.codes",
        "us-east-1.aws.edge.axiom.co",
        "api.axiom.co",
        "us.i.posthog.com",
        "eu.i.posthog.com",
        "api.segment.io",
        "api.mixpanel.com",
        "api2.amplitude.com",
        "clerk-telemetry.com",
    ];

    let mut map = HashMap::new();
    for domain in WATCHED_DOMAINS {
        let vendor = classify_vendor(domain);
        if let Ok(addrs) = (*domain, 443).to_socket_addrs() {
            for addr in addrs {
                map.entry(addr.ip().to_string())
                    .or_insert_with(|| ((*domain).to_string(), vendor.clone()));
            }
        }
    }
    map
}

fn read_active_connections() -> Vec<ActiveConnectionInfo> {
    let ip_map = build_telemetry_ip_map();

    let ps = r#"
$procs = @{}
Get-Process -ErrorAction SilentlyContinue | ForEach-Object { $procs[$_.Id] = $_.ProcessName }
Get-NetTCPConnection -State Established -ErrorAction SilentlyContinue |
  Where-Object { $_.RemoteAddress -notmatch '^(127\.|::1|0\.0\.0\.0|::)' -and $_.RemotePort -in @(80, 443, 4317, 4318) } |
  Select-Object -First 80 @{N='Pid';E={$_.OwningProcess}}, @{N='ProcessName';E={$procs[[int]$_.OwningProcess]}}, RemoteAddress, RemotePort, @{N='State';E={$_.State.ToString()}} |
  ConvertTo-Json -Compress
"#;

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let Ok(out) = output else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return Vec::new();
    }

    let rows: Vec<RawConnRow> = if text.starts_with('[') {
        serde_json::from_str(&text).unwrap_or_default()
    } else if text.starts_with('{') {
        serde_json::from_str::<RawConnRow>(&text)
            .map(|r| vec![r])
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut conns: Vec<ActiveConnectionInfo> = rows
        .into_iter()
        .filter_map(|r| {
            let process_name = r.process_name.unwrap_or_else(|| "unknown".to_string());
            let remote_address = r.remote_address?;
            let (telemetry_domain, telemetry_vendor) = match ip_map.get(&remote_address) {
                Some((dom, vend)) => (Some(dom.clone()), Some(vend.clone())),
                None => (None, None),
            };
            Some(ActiveConnectionInfo {
                pid: r.pid.unwrap_or(0),
                process_name,
                remote_address,
                remote_port: r.remote_port.unwrap_or(443),
                state: r.state.unwrap_or_else(|| "Established".to_string()),
                telemetry_domain,
                telemetry_vendor,
            })
        })
        .collect();

    // Put detected telemetry connections at the top
    conns.sort_by(|a, b| {
        b.telemetry_domain
            .is_some()
            .cmp(&a.telemetry_domain.is_some())
            .then_with(|| a.process_name.cmp(&b.process_name))
    });
    conns
}
