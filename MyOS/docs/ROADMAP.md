# ROADMAP.md — Yfitops OS Development Roadmap

**Current version:** 8.0.0-alpha  
**Active phase:** Phase 3 — Data Core  
**Last updated:** 2026-05

---

## Progress Legend

```
[x] Complete and stable
[>] In progress
[ ] Planned — not started
[~] Deferred — decision pending
[!] Blocked — dependency unresolved
```

---

## Phase 1 — Foundation ✅ Complete

**Goal:** Prove the Tauri v2 architecture works end-to-end on the target platform.

- [] Scaffold `MyOS-Native` Tauri v2 project
- [] Establish `src/` (frontend) and `src-tauri/` (backend) separation
- [] Implement Rust `get_system_health` command (CPU, RAM, disk)
- [] Create desktop shell — dock icons, top bar, wallpaper
- [] Implement `open_module` native window spawner
- [] Connect `AppState` pattern with `Arc<Mutex<T>>`
- [] Write `ARCHITECTURE.md` v1

**Deliverable:** A working Tauri desktop that opens native windows.  
**Status:** Stable. Do not modify Phase 1 infrastructure without an ADR.

---

## Phase 2 — Command Center ✅ Complete

**Goal:** Build the three most critical modules so the system is actually usable.

- [] **Terminal module**
  - [] macOS-style title bar with traffic light buttons
  - [] Rust `exec_command` bridge with command whitelist
  - [] Command history (up/down arrow)
  - [] `clear` command support
  - [] Output line limit (prevent memory overflow from verbose commands)
- [] **Activity Monitor module**
  - [] Real-time CPU and RAM charts (2-second poll)
  - [] Native process list via `sysinfo` crate
  - [] Process name + PID + RSS memory
- [] **File Browser module (read-only)**
  - [] Rust `list_dir` command
  - [] Grid view with file/folder icons
  - [] Sandboxed root paths (no traversal above allowed roots)

**Deliverable:** Three working modules in native Tauri windows.  
**Status:** Stable. File Browser needs write operations in Phase 3.

---

## Phase 3 — Data Core 🔶 In Progress

**Goal:** Add persistent storage, type safety, and the security foundation.  
**Target completion:** 4 weeks from phase start.

### 3.1 — Database Layer (Week 1)
- [ ] Add `tauri-plugin-sql` with SQLite feature to `Cargo.toml`
- [ ] Write `migrations/V1__initial.sql` with full schema
- [ ] Implement schema version tracking via `PRAGMA user_version`
- [ ] Integrate migration runner in `lib.rs::setup()`
- [ ] Add `tauri-specta` for TypeScript binding generation
- [ ] Generate `src/bindings.ts` from all Rust command signatures
- [ ] Refactor `src/yos-api.ts` to import from `bindings.ts`

### 3.2 — Settings Service (Week 1)
- [ ] Implement `get_setting(key, profile_id)` Rust command
- [ ] Implement `set_setting(key, value, profile_id)` Rust command
- [ ] Implement `list_settings(profile_id)` Rust command
- [ ] Delete Settings module's current `localStorage` usage
- [ ] Wire Settings module UI to new Rust commands

### 3.3 — Notes Service (Week 2)
- [ ] Implement `list_notes(profile_id)` Rust command
- [ ] Implement `save_note(note)` Rust command (create + update)
- [ ] Implement `delete_note(id)` Rust command
- [ ] Implement `search_notes(query)` using SQLite FTS5
- [ ] Build Notes module UI (sidebar list + editor)
- [ ] Add note pinning support

### 3.4 — Permission Guard (Week 2)
- [ ] Define `Permission` enum (all 12 capability types)
- [ ] Define `ModuleManifest` struct
- [ ] Implement `PermissionGuard` with `check()` and `assert()`
- [ ] Write manifests for all 8 existing modules
- [ ] Add `guard.assert()` as first line of every Rust command
- [ ] Log permission denials to `audit_log`
- [ ] Write `security/PERMISSION_MANIFEST.md`

### 3.5 — Search and Context Menu (Week 3)
- [ ] Add `FTS5` virtual table for file path indexing
- [ ] Implement `search_all(query)` command (notes + files)
- [ ] Build search overlay module triggered by `Ctrl+Space`
- [ ] Add right-click context menu to File Browser
- [ ] Implement file operations: rename, delete, copy path
- [ ] Context menu uses Tauri `menu` API (native OS menu)

### 3.6 — Audit Log Service (Week 3)
- [ ] Implement `write_audit(module_id, action, detail, severity)` command
- [ ] Call `write_audit` from every command (success and failure)
- [ ] Implement `query_audit(filter)` command with pagination
- [ ] Build Log Viewer module UI (timeline view, filter dropdown)
- [ ] Add audit log rotation (max 10,000 rows, archive to `.log` file)

### 3.7 — Workspace Persistence (Week 4)
- [ ] Implement `save_workspace_state(key, value)` command
- [ ] Implement `load_workspace_state(key)` command
- [ ] Persist window positions to `workspace_state` table on window move
- [ ] Restore window positions on next launch
- [ ] Persist dock icon order to `workspace_state`
- [ ] Persist last active module to `workspace_state`

### 3.8 — Profile System (Week 4)
- [ ] Implement `create_profile(name, color)` command
- [ ] Implement `switch_profile(profile_id)` command
- [ ] Implement `list_profiles()` command
- [ ] Add profile switcher to top bar UI
- [ ] Ensure all queries filter by `profile_id`
- [ ] Test profile isolation (settings from profile A invisible to profile B)

