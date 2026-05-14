# DECISIONS.md — Architecture Decision Records

Every significant architectural decision is recorded here. Format: context, options considered, decision made, consequences. Immutable once accepted — if a decision is reversed, a new ADR is created that supersedes it.

---

## ADR-001: Tauri v2 over Electron

**Date:** 2026-04  
**Status:** Accepted  
**Author:** Yfitops team

### Context
The project needed a framework to run HTML/CSS/JS modules as native desktop windows while also executing system-level Rust code. The two primary candidates were Electron and Tauri v2.

### Options Considered

| Factor | Electron | Tauri v2 |
|--------|----------|----------|
| Bundle size | ~120 MB (bundles Chromium) | ~3–15 MB (uses OS WebView) |
| Backend language | Node.js (JS) | Rust |
| Memory footprint | High | Low |
| Security model | Manual sandboxing | Built-in capability system |
| Local-first | Yes | Yes |
| Cross-platform | Yes | Yes |
| Existing codebase fit | Poor (we already moved away from Node.js) | Good |

### Decision
Tauri v2. The bundle size, memory footprint, and built-in security model are aligned with Yfitops OS's core values. The Rust backend is non-negotiable for a security-critical local system tool.

### Consequences
- All backend logic must be written in Rust
- Frontend uses Tauri IPC (`invoke` / `listen`) instead of HTTP
- Windows uses WebView2, macOS uses WKWebView — minor rendering differences to watch
- The Rust learning curve adds development time but eliminates a class of memory safety bugs

---

## ADR-002: Vanilla JS over React / Vue / Svelte

**Date:** 2026-04  
**Status:** Accepted  
**Author:** Yfitops team

### Context
The frontend needed a UI layer. Modern frameworks like React, Vue, and Svelte were considered.

### Options Considered

| Factor | React | Vue 3 | Svelte | Vanilla JS |
|--------|-------|-------|--------|------------|
| Build step required | Yes | Yes | Yes | No |
| Bundle size | ~45 KB | ~22 KB | ~5 KB | 0 KB |
| Startup time | Medium | Fast | Very fast | Instant |
| Dependency count | High | Medium | Medium | Zero |
| Long-term stability | Medium (API churn) | Medium | Medium | Maximum |
| Framework independence | No | No | No | Yes |
| Debugging simplicity | Medium | Medium | Low | High |

### Decision
Vanilla HTML, CSS, and JavaScript. No build step for the frontend means no webpack, no babel, no node_modules in the UI. The frontend is static files served by Tauri.

### Consequences
- No reactive state management out of the box — must implement manually where needed
- Component reuse is achieved via `fetch()` of shared HTML fragments and DOM manipulation
- Debugging is simpler — source maps not required, browser DevTools work directly
- Future: if a specific module genuinely needs a framework (e.g., a complex data visualisation), a new ADR must be created before adopting it

---

## ADR-003: SQLite over flat JSON for persistence

**Date:** 2026-05  
**Status:** Accepted  
**Author:** Yfitops team

### Context
Phase 0 used flat JSON files (`settings.json`, `clipboard.txt`) and a rolling text file (`audit.log`). As the number of modules grew, concurrent reads/writes caused race conditions, and querying the audit log became slow.

### Options Considered

| Factor | Flat JSON | JSON + fs-lock | SQLite |
|--------|-----------|----------------|--------|
| Concurrent access | Race conditions | Serialised (slow) | Built-in WAL |
| Query capability | None | None | Full SQL + FTS5 |
| Schema versioning | Manual | Manual | `PRAGMA user_version` |
| File portability | Yes | Yes | Single file |
| Backup simplicity | Yes | Yes | `VACUUM INTO` |
| Dependencies | Zero | Zero | sqlx (already a Tauri dep) |

### Decision
SQLite via `tauri-plugin-sql` (sqlx backend). Single `yfitops.db` file containing all persistent data. Schema versioned via `PRAGMA user_version`. Migrations in `src-tauri/migrations/`.

### Consequences
- All queries must use prepared statements (enforced in code review)
- The DB file location follows `tauri::path::BaseDirectory::AppData` on each platform
- Amnesic Mode works by switching the pool to `:memory:` — SQLite's in-memory mode
- Backup is `VACUUM INTO '/path/to/backup.db'` — trivial to implement

