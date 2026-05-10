# PERMISSION_MANIFEST.md — Module Capability Register

**Version:** 8.0  
**Status:** Authoritative — source of truth for all module permissions  
**Last updated:** 2026-05

This document is the human-readable form of the `PermissionGuard` Rust struct. Every permission granted to every module is listed here with its justification. Any permission not listed here is denied by default.

---

## 1. Permission Definitions

| Permission | Access level | What it allows | Elevated |
|-----------|-------------|----------------|---------|
| `ExecRead` | Low | Receive stdout/stderr from exec commands | No |
| `ExecWrite` | Medium | Submit whitelisted commands to exec service | No |
| `FileRead` | Low | Read file contents within allowed root paths | No |
| `FileWrite` | Medium | Create, modify, delete files within allowed root paths | No |
| `FileWatch` | Low | Subscribe to filesystem change events | No |
| `ClipboardRead` | Low | Read current clipboard contents | No |
| `ClipboardWrite` | Low | Write to clipboard | No |
| `SystemMonitor` | Low | Read CPU, RAM, disk stats, process list | No |
| `NetworkRead` | Low | Read network interface state, connection info | No |
| `NetworkControl` | **High** | Modify network settings, toggle interfaces | **Yes** |
| `AuditRead` | Low | Query audit log entries | No |
| `PackageInstall` | **High** | Install or remove `.yops` packages | **Yes** |
| `PrivacyControl` | **High** | Control Privacy Shield state and settings | **Yes** |

Elevated permissions (`NetworkControl`, `PackageInstall`, `PrivacyControl`) are:
- Only grantable to `trusted: true` system modules
- Rejected at install time for third-party `.yops` packages
- Logged at `WARN` severity every time they are invoked

---

## 2. System Module Capability Table

### 2.1 Terminal — `com.yfitops.terminal`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `ExecWrite` | ✅ | Core purpose — runs whitelisted commands |
| `ExecRead` | ✅ | Must receive command output |
| `AuditRead` | ✅ | Shows execution history in session |
| `FileRead` | ❌ | Not needed — use File Browser for file ops |
| `FileWrite` | ❌ | Not in scope — terminal is read/exec only |
| `ClipboardRead` | ❌ | Not needed |
| `ClipboardWrite` | ❌ | Not needed |
| `SystemMonitor` | ❌ | Not needed — use Activity Monitor |
| `NetworkRead` | ❌ | Not needed |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PackageInstall` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Allowed exec commands:** `ls`, `cat`, `pwd`, `whoami`, `hostname`, `date`, `uptime`, `free`, `df`, `ps`

---

### 2.2 File Browser — `com.yfitops.files`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `FileRead` | ✅ | Core purpose — browse and read files |
| `FileWrite` | ✅ | Create, rename, delete, move files |
| `FileWatch` | ✅ | Live directory refresh on changes |
| `ClipboardWrite` | ✅ | Copy file paths and content to clipboard |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed — no command execution |
| `ClipboardRead` | ❌ | Not needed — paste from clipboard is a future feature (ADR required) |
| `SystemMonitor` | ❌ | Not needed |
| `NetworkRead` | ❌ | Not needed |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PackageInstall` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Allowed root paths:** `/myos`, `/home/<user>`, `/tmp` — no traversal above these roots

---

### 2.3 Activity Monitor — `com.yfitops.monitor`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `SystemMonitor` | ✅ | Core purpose — read CPU, RAM, disk, process list |
| `NetworkRead` | ✅ | Shows active network interfaces and usage |
| `AuditRead` | ❌ | Not needed — use Log Viewer |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed |
| `FileRead` | ❌ | Not needed |
| `FileWrite` | ❌ | Not needed |
| `ClipboardRead` | ❌ | Not needed |
| `ClipboardWrite` | ❌ | Not needed |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PackageInstall` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Poll interval:** 2 seconds via Tauri `emit()` from Rust `healthService`

---

### 2.4 Notes — `com.yfitops.notes`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `FileRead` | ✅ | Import notes from `.md` or `.txt` files |
| `ClipboardWrite` | ✅ | Copy note content to clipboard |
| `FileWrite` | ❌ | Notes stored in SQLite — no direct file writes |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed |
| `ClipboardRead` | ❌ | Not needed — paste is handled by the browser natively |
| `SystemMonitor` | ❌ | Not needed |
| `NetworkRead` | ❌ | Not needed |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PackageInstall` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Storage:** SQLite `notes` table + `notes_fts` FTS5 virtual table

