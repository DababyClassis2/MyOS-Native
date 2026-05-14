# ARCHITECTURE.md — Yfitops OS System Architecture

**Version:** 8.0 (Native Vision)  
**Last updated:** 2026-05  
**Status:** Living document — updated with each architectural decision

---

## 1. Design Philosophy

Yfitops OS is built on five immutable principles. Every architectural decision is evaluated against them in the order listed:

1. **User sovereignty** — The user controls every process, every permission, every data flow
2. **Privacy by default** — Telemetry is off by default and cannot be re-enabled without user action
3. **Local-first** — All core features function without any network connection
4. **Modularity** — Every layer is independently replaceable without rebuilding the whole system
5. **Stability over novelty** — A working system beats an elegant one under construction

---

## 2. System Overview

Yfitops OS is a layered desktop operating system shell running as a native application via Tauri v2. It is not a web application. It is not a browser extension. It is a compiled Rust binary that embeds the OS's native WebView to render a rich user interface, while performing all system operations in memory-safe Rust.

```
┌─────────────────────────────────────────────────────────────┐
│  User Interface — HTML / CSS / Vanilla JS                   │
│  Rendered inside the OS native WebView (WebView2 / WKWebView│
├─────────────────────────────────────────────────────────────┤
│  Tauri IPC Bridge — invoke() / emit() / listen()           │
├─────────────────────────────────────────────────────────────┤
│  Tauri Command Gateway — Rust                               │
│  Permission Guard · Window Spawner · Event Bus              │
├─────────────────────────────────────────────────────────────┤
│  Core Services — Rust                                       │
│  exec · files · health · notes · settings · packages · logs │
├─────────────────────────────────────────────────────────────┤
│  Data Layer — SQLite (sqlx) + AppState (Arc<Mutex<T>>)      │
├─────────────────────────────────────────────────────────────┤
│  OS Kernel — Linux (Debian glibc) / future bare-metal        │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Layer Descriptions

### Layer 1 — Linux Kernel (Debian glibc)
The operating system foundation. Yfitops OS does not implement a kernel. It runs on top of Debian Linux (glibc) in the current development phase. The target is bare-metal deployment on a minimal Linux install.

Responsibilities:
- Process scheduling and isolation
- Memory management
- Filesystem access (ext4 / future custom)
- Network interface management
- Device drivers

Yfitops OS interacts with this layer exclusively through Rust's standard library and approved crates. No direct syscall assembly. No unsafe blocks except in documented exceptions.

### Layer 2 — Data and Privacy Layer

**SQLite Database (`yfitops.db`)**

Single-file database storing all persistent user data. Accessed exclusively through the Rust `AppState` singleton via `tauri-plugin-sql` (sqlx backend). The frontend never queries SQLite directly — it calls Rust commands that query it.

Schema domains:
- `settings` — key/value store, profile-scoped
- `notes` — user notes with FTS5 search index
- `audit_log` — immutable event record
- `packages` — installed .yops module manifests
- `workspace_state` — window positions, active modules
- `profiles` — sovereign identity profiles

**Session Manager**

Controls whether the database pool points to the on-disk `yfitops.db` or an in-memory SQLite database. In Amnesic Mode, no data written during the session survives process exit.

**Privacy Shield** *(Phase 4)*

A separate Rust child process that manages network isolation, MAC address randomisation, and header scrubbing. It communicates with the main process via a local Unix socket (Linux) or named pipe (Windows). Designed to fail closed — if the shield process dies, network access is suspended.

### Layer 3 — Core Services (Rust)

Each service is a Rust module in `src-tauri/src/commands/`. Services communicate with Layer 2 through the shared `AppState`. Services never communicate with each other directly — all cross-service data flows through the database or the event bus.

| Service | Crate dependencies | Responsibilities |
|---------|-------------------|-----------------|
| `exec` | stdlib `Command` | Whitelisted command execution |
| `files` | stdlib `fs`, `notify` | Directory listing, file read, file watch |
| `health` | `sysinfo` | CPU, RAM, disk, process list |
| `notes` | `sqlx`, `uuid` | CRUD + FTS5 full-text search |
| `settings` | `sqlx` | Key/value settings store |
| `packages` | `sqlx`, `serde_json` | .yops install, validate, list |
| `logs` | `sqlx`, `chrono` | Audit log write and query |
| `clipboard` | `arboard` | System clipboard read/write |

### Layer 4 — Tauri Command Gateway

The single interface between the frontend and all backend services. Every operation the frontend wants to perform must go through a registered Tauri command.

Responsibilities:
1. **Permission guard** — first check on every command, before any logic
2. **Command routing** — dispatches to the correct service
3. **Event bus** — emits structured events to frontend listeners
4. **Window manager** — spawns, tracks, and closes module windows
5. **Error normalisation** — all errors are serialised to a consistent JSON shape before leaving Rust

The gateway does not contain business logic. It is a routing and security layer only.

### Layer 5 — User Interface

Rendered by the OS native WebView inside Tauri's window shell. Built with Vanilla HTML, CSS, and JavaScript. No frontend framework is used. This is an explicit architectural constraint, not a preference.

The UI is composed of:
- **Desktop shell** (`src/index.html`) — dock, top bar, wallpaper, module launcher
- **Module windows** — individual HTML pages loaded in separate native Tauri windows
- **Shared components** — title bar, theme tokens, API wrapper loaded by every module

---

## 4. IPC Data Flow

All communication between the frontend and backend follows this strict one-way contract:

```
Frontend JS                     Rust Backend
    │                               │
    │  invoke('command_name',       │
    │    { module_id, ...args })    │
    │ ─────────────────────────────►│
    │                               │  1. permission_guard.assert()
    │                               │  2. validate inputs
    │                               │  3. execute service logic
    │                               │  4. write to audit_log
    │                               │  5. return Result<T, String>
    │  Result<T, String>            │
    │ ◄─────────────────────────────│
    │                               │
    │                               │  (async, any time)
    │  listen('event_name', cb)     │  app_handle.emit('event_name', payload)
    │ ◄─────────────────────────────│
