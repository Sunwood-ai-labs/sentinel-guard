export interface EndpointFinding {
  url: string;
  domain: string;
  vendor: string;
  sourceFile: string;
  isBlockedInHosts: boolean;
}

export interface CredentialRefFinding {
  target: string;
  category: string;
  description: string;
  sourceFile: string;
  isCrossTool: boolean;
}

export interface EnvVarFinding {
  name: string;
  recommendedValue: string;
  currentValue: string | null;
  isApplied: boolean;
}

export interface AppScanResult {
  appName: string;
  installDir: string;
  scannedFiles: string[];
  totalBytesScanned: number;
  riskLevel: "HIGH" | "MEDIUM" | "LOW" | "CLEAN";
  riskScore: number;
  telemetryEndpoints: EndpointFinding[];
  credentialRefs: CredentialRefFinding[];
  optOutEnvVars: EnvVarFinding[];
  detectedSdks: string[];
}

export interface ScanReport {
  scannedAt: string;
  scanRoots: string[];
  durationMs: number;
  totalApps: number;
  highRiskCount: number;
  mediumRiskCount: number;
  apps: AppScanResult[];
}

export interface DnsCacheEntry {
  domain: string;
  data: string;
  ttl: number;
  isTelemetry: boolean;
  matchedVendor: string | null;
  isBlockedInHosts: boolean;
}

export interface SensitiveFileStatus {
  path: string;
  label: string;
  category: string;
  exists: boolean;
  modifiedAt: string | null;
}

export interface ActiveConnectionInfo {
  pid: number;
  processName: string;
  remoteAddress: string;
  remotePort: number;
  state: string;
  telemetryDomain: string | null;
  telemetryVendor: string | null;
}

export interface LiveSnapshot {
  timestamp: string;
  dnsTotalEntries: number;
  dnsTelemetryHits: DnsCacheEntry[];
  sensitiveFiles: SensitiveFileStatus[];
  hostsBlockedDomains: string[];
  activeConnections: ActiveConnectionInfo[];
}

export interface ActionResult {
  ok: boolean;
  message: string;
}
