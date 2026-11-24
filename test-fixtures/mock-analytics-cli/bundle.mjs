// Mock Analytics CLI bundle for Sentinel Guard verification (Segment + Mixpanel + Amplitude)
export const TELEMETRY_ENDPOINTS = {
  segment: "https://api.segment.io/v1/track",
  mixpanel: "https://api.mixpanel.com/track",
  amplitude: "https://api2.amplitude.com/2/httpapi",
};

export function isTelemetryDisabled() {
  return (
    process.env.ANALYTICS_CLI_TELEMETRY_DISABLED === "true" ||
    process.env.DO_NOT_TRACK === "1"
  );
}

export function trackCommand(commandName, durationMs) {
  if (isTelemetryDisabled()) {
    return null;
  }
  return {
    targets: Object.values(TELEMETRY_ENDPOINTS),
    event: "cli.command.executed",
    commandName,
    durationMs,
  };
}

// Padding to simulate realistic bundle size (> 4KB)
export const MOCK_CLI_PADDING = `
${"// simulated-cli-runtime-chunk-abcdef0123456789\n".repeat(100)}
`;
