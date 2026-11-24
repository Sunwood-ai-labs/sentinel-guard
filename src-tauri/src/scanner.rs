use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::Local;
use regex::bytes::Regex;
use walkdir::WalkDir;

use crate::guard::{read_hosts_blocked_domains, read_user_env_vars};
use crate::models::{
    AppScanResult, CredentialRefFinding, EndpointFinding, EnvVarFinding, ScanReport,
};

const MAX_SCAN_FILE_BYTES: u64 = 350 * 1024 * 1024; // 350 MB (supports large server.asar)

pub fn default_scan_roots() -> Vec<PathBuf> {
    if let Ok(override_root) = std::env::var("SENTINEL_GUARD_SCAN_ROOT") {
        let p = PathBuf::from(override_root.trim());
        if p.exists() {
            return vec![p];
        }
    }
    let mut roots = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let programs = PathBuf::from(&local_app_data).join("Programs");
        if programs.exists() {
            roots.push(programs);
        }
    }
    roots
}

pub fn scan_installed_apps(custom_roots: Option<Vec<String>>) -> ScanReport {
    let started = Instant::now();
    let roots: Vec<PathBuf> = match custom_roots {
        Some(list) if !list.is_empty() => list.into_iter().map(PathBuf::from).collect(),
        _ => default_scan_roots(),
    };

    let env_map = read_user_env_vars();
    let hosts_blocked = read_hosts_blocked_domains();
    let patterns = CompiledPatterns::new();

    let mut apps = Vec::new();

    for root in &roots {
        if !root.exists() {
            continue;
        }
        // If root itself contains resources/*.asar, scan it directly as one app
        if root.join("resources").exists() {
            if let Some(res) = scan_single_app_dir(root, &patterns, &env_map, &hosts_blocked) {
                apps.push(res);
            }
            continue;
        }

        // Otherwise iterate immediate child directories as individual apps
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(res) =
                        scan_single_app_dir(&path, &patterns, &env_map, &hosts_blocked)
                    {
                        apps.push(res);
                    }
                }
            }
        }
    }

    // Sort by risk_score descending, then app_name ascending
    apps.sort_by(|a, b| {
        b.risk_score
            .cmp(&a.risk_score)
            .then_with(|| a.app_name.to_lowercase().cmp(&b.app_name.to_lowercase()))
    });

    let high_risk_count = apps.iter().filter(|a| a.risk_level == "HIGH").count();
    let medium_risk_count = apps.iter().filter(|a| a.risk_level == "MEDIUM").count();

    ScanReport {
        scanned_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        scan_roots: roots
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        duration_ms: started.elapsed().as_millis(),
        total_apps: apps.len(),
        high_risk_count,
        medium_risk_count,
        apps,
    }
}

