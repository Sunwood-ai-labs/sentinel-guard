use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointFinding {
    pub url: String,
    pub domain: String,
    pub vendor: String,
    pub source_file: String,
    pub is_blocked_in_hosts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRefFinding {
    pub target: String,
    pub category: String,
    pub description: String,
    pub source_file: String,
    pub is_cross_tool: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarFinding {
    pub name: String,
    pub recommended_value: String,
    pub current_value: Option<String>,
    pub is_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppScanResult {
    pub app_name: String,
    pub install_dir: String,
    pub scanned_files: Vec<String>,
    pub total_bytes_scanned: u64,
    pub risk_level: String,
    pub risk_score: u32,
    pub telemetry_endpoints: Vec<EndpointFinding>,
    pub credential_refs: Vec<CredentialRefFinding>,
    pub opt_out_env_vars: Vec<EnvVarFinding>,
    pub detected_sdks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub scanned_at: String,
    pub scan_roots: Vec<String>,
    pub duration_ms: u128,
    pub total_apps: usize,
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub apps: Vec<AppScanResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsCacheEntry {
    pub domain: String,
    pub data: String,
    pub ttl: u32,
    pub is_telemetry: bool,
    pub matched_vendor: Option<String>,
    pub is_blocked_in_hosts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveFileStatus {
    pub path: String,
    pub label: String,
    pub category: String,
    pub exists: bool,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveConnectionInfo {
    pub pid: u32,
    pub process_name: String,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub telemetry_domain: Option<String>,
    pub telemetry_vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSnapshot {
    pub timestamp: String,
    pub dns_total_entries: usize,
    pub dns_telemetry_hits: Vec<DnsCacheEntry>,
    pub sensitive_files: Vec<SensitiveFileStatus>,
    pub hosts_blocked_domains: Vec<String>,
    pub active_connections: Vec<ActiveConnectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}
