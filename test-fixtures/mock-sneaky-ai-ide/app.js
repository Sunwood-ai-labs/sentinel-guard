// Mock Electron/AI IDE bundle for Sentinel Guard verification (does NOT send any real data)
import path from "node:path";
import os from "node:os";

const CONFIG = {
  appName: "Mock Sneaky AI IDE",
  version: "1.4.2",
  posthogEndpoint: "https://us.i.posthog.com/batch/",
  sentryDsn: "https://o123456.ingest.sentry.io/api/999/envelope/",
  telemetryEnabled: process.env.SNEAKY_AI_TELEMETRY_ENABLED !== "false",
};

export function collectCrossToolIdentity() {
  const home = os.homedir();
  // Simulated cross-tool credential path references
  const awsCredsPath = path.join(home, ".aws/credentials");
  const sshKeyPath = path.join(home, ".ssh/id_ed25519");
  return { awsCredsPath, sshKeyPath };
}

export function buildTelemetryPayload(eventName) {
  return {
    api_key: "phc_mock_test_key_only",
    batch: [
      {
        event: eventName,
        properties: {
          app: CONFIG.appName,
          version: CONFIG.version,
          platform: process.platform,
          arch: process.arch,
        },
      },
    ],
  };
}

// Padding to simulate realistic minified bundle size (> 4KB)
export const MOCK_BUNDLE_PADDING = `
${"// simulated-runtime-chunk-0123456789abcdef\n".repeat(95)}
`;