```

**Rule:** The frontend never modifies state directly. It requests changes through commands. State changes are reflected back via events.

**Event naming convention:** `module:action` — e.g., `health:update`, `notes:created`, `packages:installed`, `session:mode_changed`

---

## 5. State Architecture

```rust
// src-tauri/src/state.rs

pub struct AppState {
    pub db:               Arc<Mutex<SqlitePool>>,
    pub permission_guard: Arc<PermissionGuard>,
    pub session_mode:     Arc<Mutex<SessionMode>>,
    pub workspace:        Arc<Mutex<WorkspaceState>>,
    pub active_profile:   Arc<Mutex<String>>,
}

pub enum SessionMode {
    Sovereign,   // yfitops.db on disk
    Amnesic,     // :memory: — wiped on exit
}

pub struct WorkspaceState {
    pub open_modules: HashMap<String, WindowId>,
    pub focus_mode:   bool,
    pub active_theme: ThemePalette,
}
```

State is initialised once in `lib.rs::run()` and managed by Tauri. Commands receive it via `tauri::State<'_, AppState>`.

---

## 6. Security Architecture

See `security/THREAT_MODEL.md` for full threat analysis and `security/PERMISSION_MANIFEST.md` for the capability table.

Summary of security layers:

1. **Tauri capability system** — the `capabilities/` directory defines which frontend windows can call which commands at the framework level
2. **Permission Guard** — runtime check at the top of every Rust command; enforces module manifests
3. **Exec whitelist** — hardcoded allow-list of permitted shell commands; never dynamically expanded at runtime
4. **SQL prepared statements** — zero string interpolation in any database query
5. **Input validation** — all command arguments validated in Rust before any logic executes
6. **Audit log** — every command invocation, permission denial, and system event recorded immutably

---

## 7. Module System Architecture

A module is a self-contained HTML page that:
1. Declares a `ModuleManifest` (Rust struct) listing its required permissions
2. Runs in an isolated Tauri window with its own rendering context
3. Communicates exclusively through `yos-api.ts` (which wraps Tauri `invoke()`)
4. Subscribes to events via `listen()` — never polls

