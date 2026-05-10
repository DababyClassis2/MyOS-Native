# THREAT_MODEL.md — Yfitops OS Security Threat Model

**Version:** 8.0  
**Classification:** Internal — Engineering reference  
**Last reviewed:** 2026-05  
**Review cadence:** Every major version, or after any security incident

---

## 1. Purpose

This document describes the assets Yfitops OS protects, the threats it defends against, the actors who pose those threats, and the mitigations in place or planned. It is the reference document for every security decision in the project.

A threat that is not in this document has not been assessed. When in doubt, add to this document first, then implement the mitigation.

---

## 2. Security Goals

In priority order:

1. **No data leaves the device without explicit user intent.** No telemetry, no crash reports, no background syncs.
2. **No module exceeds its declared permissions.** The permission guard is the enforcement mechanism.
3. **No user input reaches a shell interpreter unvalidated.** The exec whitelist is the enforcement mechanism.
4. **No persistent data survives an Amnesic session.** The session manager is the enforcement mechanism.
5. **No third-party package installs without user confirmation and signature validation.**

---

## 3. Assets

| Asset | Classification | Location | Impact if compromised |
|-------|---------------|----------|-----------------------|
| User notes | Sensitive | `yfitops.db` — `notes` table | Privacy violation |
| Settings (all profiles) | Sensitive | `yfitops.db` — `settings` table | Behaviour manipulation |
| Audit log | Sensitive | `yfitops.db` — `audit_log` table | Cover-up of attacker actions |
| Clipboard contents | High | `arboard` in-memory | Credential or data theft |
| Installed packages | Medium | `yfitops.db` — `packages` table | Malicious module execution |
| Exec whitelist | Critical | Hardcoded in `exec.rs` | Command injection if bypassed |
| Module manifests | Critical | Hardcoded in `permission_guard.rs` | Permission escalation |
| SQLite database file | High | `$APPDATA/yfitops/yfitops.db` | All persistent data exposure |
| Tauri IPC channel | Critical | Runtime | Arbitrary command execution |

---

## 4. Trust Boundaries

```
 ┌──────────────────────────────────────────────────────────────────────┐
 │  UNTRUSTED ZONE                                                      │
 │  ┌─────────────────────────┐   ┌────────────────────────────────┐  │
 │  │  .yops package content  │   │  Module HTML/JS (frontend)     │  │
 │  │  (unverified at rest)   │   │  (sandboxed WebView context)   │  │
 │  └────────────┬────────────┘   └───────────────┬────────────────┘  │
 └───────────────┼─────────────────────────────────┼───────────────────┘
                 │  manifest validation             │  Tauri IPC invoke()
                 ▼                                  ▼
 ┌──────────────────────────────────────────────────────────────────────┐
 │  SEMI-TRUSTED ZONE (Tauri Gateway)                                   │
 │  Permission Guard · Input validation · Audit logging                 │
 └───────────────────────────────────┬──────────────────────────────────┘
                                     │  validated, guarded call
                                     ▼
 ┌──────────────────────────────────────────────────────────────────────┐
 │  TRUSTED ZONE (Rust Core Services)                                    │
 │  exec · files · health · notes · settings · packages · logs          │
 └───────────────────────────────────┬──────────────────────────────────┘
                                     │  syscalls, file I/O
                                     ▼
 ┌──────────────────────────────────────────────────────────────────────┐
 │  KERNEL ZONE (Linux / Alpine)                                         │
 └──────────────────────────────────────────────────────────────────────┘
```

The Tauri IPC channel is the most critical trust boundary. Every `invoke()` call crosses from the untrusted WebView into trusted Rust. The Permission Guard is the gatekeeper at this crossing.

---

## 5. Threat Actors

