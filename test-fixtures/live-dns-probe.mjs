// Safe live DNS probe: resolves telemetry domains ONLY (sends ZERO HTTP requests / ZERO user data)
import dns from "node:dns/promises";

const TARGETS = [
  "us.i.posthog.com",
  "api.segment.io",
  "api.mixpanel.com",
];

async function main() {
  for (const host of TARGETS) {
    try {
      const res = await dns.lookup(host);
      console.log(`[DNS PROBE] Resolved ${host} -> ${res.address}`);
    } catch (err) {
      console.log(`[DNS PROBE] Failed ${host}: ${err.message}`);
    }
  }
}

await main();
