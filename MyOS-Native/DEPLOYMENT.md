# DEPLOYMENT.md — Production Path: Bare-Metal Debian Linux

This document outlines the final steps to package MyOS v8.0 for a dedicated Debian Linux deployment.

---

## 1. Environment Preparation
The build must be performed within an Debian environment to ensure full glibc compatibility.

For a minimal Debian installation, it is recommended to use a netinst image or `debootstrap` to create a base system without a desktop environment. This ensures the smallest possible attack surface and resource footprint.

```bash
# Update and install required build dependencies in Debian
apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libsoup2.4-dev \
    libjavascriptcoregtk-4.0-dev \
    librsvg2-dev \
    libssl-dev \
    cargo \
    nodejs \
    npm
```

## 2. Compiling the Native Binary
Compile the binary directly on the target architecture (x86_64 or aarch64) to ensure binary compatibility.

```bash
cd MyOS-Native
npm install
npm run tauri build
```

The resulting binary will be located at:
`src-tauri/target/release/yfitops-os`

## 3. System Configuration (Bare-Metal Boot)

### Replace Init
MyOS is designed to run as the primary shell. Configure `/etc/inittab` or your init system to execute the binary on boot:

```bash
# Example: Using a simple login-wrapper for autostart
/usr/bin/yfitops-os
```

### Security Hardening
1.  **Filesystem:** Ensure the root partition is encrypted with LUKS.
2.  **ReadOnly Root:** Configure the system root as read-only, mounting `yfitops.db` from a persistent (encrypted) data partition.
3.  **Kernel:** Use a hardened Linux kernel with minimal modules loaded.

---

## 4. Verification Checklist
- [ ] Binary runs without dependencies (check via `ldd yfitops-os`).
- [ ] `yfitops.db` is initialized on first boot.
- [ ] Permission Guard is correctly asserting capability on boot.
- [ ] Amnesic Mode is enabled if this is a ephemeral kiosk build.

---

*MyOS v8.0 is ready for field deployment. Ensure all security audit logs are regularly archived.*
