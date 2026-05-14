# Yfitops OS — Sovereign Desktop Shell

**Target Platform:** Minimal Debian Linux (glibc)  
**Architecture:** Tauri v2 + Rust + WebView  
**Philosophy:** Privacy-first, Local-first, User-Sovereign

## Project Overview

Yfitops OS is a native desktop shell designed to replace traditional desktop environments. It runs as a compiled Rust gateway that manages system resources and permissions, while the UI is rendered through an isolated, permission-gated WebView interface.

## Recent Migration: Alpine to Debian

The project has transitioned from an Alpine/musl base to a Debian/glibc base. This move improves hardware compatibility, stability, and access to the broader Debian package ecosystem while maintaining a minimal system footprint.

## Core Modules

- **Terminal:** Permission-gated command execution with whitelisting.
- **Files:** Sandbox-enforced file explorer and management.
- **Health:** Real-time system monitoring (CPU, RAM, Disk, Processes).
- **Packages:** Sovereign .yops module installer + Debian APT integration.
- **Security:** Immutable audit logs and a granular permission guard engine.

## Documentation

- **[system-docs/](./system-docs/):** Detailed technical documentation and path mappings.
- **[MyOS-Native/ARCHITECTURE.md](./MyOS-Native/ARCHITECTURE.md):** Deep dive into the system layers.
- **[MyOS-Native/TECHNICAL_SPEC.md](./MyOS-Native/TECHNICAL_SPEC.md):** Technical requirements and specifications.

## Development

```bash
cd MyOS-Native
npm install
npm run tauri dev
```

## Build & CI

Automated builds are performed via GitHub Actions, producing a Debian-compatible AppImage artifact for live testing.