---

### 2.5 Clipboard — `com.yfitops.clipboard`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `ClipboardRead` | ✅ | Core purpose — display current clipboard |
| `ClipboardWrite` | ✅ | Core purpose — write to clipboard |
| `FileRead` | ❌ | Not needed |
| `FileWrite` | ❌ | Not needed |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed |
| `SystemMonitor` | ❌ | Not needed |
| `NetworkRead` | ❌ | Not needed |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PackageInstall` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Note:** Clipboard history (storing previous clips) requires `FileWrite` or a dedicated table — needs a new ADR before implementation

---

### 2.6 Settings — `com.yfitops.settings`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `FileRead` | ✅ | Import/export settings as JSON file |
| `NetworkRead` | ✅ | Shows network info in the Privacy section |
| `PrivacyControl` | ✅ | Controls Privacy Shield from Settings UI |
| `FileWrite` | ❌ | Settings stored in SQLite — no direct file writes |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed |
| `ClipboardRead` | ❌ | Not needed |
| `ClipboardWrite` | ❌ | Not needed |
| `SystemMonitor` | ❌ | Not needed — use Activity Monitor |
| `NetworkControl` | ❌ | Elevated — controlled via Privacy Shield only |
| `PackageInstall` | ❌ | Elevated — use Package Manager |

**Trusted:** Yes  
**Storage:** SQLite `settings` table, profile-scoped

---

### 2.7 Log Viewer — `com.yfitops.logs`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `AuditRead` | ✅ | Core purpose — query and display audit log |
| `FileRead` | ❌ | Logs stored in SQLite — no file reads needed |
| `FileWrite` | ❌ | Not needed |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed |
| `ClipboardRead` | ❌ | Not needed |
| `ClipboardWrite` | ❌ | Not needed |
| `SystemMonitor` | ❌ | Not needed |
| `NetworkRead` | ❌ | Not needed |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PackageInstall` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Audit log is append-only.** The Log Viewer has no write access. It cannot delete, modify, or truncate log entries.

---

### 2.8 Package Manager — `com.yfitops.packages`

| Permission | Granted | Justification |
|-----------|---------|---------------|
| `PackageInstall` | ✅ | Core purpose — install and manage `.yops` packages |
| `FileRead` | ✅ | Read `.yops` package files from filesystem |
| `AuditRead` | ❌ | Not needed |
| `FileWrite` | ❌ | Package files managed by Rust service — no direct writes |
| `ExecRead` | ❌ | Not needed |
| `ExecWrite` | ❌ | Not needed |
| `ClipboardRead` | ❌ | Not needed |
| `ClipboardWrite` | ❌ | Not needed |
| `SystemMonitor` | ❌ | Not needed |
| `NetworkRead` | ❌ | Not needed — packages are local-only |
| `NetworkControl` | ❌ | Elevated — not granted |
| `PrivacyControl` | ❌ | Elevated — not granted |

**Trusted:** Yes  
**Note:** No network access. All package files are loaded from local filesystem. Online marketplace is explicitly out of scope for current phase.

---

## 3. Denied by Default

Any module (system or third-party) that has not been granted a permission will receive an `Err("Permission denied")` response from the Permission Guard. The denial is:

1. Returned immediately — no further logic executes
2. Logged to `audit_log` with `severity = 'WARN'`
3. Surfaced to the frontend as an error state in the calling module

There is no fallback, no soft failure, no default permission. Denied means denied.

---

## 4. Permission Request Protocol for Third-Party Modules

A `.yops` package that needs a permission must:

1. Declare it in the `permissions` array of its manifest JSON
2. Only request non-elevated permissions (`ExecRead`, `ExecWrite`, `FileRead`, `FileWrite`, `FileWatch`, `ClipboardRead`, `ClipboardWrite`, `SystemMonitor`, `NetworkRead`, `AuditRead`)
3. Accept that the Package Manager presents the full permission list to the user before installation

The user sees:
```
"Focus Timer" requests these permissions:
  • System Monitor — read CPU and RAM usage

[Deny]  [Allow and Install]
```

The user can deny installation if any permission is unacceptable. There is no way to install a package without reviewing its permissions.