Module window lifecycle:
```
User clicks dock icon
        │
        ▼
open_module(module_id) command
        │
  window_spawner checks if already open
        │
  ┌─────┴─────┐
  │           │
  Already   Not open
  open       │
  │        Spawn new WebviewWindow
  │        with module URL
  │        register in workspace_state
  │           │
  Focus    ───┘
  existing
  window
```

---

## 8. Database Schema

```sql
-- Settings: profile-scoped key/value
CREATE TABLE settings (
    key        TEXT    NOT NULL,
    value      TEXT    NOT NULL,
    profile_id TEXT    NOT NULL DEFAULT 'default',
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (key, profile_id)
);

-- Notes: searchable, profile-scoped
CREATE TABLE notes (
    id         TEXT    PRIMARY KEY NOT NULL,
    title      TEXT    NOT NULL,
    body       TEXT    NOT NULL DEFAULT '',
    is_pinned  INTEGER NOT NULL DEFAULT 0,
    profile_id TEXT    NOT NULL DEFAULT 'default',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE VIRTUAL TABLE notes_fts USING fts5(
    title, body, content=notes, content_rowid=rowid
);

-- Audit log: append-only
CREATE TABLE audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    ts         INTEGER NOT NULL DEFAULT (unixepoch()),
    module_id  TEXT    NOT NULL,
    action     TEXT    NOT NULL,
    detail     TEXT,
    severity   TEXT    NOT NULL DEFAULT 'INFO',
    profile_id TEXT    NOT NULL DEFAULT 'default'
);
CREATE INDEX idx_audit_ts        ON audit_log(ts DESC);
CREATE INDEX idx_audit_module    ON audit_log(module_id);
CREATE INDEX idx_audit_severity  ON audit_log(severity);

-- Packages
CREATE TABLE packages (
    id            TEXT    PRIMARY KEY NOT NULL,
    manifest_json TEXT    NOT NULL,
    enabled       INTEGER NOT NULL DEFAULT 1,
    installed_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Workspace state
CREATE TABLE workspace_state (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Profiles
CREATE TABLE profiles (
    id           TEXT    PRIMARY KEY NOT NULL,
    name         TEXT    NOT NULL,
    avatar_color TEXT    NOT NULL DEFAULT '#00ff88',
    is_active    INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL DEFAULT (unixepoch())
);
INSERT INTO profiles (id, name, is_active) VALUES ('default', 'Default', 1);
```

---

## 9. Deployment Architecture

**Development:**
```
Windows Host (D:\Vm-Shared\import.txt)
        │
        │  VirtualBox shared folder
        ▼
Debian Linux VM (/mnt/shared/import.txt)
        │
        │  cp → tr -d '\r' → sh
        ▼
/myos/shell/ (current v0.3 Node.js shell)

MyOS-Native/ (v8.0 Tauri app)
        │
        │  npm run tauri dev
        ▼
WebView2 window (localhost, no server)
```

**Production target:**
```
Tauri build → single native binary
        │
        │  installed to target machine
        ▼
Bare-metal Debian or custom Linux base
        │
        │  systemd service or init script
        ▼
Auto-start on login
```

**Build command:**
```bash
cd MyOS-Native
npm run tauri build
# Output: src-tauri/target/release/yfitops-os[.exe]
```

---

## 10. Future Architecture (Phase 5+)

These are documented as intent, not commitment. They require separate ADRs before implementation begins.

| Feature | Architecture approach | Phase |
|---------|----------------------|-------|
| Privacy Shield | Separate Rust child process, Unix socket IPC | 4 |
| Local AI Assistant | llama.cpp via Rust bindings, sandboxed subprocess | 5 |
| .yops marketplace | Local registry file, signed manifests, no cloud | 4 |
| Bare-metal boot | Custom init script + systemd unit | 6 |
| Multi-user support | Profile isolation + SQLite per-profile databases | 5 |
| Custom file system | FUSE-based overlay FS for sovereignty layer | 7 |