| Actor | Motivation | Capability | Likelihood |
|-------|-----------|------------|------------|
| Malicious `.yops` package | Data theft, system access | JavaScript execution in WebView | Medium |
| Compromised module HTML | XSS, data exfiltration | DOM manipulation, invoke() calls | Medium |
| Local user (curiosity/accident) | Accidental data deletion or exposure | Full filesystem access on host | Low |
| Network attacker (no local access) | Data exfiltration | Network-level only | Low (local-first) |
| Supply chain attack on Rust deps | Backdoor, crypto mining | Compiler-level access | Low (small dep tree) |
| Physical attacker (device theft) | Data theft | Full disk access | Medium |

---

## 6. Threat Register

### T-001: Command Injection via exec Service

**Description:** A module passes user-controlled input as a shell command or argument to the exec service, allowing arbitrary command execution.

**Attack vector:** Frontend JS sends `{ cmd: "sh", args: ["-c", "rm -rf /"] }` to `exec_command`.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| Exec whitelist (hardcoded) | `ALLOWED_COMMANDS` const in `exec.rs` | ✅ Done |
| No shell interpreter allowed | `sh`, `bash`, `zsh` not in whitelist | ✅ Done |
| No argument interpolation | Args passed as `Vec<String>` to `Command::new()` | ✅ Done |
| Permission guard | `ExecWrite` required before any exec | ✅ Done |
| Audit logging | All exec calls logged | ✅ Done |

**Residual risk:** Low. The whitelist is compile-time. An attacker who can modify `exec.rs` already has code execution.

---

### T-002: Permission Escalation via Manifest Spoofing

**Description:** A `.yops` package declares elevated permissions (`NetworkControl`, `PackageInstall`) in its manifest, gaining capabilities beyond what an untrusted module is allowed.

**Attack vector:** Malicious `.yops` JSON includes `"permissions": ["PackageInstall", "NetworkControl"]`.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| Elevated permission block for untrusted modules | `install_package` rejects elevated perms on non-trusted | ✅ Done (spec) |
| Manifest signature verification | SHA-256 check before install | 🔶 Planned |
| User confirmation dialog | Modal before any package install | 🔶 Planned |
| Sandbox permissions at window level | Tauri `capabilities/` config per window | ✅ Done |

**Residual risk:** Medium until signature verification is implemented.

---

### T-003: XSS via API Response Data Injected into innerHTML

