# DEPLOYMENT.md — Production Path: Bare-Metal Alpine Linux

This document outlines the final steps to package MyOS v8.0 for a dedicated Alpine Linux deployment.

---

## 1. Environment Preparation
The build must be performed within an Alpine environment to ensure full musl libc compatibility.

```bash
# Install required build dependencies in Alpine
apk add --no-cache \
    curl \
    build-base \
    pkgconfig \
    gtk+3.0-dev \
    webkit2gtk-dev \
    libsoup-dev \
    javascriptcoregtk-dev \
    librsvg-dev \
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
