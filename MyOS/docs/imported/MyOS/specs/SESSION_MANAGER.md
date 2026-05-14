# SESSION_MANAGER.md — Session Manager Specification

**Version:** 8.0  
**Status:** Specification — Phase 4  
**Last updated:** 2026-05

---

## 1. Overview

The Session Manager controls the persistence model for the current user session. It is the implementation of one of Yfitops OS's most distinctive capabilities: **Amnesic Mode** — a session that leaves no trace on disk.

The Session Manager is a Rust service that controls which SQLite pool the rest of the system writes to. It is the only component that knows whether the current session is persistent or ephemeral.

---

## 2. Session Modes

### Sovereign Mode (Default)

In Sovereign Mode, all data written during the session is persisted to `yfitops.db` on disk. The session survives application restart. Window positions, notes, settings, and audit logs are all retained.

This is the default mode on every launch.

### Amnesic Mode

In Amnesic Mode, the SQLite pool is switched to an in-memory database (`:memory:`). All writes go to RAM. On process exit:

1. All in-memory tables are explicitly dropped (zeroing the data)
2. The process exits
3. No data from the session exists on disk

Amnesic Mode is the privacy user's daily driver mode. It is inspired by Tails OS but implemented at the application layer rather than the OS layer.

**What Amnesic Mode does NOT protect against:**
- Physical memory attacks (cold boot attack) — RAM content is not encrypted
- Files written via `FileWrite` permission (see §6)
- Clipboard contents (system clipboard survives app exit — user must clear manually)
- Crash dumps from the OS

These limitations are documented in the `THREAT_MODEL.md` and displayed in the UI.

---

## 3. State Model

```rust
// src-tauri/src/state.rs

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
pub enum SessionMode {
    Sovereign,
    Amnesic,
}

pub struct SessionState {
    pub mode:       SessionMode,
    pub started_at: i64,         // Unix timestamp
    pub profile_id: String,      // Active profile
}
```

The session state is stored in `AppState` and is the only piece of application state that is never persisted to SQLite (because Amnesic Mode SQLite may be gone).

---

## 4. Mode Switch Protocol

Switching from Sovereign to Amnesic (or vice versa) is a significant operation. It follows this exact sequence:

### Sovereign → Amnesic

```
1. Display confirmation dialog in UI
   "Switch to Amnesic Mode?
    All current session data has been saved.
    New data from this point forward will NOT be saved."
   [Cancel] [Switch to Amnesic]

2. User confirms.

3. Rust: flush all pending writes to yfitops.db (drain queue)

4. Rust: close the on-disk SQLite pool

5. Rust: open a new in-memory SQLite pool (sqlite::memory:)

6. Rust: run migrations on in-memory pool (same schema)

7. Rust: update SessionState.mode = Amnesic

8. Rust: emit 'session:mode_changed' { from: 'Sovereign', to: 'Amnesic' }

9. UI: top bar turns amber, ghost icon appears, banner shown in all modules
```

### Amnesic → Sovereign

```
1. Display confirmation dialog in UI
   "Switch to Sovereign Mode?
    Data from this session will NOT be preserved — it was Amnesic.
    New data from this point forward WILL be saved."
   [Cancel] [Switch to Sovereign]

2. User confirms.

3. Rust: zero the in-memory pool (DROP all tables)

4. Rust: close in-memory pool

5. Rust: open on-disk pool (yfitops.db)

6. Rust: update SessionState.mode = Sovereign

7. Rust: emit 'session:mode_changed' { from: 'Amnesic', to: 'Sovereign' }

8. UI: top bar returns to normal, amber banner dismissed
```

**Important:** Switching mode mid-session does not migrate data between pools. Notes written in Amnesic Mode are lost when switching to Sovereign. This is by design. The confirmation dialog makes this clear.

---

## 5. UI Contract

### Top Bar — Mode Indicator

| State | Top bar colour | Icon | Tooltip |
|-------|--------------|------|---------|
| Sovereign | Dark (`var(--yos-bg)`) | None | — |
| Amnesic | Amber accent | 👻 Ghost | "Amnesic Mode — this session will not be saved" |
| Amnesic + Privacy Shield | Amber + green pulse | 👻 + 🛡 | Both active |

### Module Banners

Every module that writes data shows a dismissable banner when Amnesic Mode is active:

```html
<div class="yos-amnesic-banner" id="amnesic-banner" hidden>
  <span>👻 Amnesic Mode</span>
  <span class="yos-text-muted">Nothing you do here will be saved</span>
</div>
```

```javascript
await listen('session:mode_changed', (event) => {
    const banner = document.getElementById('amnesic-banner');
    banner.hidden = event.payload.to !== 'Amnesic';
});
```

---

## 6. File Write Blocking in Amnesic Mode

When Amnesic Mode is active, the `FileWrite` permission is additionally restricted. Before any file write operation, the `fileService` checks the session mode:

```rust
#[tauri::command]
pub async fn write_file(
    module_id: String,
    path: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.permission_guard.assert(&module_id, Permission::FileWrite)?;

    // Amnesic Mode: block file writes unless user explicitly overrides
    let session_mode = state.session_mode.lock().await;
    if *session_mode == SessionMode::Amnesic {
        return Err(
            "File write blocked in Amnesic Mode. \
             Switch to Sovereign Mode to write files.".into()
        );
    }

    // ... proceed with write
}
```

This means modules that rely on `FileWrite` (File Browser) will display an error in Amnesic Mode when the user tries to modify files. This is intentional and clearly communicated.

---

## 7. Startup Mode Detection

On every startup, the Session Manager reads the persisted setting `session.default_mode` from `yfitops.db`. If it is `"Amnesic"`, the mode switch happens before any module is opened.

```rust
// In lib.rs::setup(), after DB init:
let default_mode_str = sqlx::query_scalar!(
    "SELECT value FROM settings WHERE key = 'session.default_mode' AND profile_id = 'default'"
)
.fetch_optional(&db).await?;

let initial_mode = match default_mode_str.as_deref() {
    Some("Amnesic") => SessionMode::Amnesic,
    _ => SessionMode::Sovereign,
};
```

The setting `session.default_mode` can only be changed in Sovereign Mode. You cannot set Amnesic as the default from within an Amnesic session (the write would not persist).

---

## 8. Rust Command Interface

```rust
#[tauri::command]
#[specta::specta]
pub async fn get_session_state(
    state: tauri::State<'_, AppState>,
) -> Result<SessionState, String>

#[tauri::command]
#[specta::specta]
pub async fn set_session_mode(
    module_id: String,
    mode: SessionMode,
    state: tauri::State<'_, AppState>,
) -> Result<SessionState, String>
// Only callable by com.yfitops.settings (PrivacyControl permission required)
```

---

## 9. Session Duration Tracking

The session duration is tracked by the `SessionState.started_at` timestamp. On clean exit, the duration is written to the audit log:

```
app:exited | detail: "session_duration_secs=3842, mode=Sovereign, profile=default"
```

In Amnesic Mode, this is written to the in-memory log and then discarded. No session duration data is persisted for Amnesic sessions.