**Description:** A Rust command returns user-controlled text (e.g., a note's body or a filename), which is then rendered via `innerHTML` in a module's HTML, executing embedded scripts.

**Attack vector:** A note body contains `<script>window.__TAURI__.core.invoke('exec_command', ...)</script>`. The Notes module renders it via `el.innerHTML = note.body`.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| `textContent` mandate in JS contract | `CONTRIBUTING.md` rule | ✅ Documented |
| CSP headers on module windows | `tauri.conf.json` CSP policy | 🔶 Planned |
| No `eval()` allowed | `CONTRIBUTING.md` rule + clippy lint | ✅ Documented |
| `DOMPurify` for markdown preview | Notes module only, explicit ADR needed | 🔲 Future |

**Residual risk:** Medium — CSP implementation is the critical remaining control.

**CSP to implement:**
```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'none'"
}
```

---

### T-004: Data Persistence in Amnesic Mode

**Description:** In Amnesic Mode, data written during the session is accidentally persisted to disk, violating the session contract.

**Attack vector:** A module bypasses the SQLite pool and writes directly to a file via the `files` service. Or the in-memory pool is flushed to disk during a SQLite checkpoint.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| SQLite `:memory:` pool in Amnesic Mode | `SessionMode::Amnesic` switches pool | 🔶 Planned |
| `FileWrite` blocked in Amnesic Mode | Session manager checks mode before file writes | 🔶 Planned |
| Explicit pool zero on exit | `DROP TABLE` on all tables before process exit | 🔶 Planned |
| Visual indicator in UI | Amber top bar + ghost icon | 🔶 Planned |

**Residual risk:** High until implemented. Amnesic Mode is Phase 4.

---

### T-005: SQLite Injection via Dynamic Query Construction

**Description:** A Rust command builds a SQL query by concatenating user-controlled strings, allowing an attacker to modify the query structure.

**Attack vector:** `format!("SELECT * FROM notes WHERE title = '{}'", user_input)` where `user_input` is `'; DROP TABLE notes; --`.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| Prepared statements only | `sqlx::query!()` macro mandate | ✅ Documented |
| Code review rule | `CONTRIBUTING.md` SQL pattern | ✅ Documented |
| Clippy linting for format! in SQL | Custom lint rule (planned) | 🔲 Future |

**Residual risk:** Low. The `sqlx::query!()` macro statically checks queries at compile time.

---

### T-006: Unauthorised IPC Access from Untrusted Window

**Description:** A malicious web page opened in a secondary window calls Tauri commands using `window.__TAURI__`, bypassing module restrictions.

**Attack vector:** An HTML file opened via `open_module` injects a `<script>` that calls elevated commands using the exposed Tauri API.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| Tauri capability system | `capabilities/` JSON limits which windows call which commands | ✅ Done |
| Runtime Permission Guard | `module_id` verified against manifest at command execution | ✅ Done (spec) |
| WebView origin restrictions | Module URLs validated against allowed origin list | 🔶 Planned |

**Residual risk:** Medium until origin validation is implemented.

---

### T-007: Physical Disk Access (Device Theft)

**Description:** An attacker with physical access to the device reads `yfitops.db` directly, bypassing all application-level controls.

**Attack vector:** Boot from USB → mount filesystem → `cp yfitops.db /media/usb/`.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| SQLite encryption (SQLCipher) | Planned — requires ADR | 🔲 Future |
| Full-disk encryption (LUKS) | User responsibility (OS-level) | 🔲 Docs planned |
| Note: Amnesic Mode does not protect against this | Amnesic wipes on app exit, not disk | ⚠️ Known gap |

**Residual risk:** High until SQLCipher integration. Users should enable full-disk encryption.

---

### T-008: Dependency Supply Chain Attack

**Description:** A Rust crate or npm package used by the project is compromised upstream, introducing malicious code into the build.

**Mitigations:**
| Mitigation | Implementation | Status |
|-----------|---------------|--------|
| `Cargo.lock` committed | Locks exact dependency versions | ✅ Done |
| `package-lock.json` committed | Locks exact npm versions | ✅ Done |
| Minimal dependency policy | New deps require ADR | ✅ Documented |
| `cargo audit` in CI | Checks for known vulnerabilities | 🔶 Planned |

**Residual risk:** Low-Medium. Small dep tree reduces attack surface.

---

## 7. Mitigations Summary

| Control | Protects against | Status |
|---------|-----------------|--------|
| Exec whitelist (hardcoded) | T-001 | ✅ |
| Permission Guard | T-001, T-002, T-006 | ✅ Spec / 🔶 Partial impl |
| Prepared SQL statements | T-005 | ✅ |
| `textContent` mandate | T-003 | ✅ |
| Tauri capability system | T-006 | ✅ |
| CSP headers | T-003, T-006 | 🔶 Planned |
| Amnesic Mode | T-004 | 🔶 Planned |
| Manifest signature verification | T-002 | 🔶 Planned |
| SQLCipher encryption | T-007 | 🔲 Future |
| `cargo audit` CI | T-008 | 🔶 Planned |

---

## 8. Out of Scope (Current Phase)

The following threats are acknowledged but explicitly out of scope until Phase 5:

- **Browser fingerprinting** — requires Privacy Shield
- **Network traffic analysis** — requires Privacy Shield
- **Kernel exploits** — mitigated by Linux; not a Yfitops OS responsibility
- **Hardware attacks (firmware, TPM)** — out of scope permanently
- **Multi-user privilege separation** — Phase 5+
- **Side-channel attacks (timing, cache)** — future academic hardening