---

## Phase 4 — Ecosystem 🔲 Planned

**Goal:** Make the system extensible and secure. First complete Phase 3.  
**Estimated start:** Week 5

### 4.1 — Amnesic Mode
- [ ] Implement `SessionMode` enum in `AppState`
- [ ] Implement `set_session_mode(mode)` Rust command
- [ ] Switch SQLite pool to `:memory:` when Amnesic mode is activated
- [ ] Zero in-memory pool on session end (explicit drop)
- [ ] Visual indicator in top bar (amber glow + ghost icon)
- [ ] Write `specs/SESSION_MANAGER.md`

### 4.2 — Package Installer
- [ ] Define `.yops` manifest schema v1 (see `specs/YOPS_FORMAT.md`)
- [ ] Implement `install_package(manifest_json)` command
  - [ ] Parse and validate manifest
  - [ ] Reject permission escalation (untrusted packages cannot request elevated permissions)
  - [ ] Verify manifest signature (SHA-256)
  - [ ] Write to `packages` table
  - [ ] Emit `packages:installed` event
- [ ] Implement `uninstall_package(id)` command
- [ ] Implement `list_packages()` command
- [ ] Implement `toggle_package(id, enabled)` command
- [ ] Build Package Manager module UI
- [ ] Write `specs/YOPS_FORMAT.md`

### 4.3 — Wallpaper-Aware Theming
- [ ] Add `image` crate to extract dominant colour palette from wallpaper
- [ ] Implement `set_wallpaper(path)` command with palette extraction
- [ ] Emit `theme:updated` event with `ThemePalette` payload
- [ ] All modules listen for `theme:updated` and apply CSS variables live
- [ ] Write `specs/THEMING.md`

### 4.4 — Focus Mode
- [ ] Implement `toggle_focus_mode()` command
- [ ] Focus mode state stored in `workspace_state`
- [ ] When active: dim all non-essential module windows (CSS opacity 0.3)
- [ ] Show countdown timer in top bar (Pomodoro — 25 min default, configurable)
- [ ] Suppress all Tauri events to non-focus modules during focus mode
- [ ] Record focus sessions to `audit_log` (start, end, duration)

### 4.5 — Window Manager
- [ ] Implement snap zones (left half, right half, maximise, quarters)
- [ ] Implement window z-ordering (bring to front / send to back)
- [ ] Implement window grouping (tab modules into single window)
- [ ] Write `specs/WINDOW_MANAGER.md`

---

## Phase 5 — Privacy Shield 🔲 Planned

**Goal:** Implement the privacy layer as a separate, isolated process.  
**Dependency:** Phase 4 must be stable and passing all security checks.

- [ ] Design Privacy Shield as a separate Rust binary
- [ ] Implement IPC between main process and shield process (Unix socket)
- [ ] MAC address randomisation on session start
- [ ] Network Guardian: per-process network call monitoring
- [ ] Implement Privacy Heatmap module (visualises data exposure)
- [ ] Timezone masking
- [ ] User-Agent header rotation
- [ ] Write `specs/PRIVACY_SHIELD.md`

---

## Phase 6 — AI Layer 🔲 Disabled

**Status:** This phase is explicitly disabled until Phase 5 is complete and audited.

Requirements before AI layer can be activated:
- Privacy Shield is operational and verified
- Audit log covers 100% of command invocations
- Local AI model runs entirely offline (no API calls)
- User explicitly opts in through a permanent settings toggle

Planned capabilities (not a commitment):
- Local assistant using llama.cpp Rust bindings
- Log analysis and anomaly detection
- System diagnostics and recommendations
- Context-aware command suggestions

---

## Phase 7 — Bare-Metal Target 🔲 Future

**Goal:** Boot Yfitops OS on real hardware without a host OS.

- [ ] Minimal Alpine Linux base (no GUI, no Xorg, no GNOME)
- [ ] Custom init script (replaces systemd for boot)
- [ ] Tauri app as the login shell replacement
- [ ] Single-user mode by default
- [ ] Encrypted root filesystem (LUKS)
- [ ] USB boot image builder

---

## Non-Goals (Permanent)

These will never be implemented regardless of user requests or external pressure:

| Non-goal | Reason |
|----------|--------|
| Cloud sync or backup | Violates local-first principle |
| Telemetry or analytics | Violates privacy-first principle |
| Third-party ad networks | Violates sovereignty principle |
| Social features or sharing | Out of scope for a sovereign OS |
| Custom kernel | Out of scope — use Linux |
| Browser engine | Out of scope — use OS WebView |
| Wayland/X11 compositor | Out of scope — use Tauri's window manager |
| Enterprise SSO / LDAP | Out of scope for current phase |

---

## Version Numbering

```
MAJOR.MINOR.PATCH[-LABEL]

MAJOR: Architectural layer change (e.g., 7→8 was Node.js→Tauri)
MINOR: New module or major feature added
PATCH: Bug fixes, security patches, dependency updates
LABEL: alpha | beta | rc

Examples:
8.0.0-alpha   Current development version
8.1.0-alpha   When Phase 3 completes
8.2.0-beta    When Phase 4 completes
8.3.0-rc      When Phase 5 is audited
8.3.0         First production release
```