struct CompiledPatterns {
    telemetry_domain_anchor: Regex,
    opt_out_anchor: Regex,
    sdk_rules: Vec<(&'static str, Regex)>,
    cred_codex_direct: Regex,
    cred_codex_dir: Regex,
    cred_auth_json: Regex,
    cred_claude_json: Regex,
    cred_aws: Regex,
    cred_ssh: Regex,
}

impl CompiledPatterns {
    fn new() -> Self {
        // Pure literal-alternation anchors: regex crate compiles these to SIMD Teddy/Aho-Corasick (GB/s)
        let telemetry_domain_anchor = Regex::new(
            r"posthog\.com|axiom\.co|sentry\.io|ingest\.|telemetry\.|analytics\.|segment\.io|segment\.com|mixpanel\.com|amplitude\.com|datadoghq\.com|bugsnag\.com|honeycomb\.io|logrocket\.com",
        )
        .expect("valid telemetry_domain_anchor regex");

        let opt_out_anchor = Regex::new(
            r"TELEMETRY_ENABLED|TELEMETRY_DISABLED|OTEL_SDK_DISABLED|ANALYTICS_ENABLED|ANALYTICS_DISABLED|DO_NOT_TRACK|DISABLE_TELEMETRY",
        )
        .expect("valid opt_out_anchor regex");

        let sdk_rules = vec![
            ("PostHog", Regex::new(r"posthog|PostHog").unwrap()),
            ("Axiom", Regex::new(r"axiom\.co|t3-code-relay-traces").unwrap()),
            ("OpenTelemetry", Regex::new(r"OTEL_SDK_DISABLED|OTLPTraceExporter|opentelemetry").unwrap()),
            ("Sentry", Regex::new(r"@sentry/|sentry\.io").unwrap()),
            ("Segment", Regex::new(r"api\.segment\.io|analytics-node").unwrap()),
            ("Mixpanel", Regex::new(r"api\.mixpanel\.com").unwrap()),
            ("Amplitude", Regex::new(r"api\.amplitude\.com|api2\.amplitude\.com").unwrap()),
        ];

        Self {
            telemetry_domain_anchor,
            opt_out_anchor,
            sdk_rules,
            cred_codex_direct: Regex::new(r"\.codex[/\\]auth\.json").unwrap(),
            cred_codex_dir: Regex::new(r#"["']\.codex["']"#).unwrap(),
            cred_auth_json: Regex::new(r#"["']auth\.json["']"#).unwrap(),
            cred_claude_json: Regex::new(r"\.claude\.json").unwrap(),
            cred_aws: Regex::new(r"\.aws[/\\]credentials").unwrap(),
            cred_ssh: Regex::new(r"\.ssh[/\\]id_(?:rsa|ed25519)").unwrap(),
        }
    }
}

fn scan_single_app_dir(
    app_dir: &Path,
    patterns: &CompiledPatterns,
    env_map: &HashMap<String, String>,
    hosts_blocked: &HashSet<String>,
) -> Option<AppScanResult> {
    let app_name = app_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| app_dir.to_string_lossy().to_string());

    let candidate_files = collect_scan_candidates(app_dir);
    if candidate_files.is_empty() {
        return None;
    }

    let mut scanned_files = Vec::new();
    let mut total_bytes_scanned = 0u64;

    let mut endpoints_map: BTreeMap<String, EndpointFinding> = BTreeMap::new();
    let mut cred_refs_map: BTreeMap<String, CredentialRefFinding> = BTreeMap::new();
    let mut env_vars_set: BTreeSet<String> = BTreeSet::new();
    let mut detected_sdks: BTreeSet<String> = BTreeSet::new();

    for file_path in candidate_files {
        let Ok(meta) = fs::metadata(&file_path) else {
            continue;
        };
        let len = meta.len();
        if len == 0 || len > MAX_SCAN_FILE_BYTES {
            continue;
        }
        let Ok(bytes) = fs::read(&file_path) else {
            continue;
        };

        let rel_name = file_path
            .strip_prefix(app_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();
        scanned_files.push(rel_name.clone());
        total_bytes_scanned += len;

        // 1. Telemetry URLs (anchor-based fast extraction)
        for m in patterns.telemetry_domain_anchor.find_iter(&bytes) {
            if let Some(url) = extract_enclosing_url(&bytes, m.start(), m.end()) {
                if is_noise_url(&url) {
                    continue;
                }
                let domain = extract_domain(&url);
                if domain.is_empty() || is_noise_domain(&domain) {
                    continue;
                }
                let vendor = classify_vendor(&domain);
                endpoints_map.entry(url.clone()).or_insert_with(|| EndpointFinding {
                    url,
                    is_blocked_in_hosts: hosts_blocked.contains(&domain),
                    domain,
                    vendor,
                    source_file: rel_name.clone(),
                });
            }
        }

        // 2. Opt-out env vars (anchor-based fast extraction)
        for m in patterns.opt_out_anchor.find_iter(&bytes) {
            if let Some(var_name) = extract_enclosing_env_var(&bytes, m.start(), m.end()) {
                if !is_noise_env_var(&var_name) {
                    env_vars_set.insert(var_name);
                }
            }
        }

        // 3. Telemetry SDK fingerprints
        for (sdk_name, rx) in &patterns.sdk_rules {
            if rx.is_match(&bytes) {
                detected_sdks.insert((*sdk_name).to_string());
            }
        }

        // 4. Cross-tool credential / sensitive file references
        check_credential_refs(&app_name, &rel_name, &bytes, patterns, &mut cred_refs_map);
    }

    if scanned_files.is_empty() {
        return None;
    }

    let telemetry_endpoints: Vec<EndpointFinding> = endpoints_map.into_values().collect();
    let credential_refs: Vec<CredentialRefFinding> = cred_refs_map.into_values().collect();

    let opt_out_env_vars: Vec<EnvVarFinding> = env_vars_set
        .into_iter()
        .map(|name| {
            let recommended = recommended_opt_out_value(&name).to_string();
            let current = env_map.get(&name.to_uppercase()).cloned();
            let is_applied = current
                .as_deref()
                .map(|v| v.eq_ignore_ascii_case(&recommended) || v == "1" && recommended == "true" || v == "0" && recommended == "false")
                .unwrap_or(false);
            EnvVarFinding {
                name,
                recommended_value: recommended,
                current_value: current,
                is_applied,
            }
        })
        .collect();

    // Compute risk score
    let mut risk_score: u32 = 0;

    let unblocked_endpoints = telemetry_endpoints
        .iter()
        .filter(|e| !e.is_blocked_in_hosts)
        .count();
    let cross_tool_creds = credential_refs
        .iter()
        .filter(|c| c.is_cross_tool)
        .count();
    let unapplied_opt_outs = opt_out_env_vars
        .iter()
        .filter(|v| !v.is_applied)
        .count();

    if !telemetry_endpoints.is_empty() {
        risk_score += (telemetry_endpoints.len() as u32 * 15).min(35);
    }
    if unblocked_endpoints > 0 {
        risk_score += 10;
    }
    if cross_tool_creds > 0 {
        risk_score += (cross_tool_creds as u32 * 35).min(55);
    }
    if unapplied_opt_outs > 0 && !telemetry_endpoints.is_empty() {
        risk_score += 10;
    }
    risk_score = risk_score.min(100);

    let risk_level = if cross_tool_creds > 0 || risk_score >= 60 {
        "HIGH"
    } else if risk_score >= 25 || !telemetry_endpoints.is_empty() {
        "MEDIUM"
    } else if !detected_sdks.is_empty() || !opt_out_env_vars.is_empty() {
        "LOW"
    } else {
        "CLEAN"
    }
    .to_string();

    Some(AppScanResult {
        app_name,
        install_dir: app_dir.to_string_lossy().to_string(),
        scanned_files,
        total_bytes_scanned,
        risk_level,
        risk_score,
        telemetry_endpoints,
        credential_refs,
        opt_out_env_vars,
        detected_sdks: detected_sdks.into_iter().collect(),
    })
}

fn collect_scan_candidates(app_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Priority 1: resources/*.asar (Electron main/server bundles)
    let resources_dir = app_dir.join("resources");
    if resources_dir.exists() {
        if let Ok(entries) = fs::read_dir(&resources_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        if ext.eq_ignore_ascii_case("asar") {
                            files.push(p);
                        }
                    }
                }
            }
        }
    }

    // Priority 2: If no .asar found, look for JS/MJS/CJS bundle files up to depth 3 (max 15 files)
    if files.is_empty() {
        for entry in WalkDir::new(app_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "asar" | "js" | "mjs" | "cjs") {
                    if let Ok(meta) = fs::metadata(p) {
                        if meta.len() > 256 && meta.len() < 30 * 1024 * 1024 {
                            files.push(p.to_path_buf());
                            if files.len() >= 15 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    files
}

fn check_credential_refs(
    app_name: &str,
    rel_name: &str,
    bytes: &[u8],
    patterns: &CompiledPatterns,
    out: &mut BTreeMap<String, CredentialRefFinding>,
) {
    let app_lower = app_name.to_ascii_lowercase();

    // Pattern 1: ~/.codex/auth.json (either direct path or ".codex" + "auth.json" in same bundle)
    let has_codex_direct = patterns.cred_codex_direct.is_match(bytes)
        || (patterns.cred_codex_dir.is_match(bytes) && patterns.cred_auth_json.is_match(bytes));
    if has_codex_direct {
        let is_cross = !app_lower.contains("codex") && !app_lower.contains("openai");
        out.entry("~/.codex/auth.json".to_string()).or_insert_with(|| {
            CredentialRefFinding {
                target: "~/.codex/auth.json".to_string(),
                category: "AI CLI 認証情報".to_string(),
                description: "OpenAI Codex CLI のアカウント・トークン情報への参照を検出"
                    .to_string(),
                source_file: rel_name.to_string(),
                is_cross_tool: is_cross,
            }
        });
    }

    // Pattern 2: ~/.claude.json
    if patterns.cred_claude_json.is_match(bytes) {
        let is_cross = !app_lower.contains("claude") && !app_lower.contains("anthropic");
        out.entry("~/.claude.json".to_string()).or_insert_with(|| {
            CredentialRefFinding {
                target: "~/.claude.json".to_string(),
                category: "AI CLI 認証情報".to_string(),
                description: "Claude Code のユーザーID・OAuthアカウント情報への参照を検出"
                    .to_string(),
                source_file: rel_name.to_string(),
                is_cross_tool: is_cross,
            }
        });
    }

    // Pattern 3: ~/.aws/credentials
    if patterns.cred_aws.is_match(bytes) {
        let is_cross = !app_lower.contains("aws");
        out.entry("~/.aws/credentials".to_string()).or_insert_with(|| {
            CredentialRefFinding {
                target: "~/.aws/credentials".to_string(),
                category: "クラウド認証情報".to_string(),
                description: "AWS 認証情報ファイルへの参照を検出".to_string(),
                source_file: rel_name.to_string(),
                is_cross_tool: is_cross,
            }
        });
    }

    // Pattern 4: SSH private keys
    if patterns.cred_ssh.is_match(bytes) {
        let is_cross = !app_lower.contains("ssh") && !app_lower.contains("git");
        out.entry("~/.ssh/id_*".to_string()).or_insert_with(|| {
            CredentialRefFinding {
                target: "~/.ssh/id_*".to_string(),
                category: "SSH 秘密鍵".to_string(),
                description: "SSH 秘密鍵パスへの参照を検出".to_string(),
                source_file: rel_name.to_string(),
                is_cross_tool: is_cross,
            }
        });
    }
}

fn extract_enclosing_url(bytes: &[u8], start: usize, end: usize) -> Option<String> {
    let left_limit = start.saturating_sub(64);
    let mut s = start;
    while s > left_limit {
        let b = bytes[s - 1];
        if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b':' | b'/') {
            s -= 1;
        } else {
            break;
        }
    }
    let slice_prefix = &bytes[s..start];
    // Find "http://" or "https://" in the prefix
    let http_pos = slice_prefix
        .windows(7)
        .position(|w| w == b"http://" || w == b"https:/")?;
    let url_start = s + http_pos;

    let right_limit = (end + 96).min(bytes.len());
    let mut e = end;
    while e < right_limit {
        let b = bytes[e];
        if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'/' | b':') {
            e += 1;
        } else {
            break;
        }
    }
    let url = std::str::from_utf8(&bytes[url_start..e]).ok()?;
    if url.starts_with("http://") || url.starts_with("https://") {
        Some(url.trim_end_matches(['.', '/', ':']).to_string())
    } else {
        None
    }
}

fn extract_enclosing_env_var(bytes: &[u8], start: usize, end: usize) -> Option<String> {
    let left_limit = start.saturating_sub(42);
    let mut s = start;
    while s > left_limit {
        let b = bytes[s - 1];
        if b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' {
            s -= 1;
        } else {
            break;
        }
    }
    let right_limit = (end + 16).min(bytes.len());
    let mut e = end;
    while e < right_limit {
        let b = bytes[e];
        if b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' {
            e += 1;
        } else {
            break;
        }
    }
    let name = std::str::from_utf8(&bytes[s..e]).ok()?;
    if name.len() >= 8 && name.len() <= 64 && name.as_bytes()[0].is_ascii_uppercase() {
        Some(name.to_string())
    } else {
        None
    }
}

fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split(['/', ':', '?', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

pub fn classify_vendor(domain: &str) -> String {
    let d = domain.to_ascii_lowercase();
    if d.contains("posthog") {
        "PostHog".to_string()
    } else if d.contains("axiom") {
        "Axiom (OTLP)".to_string()
    } else if d.contains("sentry") {
        "Sentry".to_string()
    } else if d.contains("segment") {
        "Segment".to_string()
    } else if d.contains("mixpanel") {
        "Mixpanel".to_string()
    } else if d.contains("amplitude") {
        "Amplitude".to_string()
    } else if d.contains("datadog") {
        "Datadog".to_string()
    } else if d.starts_with("ingest.") || d.contains(".ingest.") {
        "Custom Ingest".to_string()
    } else if d.contains("telemetry") {
        "Telemetry Endpoint".to_string()
    } else if d.contains("analytics") {
        "Analytics Endpoint".to_string()
    } else {
        "External Telemetry".to_string()
    }
}

fn is_noise_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("example.com")
        || lower.contains("localhost")
        || lower.contains("local.")
        || lower.contains("/docs/")
        || lower.contains("docs.sentry.io")
        || lower.contains("opentelemetry.io")
        || lower.contains("github.com")
        || lower.contains("schema")
        || lower.ends_with(".svg")
        || lower.ends_with(".png")
}

fn is_noise_domain(domain: &str) -> bool {
    !domain.contains('.')
        || domain.starts_with('.')
        || domain.contains("example.")
        || domain == "posthog.com"
        || domain == "www.posthog.com"
        || domain == "sentry.io"
        || domain == "docs.sentry.io"
        || domain == "axiom.co"
}

fn is_noise_env_var(var: &str) -> bool {
    var.starts_with("TEST_") || var.starts_with("MOCK_") || var.len() < 8
}

fn recommended_opt_out_value(var_name: &str) -> &'static str {
    let upper = var_name.to_ascii_uppercase();
    if upper.ends_with("_ENABLED") {
        "false"
    } else if upper == "DO_NOT_TRACK" {
        "1"
    } else {
        "true"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Scans local machine Programs directory"]
    fn smoke_scan_local_programs() {
        let report = scan_installed_apps(None);
        println!(
            "Scanned {} apps in {} ms (HIGH={}, MEDIUM={})",
            report.total_apps, report.duration_ms, report.high_risk_count, report.medium_risk_count
        );
        for app in &report.apps {
            println!(
                "  [{} {:>3}] {} | endpoints={} | creds={} | env_vars={} | sdks={:?}",
                app.risk_level,
                app.risk_score,
                app.app_name,
                app.telemetry_endpoints.len(),
                app.credential_refs.len(),
                app.opt_out_env_vars.len(),
                app.detected_sdks
            );
            for c in &app.credential_refs {
                println!("      CRED: {} (cross_tool={}) in {}", c.target, c.is_cross_tool, c.source_file);
            }
            for e in &app.telemetry_endpoints {
                println!("      URL : {} [{}] blocked={}", e.url, e.vendor, e.is_blocked_in_hosts);
            }
        }
    }

    #[test]
    #[ignore = "Requires local T3 Code installation"]
    fn verify_t3code_full_detection() {
        let t3_dir = std::env::var("LOCALAPPDATA")
            .map(|p| format!(r"{}\Programs\t3code", p))
            .unwrap();
        let report = scan_installed_apps(Some(vec![t3_dir.clone()]));
        assert_eq!(report.total_apps, 1, "t3code directory should be scanned");

        let t3 = &report.apps[0];
        println!("=== SENTINEL GUARD: T3 CODE DETECTION TEST ===");
        println!("App Name       : {}", t3.app_name);
        println!("Install Dir    : {}", t3.install_dir);
        println!("Scan Duration  : {} ms", report.duration_ms);
        println!("Scanned Files  : {:?} ({:.1} MB)", t3.scanned_files, t3.total_bytes_scanned as f64 / (1024.0 * 1024.0));
        println!("Risk Level     : {} (Score: {}/100)", t3.risk_level, t3.risk_score);
        println!("Detected SDKs  : {:?}", t3.detected_sdks);

        println!("\n[1] Cross-Tool Credential References ({}):", t3.credential_refs.len());
        for c in &t3.credential_refs {
            println!("  - {} | cross_tool={} | source={} | {}", c.target, c.is_cross_tool, c.source_file, c.description);
        }

        println!("\n[2] Telemetry Endpoints ({}):", t3.telemetry_endpoints.len());
        for e in &t3.telemetry_endpoints {
            println!("  - {} [{}] | domain={} | hosts_blocked={} | source={}", e.url, e.vendor, e.domain, e.is_blocked_in_hosts, e.source_file);
        }

        println!("\n[3] Opt-out Environment Variables ({}):", t3.opt_out_env_vars.len());
        for v in &t3.opt_out_env_vars {
            println!("  - {} (recommended={}) | current={:?} | applied={}", v.name, v.recommended_value, v.current_value, v.is_applied);
        }

        // Assertions for T3 Code telemetry & cross-tool behavior
        assert_eq!(t3.risk_level, "HIGH", "t3code must be flagged as HIGH risk");
        assert!(
            t3.telemetry_endpoints.iter().any(|e| e.domain == "ingest.t3.codes"),
            "must detect ingest.t3.codes"
        );
        assert!(
            t3.telemetry_endpoints.iter().any(|e| e.domain == "us-east-1.aws.edge.axiom.co"),
            "must detect Axiom OTLP trace endpoint"
        );
        assert!(
            t3.credential_refs.iter().any(|c| c.target == "~/.codex/auth.json" && c.is_cross_tool),
            "must detect cross-tool ~/.codex/auth.json reference"
        );
        assert!(
            t3.credential_refs.iter().any(|c| c.target == "~/.claude.json" && c.is_cross_tool),
            "must detect cross-tool ~/.claude.json reference"
        );
        assert!(
            t3.opt_out_env_vars.iter().any(|v| v.name == "T3CODE_TELEMETRY_ENABLED"),
            "must detect T3CODE_TELEMETRY_ENABLED"
        );
        assert!(
            t3.opt_out_env_vars.iter().any(|v| v.name == "T3CODE_OTEL_SDK_DISABLED"),
            "must detect T3CODE_OTEL_SDK_DISABLED"
        );

        // Also verify Live Monitor snapshot
        let live = crate::monitor::collect_live_snapshot();
        println!("\n=== SENTINEL GUARD: LIVE MONITOR SNAPSHOT ===");
        println!("Timestamp          : {}", live.timestamp);
        println!("DNS Cache Entries  : {} (Telemetry hits: {})", live.dns_total_entries, live.dns_telemetry_hits.len());
        for hit in &live.dns_telemetry_hits {
            println!("  DNS HIT: {} -> {} (vendor={:?}, hosts_blocked={})", hit.domain, hit.data, hit.matched_vendor, hit.is_blocked_in_hosts);
        }
        println!("Sensitive Files    :");
        for sf in &live.sensitive_files {
            if sf.exists {
                println!("  EXISTS : [{}] {} ({})", sf.category, sf.path, sf.modified_at.as_deref().unwrap_or("-"));
            }
        }
        println!("Active Connections : {}", live.active_connections.len());
        assert!(
            live.sensitive_files.iter().any(|s| s.path.ends_with(r".codex\auth.json") && s.exists),
            "live monitor must detect local .codex/auth.json"
        );
        assert!(
            live.sensitive_files.iter().any(|s| s.path.ends_with(".claude.json") && s.exists),
            "live monitor must detect local .claude.json"
        );
    }

    #[test]
    fn verify_mock_sample_apps_static_scan() {
        let fixtures_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("test-fixtures");

        // Static scan on all 3 mock sample apps (portable, runs in CI)
        let report = scan_installed_apps(Some(vec![fixtures_root.to_string_lossy().to_string()]));
        println!("=== MOCK SAMPLE APPS STATIC SCAN ({} ms) ===", report.duration_ms);
        assert_eq!(report.total_apps, 3, "should scan all 3 mock sample apps");

        // Verify mock-sneaky-ai-ide => HIGH
        let sneaky = report.apps.iter().find(|a| a.app_name == "mock-sneaky-ai-ide").unwrap();
        assert_eq!(sneaky.risk_level, "HIGH");
        assert!(sneaky.telemetry_endpoints.iter().any(|e| e.domain == "us.i.posthog.com"));
        assert!(sneaky.telemetry_endpoints.iter().any(|e| e.domain == "o123456.ingest.sentry.io"));
        assert!(sneaky.credential_refs.iter().any(|c| c.target == "~/.aws/credentials" && c.is_cross_tool));
        assert!(sneaky.credential_refs.iter().any(|c| c.target == "~/.ssh/id_*" && c.is_cross_tool));
        assert!(sneaky.opt_out_env_vars.iter().any(|v| v.name == "SNEAKY_AI_TELEMETRY_ENABLED"));

        // Verify mock-analytics-cli => MEDIUM (telemetry only, no cross-tool creds)
        let cli = report.apps.iter().find(|a| a.app_name == "mock-analytics-cli").unwrap();
        assert_eq!(cli.risk_level, "MEDIUM");
        assert_eq!(cli.credential_refs.len(), 0);
        assert!(cli.telemetry_endpoints.iter().any(|e| e.domain == "api.segment.io"));
        assert!(cli.telemetry_endpoints.iter().any(|e| e.domain == "api.mixpanel.com"));
        assert!(cli.telemetry_endpoints.iter().any(|e| e.domain == "api2.amplitude.com"));
        assert!(cli.opt_out_env_vars.iter().any(|v| v.name == "ANALYTICS_CLI_TELEMETRY_DISABLED"));

        // Verify mock-clean-notepad => CLEAN (zero false positives)
        let clean = report.apps.iter().find(|a| a.app_name == "mock-clean-notepad").unwrap();
        assert_eq!(clean.risk_level, "CLEAN");
        assert_eq!(clean.risk_score, 0);
        assert!(clean.telemetry_endpoints.is_empty());
        assert!(clean.credential_refs.is_empty());
    }

    #[test]
    #[ignore = "Opens live outbound TCP socket to verify active telemetry detection"]
    fn verify_live_tcp_telemetry_detection() {
        let fixtures_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("test-fixtures");

        // Live TCP telemetry connection detection test (sends ZERO bytes, pure TCP socket to PostHog IP)
        let probe_script = fixtures_root.join("live-tcp-probe.mjs");
        let mut child = std::process::Command::new("node")
            .arg(&probe_script)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("node should spawn live-tcp-probe.mjs");
        let child_pid = child.id();

        std::thread::sleep(std::time::Duration::from_millis(1200));

        let live = crate::monitor::collect_live_snapshot();
        let _ = child.kill();
        let _ = child.wait();

        let matched_conns: Vec<_> = live
            .active_connections
            .iter()
            .filter(|c| c.telemetry_domain.is_some())
            .collect();

        assert!(
            matched_conns.iter().any(|c| {
                c.pid == child_pid && c.telemetry_domain.as_deref() == Some("us.i.posthog.com")
            }),
            "live monitor must detect mock node process (PID={}) connecting to us.i.posthog.com",
            child_pid
        );
    }
}
