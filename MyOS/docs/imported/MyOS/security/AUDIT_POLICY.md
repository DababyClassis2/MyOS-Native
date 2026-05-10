# AUDIT_POLICY.md — Audit Logging Policy

**Version:** 8.0  
**Status:** Authoritative  
**Last updated:** 2026-05

---

## 1. Purpose

The audit log is the nervous system of Yfitops OS's security model. Every significant system event — command execution, permission decision, session change, data modification — is recorded here. The audit log makes the system accountable to the user. It is not a diagnostic tool. It is a sovereignty tool.

The user can see exactly what Yfitops OS has done on their behalf, and exactly when.

---

## 2. What Is Always Logged

The following events are mandatory. There are no exceptions, no opt-outs, no toggles.

### 2.1 System Events

| Event | Action string | Severity | Detail |
|-------|--------------|----------|--------|
| Application started | `app:started` | `INFO` | Yfitops OS version, session mode |
| Application exited | `app:exited` | `INFO` | Session duration in seconds |
| Session mode changed | `session:mode_changed` | `WARN` | Old mode → new mode |
| Profile switched | `profile:switched` | `INFO` | From profile_id → to profile_id |
| Profile created | `profile:created` | `INFO` | New profile name |
| Profile deleted | `profile:deleted` | `WARN` | Deleted profile_id, row counts per table |

### 2.2 Command Events

| Event | Action string | Severity | Detail |
|-------|--------------|----------|--------|
| Exec command run | `exec:command` | `INFO` | Command name + args (no output) |
| Exec command failed | `exec:error` | `ERROR` | Command name + error message |
| Permission denied | `security:permission_denied` | `WARN` | Module ID, requested permission |
| File read | `files:read` | `INFO` | File path |
| File written | `files:write` | `INFO` | File path, bytes written |
| File deleted | `files:delete` | `WARN` | File path |
| Directory listed | `files:list` | `INFO` | Directory path |
| Clipboard read | `clipboard:read` | `INFO` | Content length (bytes), not content |
| Clipboard written | `clipboard:write` | `INFO` | Content length (bytes), not content |

### 2.3 Data Events

| Event | Action string | Severity | Detail |
|-------|--------------|----------|--------|
| Note created | `notes:created` | `INFO` | Note ID, title |
| Note updated | `notes:updated` | `INFO` | Note ID, title |
| Note deleted | `notes:deleted` | `WARN` | Note ID, title |
| Setting changed | `settings:changed` | `INFO` | Key, old value → new value |
| Package installed | `packages:installed` | `INFO` | Package ID, version, permissions granted |
| Package uninstalled | `packages:uninstalled` | `WARN` | Package ID |
| Package disabled | `packages:disabled` | `INFO` | Package ID |

### 2.4 Security Events

| Event | Action string | Severity | Detail |
|-------|--------------|----------|--------|
| Permission denied | `security:permission_denied` | `WARN` | Module ID, permission requested |
| Invalid manifest | `security:invalid_manifest` | `ERROR` | Package ID, validation error |
| Signature mismatch | `security:signature_mismatch` | `ERROR` | Package ID |
| Elevated permission attempt by untrusted module | `security:escalation_attempt` | `ERROR` | Module ID, permission |
| Unknown module ID | `security:unknown_module` | `WARN` | Received module_id |

---

## 3. What Is Never Logged

The following are explicitly excluded from the audit log. Logging these would violate the privacy model.

| Data | Reason |
|------|--------|
| Command output (stdout/stderr) | Could contain passwords, keys, sensitive data |
| Note body content | Private user data |
| Clipboard contents | Could contain passwords or sensitive data |
| Setting values for sensitive keys | Passwords, tokens, keys |
| File contents | Private user data |
| Search queries | Privacy — user's search intent is private |

The audit log records *that* something happened, and *when*, and *which module* caused it. It does not record *what was in it*.

---

## 4. Log Schema

