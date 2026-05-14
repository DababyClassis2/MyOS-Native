# Codebase Inventory & Path Mapping

## Core Project Structure

- **/home/builder/Desktop/Yfitops-Os/**: Primary project root.
    - **.github/workflows/**: GitHub Actions build configurations.
    - **MyOS-Native/**: The Tauri/Rust native implementation (Current Focus).
        - **src-tauri/**: Rust backend source.
            - **src/commands/**: System bridge modules (Files, Terminal, Health, etc.).
            - **src/state/**: Application global state and permission engine.
        - **src/**: Frontend UI source (HTML/CSS/JS).
            - **modules/**: Individual OS application views.
            - **assets/**: Global themes and icons.
    - **MyOS/**: Documentation and legacy design specs.
    - **system-docs/**: New technical documentation generated for Debian migration.

## Module Analysis (Debian Compatibility)

| Module | Status | Technical Notes |
|--------|--------|-----------------|
| **Terminal** | Needs Review | Currently calls `sh -c`. Debian's `dash` might require explicit `bash` calls for complex scripts. |
| **Packages** | Needs Update | Currently handles internal .yops manifests. Needs `apt` integration for Debian system packages. |
| **Health** | Verified | Uses `sysinfo` crate; fully compatible with Debian kernel. |
| **Files** | Verified | Uses standard Rust `fs`; sandbox logic is OS-agnostic. |
| **Database** | Verified | SQLite/sqlx is portable and stable on Debian/glibc. |

## Pending Tasks
1. Update `packages.rs` to support `apt-get` commands via whitelisted `exec`.
2. Document all sandbox paths used in `PermissionGuard`.
3. Map frontend WebView event listeners to ensure parity with Debian rendering.
