---
layout: home

hero:
  name: Sentinel Guard
  text: Telemetry & Privacy Watch for Windows
  tagline: Detect hidden telemetry endpoints, cross-tool credential reads, and live outbound trackers—all locally with zero external requests.
  image:
    src: /logo.svg
    alt: Sentinel Guard
  actions:
    - theme: brand
      text: Getting Started
      link: /guide/getting-started
    - theme: alt
      text: Architecture
      link: /guide/architecture

features:
  - title: Fast Rust SIMD Scanner
    details: Scans 275 MB+ Electron .asar bundles in ~0.4s for PostHog, Axiom, Sentry, Segment, Mixpanel, Amplitude, and custom ingest URLs.
  - title: Cross-Tool Credential Detection
    details: Flags apps that reference other tools' auth files such as ~/.codex/auth.json, ~/.claude.json, ~/.aws/credentials, and ~/.ssh/id_*.
  - title: Live TCP Watch & One-Click Block
    details: Matches active HTTPS/OTLP connections by process name and PID without admin rights, and applies HKCU env opt-outs or UAC hosts blocks in one click.
---