```sql
CREATE TABLE audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    ts         INTEGER NOT NULL DEFAULT (unixepoch()),
    module_id  TEXT    NOT NULL,
    action     TEXT    NOT NULL,
    detail     TEXT,              -- nullable — never contains sensitive content
    severity   TEXT    NOT NULL DEFAULT 'INFO',
    profile_id TEXT    NOT NULL DEFAULT 'default'
);
```

**Severity levels:**

| Level | Meaning |
|-------|---------|
| `INFO` | Normal operation |
| `WARN` | Unusual but not an error — security-relevant (deletions, mode changes, permission denials) |
| `ERROR` | Operation failed — includes security violations |

---

## 5. Rust Implementation

All audit writes go through a single function. No module calls `INSERT INTO audit_log` directly.

```rust
// src-tauri/src/core/audit.rs

use crate::state::AppState;

pub async fn write(
    state: &AppState,
    module_id: &str,
    action: &str,
    detail: Option<&str>,
    severity: &str,
) {
    let db = state.db.lock().await;
    let profile_id = state.active_profile.lock().await.clone();

    // Fire and forget — audit write failure must never block the caller
    let _ = sqlx::query!(
        "INSERT INTO audit_log (module_id, action, detail, severity, profile_id)
         VALUES (?, ?, ?, ?, ?)",
        module_id,
        action,
        detail,
        severity,
        profile_id
    )
    .execute(&*db)
    .await;
    // Intentionally ignoring the result — logging must not be a failure mode
}
```

Usage in every command:
```rust
audit::write(&state, &module_id, "files:read", Some(&path), "INFO").await;
```

---

## 6. Retention and Rotation Policy

The audit log is bounded. It does not grow without limit.

| Rule | Value | Rationale |
|------|-------|-----------|
| Maximum rows | 50,000 | ~30 days of active use |
| Archive trigger | > 40,000 rows | Rotates before hitting the limit |
| Archive format | Append to `audit_YYYY-MM.log` flat file | Human-readable, portable |
| Archive location | `$APPDATA/yfitops/logs/` | Alongside the database |
| Rows to keep after rotation | 5,000 most recent | Maintains recent context |
| Amnesic Mode | No rotation, no archive | In-memory — wiped on exit |

Rotation is triggered automatically by the `healthService` background task that runs every 10 minutes.

```rust
// Rotation query — runs inside a transaction
async fn rotate_audit_log(db: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM audit_log")
        .fetch_one(db).await?;

    if count > 40_000 {
        // Archive entries older than the newest 5,000
        let cutoff_id: i64 = sqlx::query_scalar!(
            "SELECT id FROM audit_log ORDER BY id DESC LIMIT 1 OFFSET 4999"
        ).fetch_one(db).await?;

        // Write archived rows to flat file, then delete
        // ... (file I/O to audit_YYYY-MM.log)

        sqlx::query!("DELETE FROM audit_log WHERE id <= ?", cutoff_id)
            .execute(db).await?;
    }
    Ok(())
}
```

---

## 7. User Access to Audit Log

The audit log is the user's data. They can:

1. **View it** — via the Log Viewer module (full timeline, filterable)
2. **Export it** — via Settings → Export → Audit Log (exports to `.csv` or `.json`)
3. **Delete it** — via Settings → Privacy → Clear Audit Log (with confirmation dialog)

**What the user cannot do:**
- Edit individual log entries (the log is append-only at the application level)
- Disable audit logging (it is not a setting — it is an architectural invariant)

---

## 8. Amnesic Mode Behaviour

In Amnesic Mode:
- Audit log writes go to the in-memory SQLite pool
- No archive file is written
- No rotation occurs (pointless for an ephemeral session)
- On session exit, the entire log is discarded with the memory pool
- The Log Viewer module shows a banner: `"Amnesic session — logs will not be saved"`

This is intentional. Amnesic Mode means the session leaves no record, including its audit log.