---

## ADR-004: tauri-specta for TypeScript type generation

**Date:** 2026-05  
**Status:** Accepted  
**Author:** Yfitops team

### Context
The frontend called Rust commands via `invoke()` with manually typed arguments. Any mismatch between the Rust function signature and the JS call site would only be caught at runtime. With 20+ commands and growing, this was unsustainable.

### Options Considered
1. **Manual types** — maintain a hand-written `types.ts` file mirroring Rust structs
2. **Runtime validation (zod)** — validate at call time with zod schemas
3. **tauri-specta** — auto-generate TypeScript bindings directly from Rust `#[specta::specta]` annotations

### Decision
`tauri-specta`. It is the only option that eliminates the type drift problem at the source. Type mismatches become compile errors, not runtime errors.

### Consequences
- Every Rust command must be annotated with `#[specta::specta]`
- `src/bindings.ts` is auto-generated — never edit it manually
- `src/yos-api.ts` imports from `bindings.ts` — this is the only public API surface for the frontend
- A new CI step runs `cargo test` to verify bindings generate cleanly

---

## ADR-005: Permission Guard as first-line runtime enforcement

**Date:** 2026-05  
**Status:** Accepted  
**Author:** Yfitops team

### Context
Tauri's capability system (the `capabilities/` JSON files) provides compile-time access control — it prevents window types from calling commands they aren't listed for. But it doesn't enforce module-level isolation within the same window type. All modules run in the same WebviewWindow type and could theoretically call any command.

### Options Considered
1. **Rely on Tauri capabilities only** — coarse-grained, not module-aware
2. **Runtime permission guard** — each command checks the calling module's manifest at runtime
3. **Separate window process per module** — each module in a fully isolated process

### Decision
Runtime Permission Guard (option 2) for now, with a path to option 3 (process isolation) in Phase 5. The guard checks `module_id` against the loaded `PermissionGuard` struct which maps module IDs to their declared `Permission` sets.

### Consequences
- Every Rust command must accept `module_id: String` as a parameter
- The `PermissionGuard` must be populated at startup from `ModuleManifest` definitions
- Permission denials are logged to `audit_log` with severity `WARN`
- Module manifests are the single source of truth for what each module can do

---

## ADR-006: Exec whitelist is hardcoded, never dynamic

**Date:** 2026-05  
**Status:** Accepted — immutable  
**Author:** Yfitops team

### Context
The Terminal module needs to execute shell commands. The original implementation (v0.3) used a string allowlist stored in `settings.json`. A bug was discovered where a settings update could expand the allowlist at runtime.

### Decision
The exec command whitelist is a compile-time constant in `src-tauri/src/commands/exec.rs`. It cannot be modified by any runtime configuration, settings change, or API call. Any change to the whitelist requires a code change, code review, and a new binary build.

```rust
const ALLOWED_COMMANDS: &[&str] = &[
    "ls", "cat", "pwd", "whoami", "hostname",
    "date", "uptime", "free", "df", "ps",
];
```

### Consequences
- Zero risk of runtime privilege escalation via settings manipulation
- Adding a new allowed command requires an explicit engineering decision (new ADR)
- Power users who want more commands must install a `.yops` terminal extension with appropriate permissions declared in its manifest (which requires user confirmation)

---

## ADR-007: One SQLite database file per device, not per profile

**Date:** 2026-05  
**Status:** Accepted  
**Author:** Yfitops team

### Context
The Profile System requires data isolation between profiles. Two options: one DB per profile, or one DB with a `profile_id` column on every table.

### Options Considered
1. **Separate DB per profile** — `default.db`, `work.db`, etc.
2. **Single DB with `profile_id` column** — all tables include profile scoping

### Decision
Single DB with `profile_id` column. This simplifies backup (one file to copy), migration (one migration script runs once), and Amnesic Mode (one in-memory pool to switch).

### Consequences
- All queries that return user data must include `WHERE profile_id = ?`
- A linting rule is added to flag queries that access profile-scoped tables without a `profile_id` filter
- Profile deletion requires `DELETE FROM <table> WHERE profile_id = ?` on all scoped tables — must be done in a transaction
