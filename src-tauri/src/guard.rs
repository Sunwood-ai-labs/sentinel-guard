use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use winreg::enums::*;
use winreg::RegKey;

use crate::models::ActionResult;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

/// Read all Windows User environment variables from HKCU\Environment
pub fn read_user_env_vars() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(env_key) = hkcu.open_subkey("Environment") {
        for item in env_key.enum_values().flatten() {
            let (name, val) = item;
            map.insert(name.to_uppercase(), val.to_string());
        }
    }
    map
}

/// Read blocked domains (0.0.0.0 or 127.0.0.1 or ::) from Windows hosts file
pub fn read_hosts_blocked_domains() -> HashSet<String> {
    let mut blocked = HashSet::new();
    let Ok(content) = fs::read_to_string(HOSTS_PATH) else {
        return blocked;
    };
    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(ip) = parts.next() else {
            continue;
        };
        if ip == "0.0.0.0" || ip == "127.0.0.1" || ip == "::" || ip == "::1" {
            for host in parts {
                blocked.insert(host.to_ascii_lowercase());
            }
        }
    }
    blocked
}

/// Set a Windows User environment variable permanently (via PowerShell .NET API so WM_SETTINGCHANGE fires)
pub fn set_user_env_var(name: &str, value: &str) -> ActionResult {
    if !is_safe_env_name(name) {
        return ActionResult {
            ok: false,
            message: format!("不正な環境変数名です: {}", name),
        };
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE) {
        Ok(env_key) => {
            if let Err(e) = env_key.set_value(name, &value) {
                return ActionResult {
                    ok: false,
                    message: format!("レジストリ書き込み失敗: {}", e),
                };
            }
        }
        Err(e) => {
            return ActionResult {
                ok: false,
                message: format!("HKCU\\Environment を開けませんでした: {}", e),
            };
        }
    }

    // Also broadcast environment change via .NET API (no console window)
    let script = format!(
        "[Environment]::SetEnvironmentVariable('{}', '{}', 'User')",
        name.replace('\'', "''"),
        value.replace('\'', "''")
    );
    let _ = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    ActionResult {
        ok: true,
        message: format!("{}={} をユーザー環境変数に設定しました（対象アプリを再起動すると反映されます）", name, value),
    }
}

/// Remove a Windows User environment variable
pub fn remove_user_env_var(name: &str) -> ActionResult {
    if !is_safe_env_name(name) {
        return ActionResult {
            ok: false,
            message: format!("不正な環境変数名です: {}", name),
        };
    }

    let script = format!(
        "[Environment]::SetEnvironmentVariable('{}', $null, 'User')",
        name.replace('\'', "''")
    );
    match Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(out) if out.status.success() => ActionResult {
            ok: true,
            message: format!("{} をユーザー環境変数から削除しました", name),
        },
        Ok(out) => ActionResult {
            ok: false,
            message: format!(
                "削除に失敗しました: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        },
        Err(e) => ActionResult {
            ok: false,
            message: format!("PowerShell 実行エラー: {}", e),
        },
    }
}

/// Add domains to Windows hosts file (requests UAC elevation via PowerShell Start-Process -Verb RunAs)
pub fn block_domains_in_hosts_elevated(domains: Vec<String>) -> ActionResult {
    let valid_domains: Vec<String> = domains
        .into_iter()
        .map(|d| d.trim().to_ascii_lowercase())
        .filter(|d| is_safe_domain(d))
        .collect();

    if valid_domains.is_empty() {
        return ActionResult {
            ok: false,
            message: "ブロック対象の有効なドメインがありません".to_string(),
        };
    }

    // Write a temporary PowerShell script and execute it elevated via UAC
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("sentinel-guard-hosts-update.ps1");

    let domain_list_ps = valid_domains
        .iter()
        .map(|d| format!("'{}'", d))
        .collect::<Vec<_>>()
        .join(", ");

    let ps_content = format!(
        r#"$ErrorActionPreference = 'Stop'
$hostsPath = '{hosts}'
$newDomains = @({domains})
$beginMarker = '# BEGIN SENTINEL-GUARD TELEMETRY BLOCK'
$endMarker = '# END SENTINEL-GUARD TELEMETRY BLOCK'

$content = if (Test-Path $hostsPath) {{ Get-Content -Path $hostsPath -Raw -Encoding UTF8 }} else {{ '' }}
$existing = @()
$pattern = "(?s)$([regex]::Escape($beginMarker)).*?$([regex]::Escape($endMarker))"
if ($content -match $pattern) {{
    $block = $Matches[0]
    foreach ($line in ($block -split "`r?`n")) {{
        if ($line -match '^\s*0\.0\.0\.0\s+([a-zA-Z0-9.-]+)') {{
            $existing += $Matches[1].ToLower()
        }}
    }}
    $content = [regex]::Replace($content, $pattern, '').TrimEnd()
}}

$merged = ($existing + $newDomains) | Sort-Object -Unique
$lines = @($beginMarker)
foreach ($d in $merged) {{
    $lines += "0.0.0.0 $d"
    $lines += ":: $d"
}}
$lines += $endMarker

$updated = ($content.TrimEnd() + "`r`n`r`n" + ($lines -join "`r`n") + "`r`n")
Set-Content -Path $hostsPath -Value $updated -Encoding UTF8 -NoNewline
ipconfig /flushdns | Out-Null
"#,
        hosts = HOSTS_PATH,
        domains = domain_list_ps
    );

    if let Err(e) = fs::write(&script_path, ps_content) {
        return ActionResult {
            ok: false,
            message: format!("一時スクリプト作成に失敗しました: {}", e),
        };
    }

    let elevate_cmd = format!(
        "$p = Start-Process -FilePath 'powershell.exe' -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','{}' -Verb RunAs -PassThru -Wait; exit $p.ExitCode",
        script_path.to_string_lossy().replace('\'', "''")
    );

    let result = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &elevate_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if Path::new(&script_path).exists() {
        let _ = fs::remove_file(&script_path);
    }

    match result {
        Ok(out) if out.status.success() => ActionResult {
            ok: true,
            message: format!(
                "{} 件のドメインを hosts でブロックし、DNS キャッシュをクリアしました",
                valid_domains.len()
            ),
        },
        Ok(out) => ActionResult {
            ok: false,
            message: format!(
                "UAC 昇格がキャンセルされたか失敗しました: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        },
        Err(e) => ActionResult {
            ok: false,
            message: format!("hosts 更新コマンドの起動に失敗しました: {}", e),
        },
    }
}

fn is_safe_env_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_safe_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && domain.contains('.')
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}
