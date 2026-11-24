# Testing & Verification

Sentinel Guard includes deterministic mock fixtures under `test-fixtures/` so detection accuracy can be verified offline and in CI.

## Mock Fixtures

| Fixture | Expected Risk | Purpose |
| :--- | :--- | :--- |
| `test-fixtures/mock-sneaky-ai-ide/app.js` | `HIGH (100)` | PostHog + Sentry endpoints, `~/.aws/credentials` & `~/.ssh/id_ed25519` cross-tool refs, opt-out env |
| `test-fixtures/mock-analytics-cli/bundle.mjs` | `MEDIUM (55)` | Segment + Mixpanel + Amplitude endpoints only, zero cross-tool refs |
| `test-fixtures/mock-clean-notepad/main.js` | `CLEAN (0)` | Local-only app with a benign doc URL comment (verifies zero false positives) |
| `test-fixtures/live-tcp-probe.mjs` | Live TCP hit | Opens a raw TCP socket to `us.i.posthog.com:443` with zero payload bytes to verify PID-level live detection |

## Running Tests

Portable CI-safe suite:

```powershell
cd src-tauri
cargo test -- --nocapture
```

Extended local & live network tests (`#[ignore]`):

```powershell
cd src-tauri
cargo test -- --ignored --nocapture
```
