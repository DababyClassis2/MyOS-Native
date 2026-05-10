# Log Viewer — Module Specification

**Module ID:** `com.yfitops.logs`  
**Status:** 🔶 Phase 3  
**Version:** 8.0  
**Permissions:** `AuditRead`  
**Trusted:** Yes

---

## 1. Purpose

The Log Viewer is the user's window into everything Yfitops OS has done on their behalf. It renders the immutable audit log as a human-readable, filterable, chronological timeline. It answers the question: *"What has this system done?"*

It is a read-only module. It does not write to the log. It does not delete entries. Its only job is to make the audit log legible and navigable.

---

## 2. Layout

```
┌─ ● ● ● ─────── Log Viewer ──────────── [filter ▾] [export] [auto-scroll ●] ─┐
│                                                                               │
│  All  |  INFO  |  WARN  |  ERROR           🔍 Search logs...                 │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  14:32:01  [com.yfitops.terminal]  exec:command          ls -la /myos  INFO  │
│  14:31:58  [com.yfitops.files]     files:read            /myos/settings INFO  │
│  14:31:44  [com.yfitops.terminal]  exec:command          pwd            INFO  │
│  14:30:11  [com.yfitops.settings]  settings:changed      appearance...  INFO  │
│  14:29:03  [security]              security:permission_denied  ...      WARN  │
│  14:28:51  [com.yfitops.terminal]  exec:command          whoami         INFO  │
│  14:27:44  [app]                   app:started           v8.0.0-alpha   INFO  │
│                                                                               │
│  ─────────────────────────── May 9, 2026 ──────────────────────────────────  │
│                                                                               │
│  23:51:02  [com.yfitops.notes]     notes:deleted         "Draft ideas"  WARN  │
│                                                                               │
└───────────────────────────────────────────────────────────────────────────────┘
│  Showing 47 events  ·  Today: 38  ·  Oldest: May 8, 2026        [load more]  │
└───────────────────────────────────────────────────────────────────────────────┘
```

**Regions:**
- **Title bar** — traffic lights, "Log Viewer", filter dropdown, export button, auto-scroll toggle
- **Tab bar** — severity filter tabs (All / INFO / WARN / ERROR) + live search
- **Timeline** — scrollable chronological event list, newest at top
- **Day separators** — visual breaks between days
- **Footer** — summary counts + load more pagination

---

## 3. Features

### 3.1 Timeline View

Each log entry is a single row:

```
[timestamp]  [module badge]  [action]  [detail (truncated)]  [severity badge]
```

| Column | Width | Content |
|--------|-------|---------|
| Timestamp | 80px | `HH:MM:SS` — hover shows full ISO date |
| Module badge | 200px | Rounded pill with module short name. Colour per module |
| Action | 180px | `action_string` monospace |
| Detail | flex | Truncated at 60 chars, click to expand full detail |
| Severity | 60px | Coloured badge: INFO / WARN / ERROR |

**Severity colours:**

| Level | Text | Background | Row tint |
|-------|------|-----------|----------|
| `INFO` | `var(--yos-text-muted)` | none | none |
| `WARN` | `var(--yos-warn)` | `var(--yos-warn-dim)` | subtle amber row |
| `ERROR` | `var(--yos-danger)` | `var(--yos-danger-dim)` | subtle red row |

**Module badge colours (consistent across sessions):**

| Module | Badge colour |
|--------|-------------|
| `com.yfitops.terminal` | `#4a9e6b` |
| `com.yfitops.files` | `#6b7fa3` |
| `com.yfitops.monitor` | `#9b6b9e` |
| `com.yfitops.notes` | `#9e8a4a` |
| `com.yfitops.clipboard` | `#4a8a9e` |
| `com.yfitops.settings` | `#7a9e4a` |
| `com.yfitops.packages` | `#9e5a4a` |
| `app` | `var(--yos-accent-dim)` with accent text |
| `security` | `var(--yos-danger-dim)` with danger text |

### 3.2 Severity Filter Tabs

Clicking a tab filters the visible entries:
- **All** — shows all entries regardless of severity
- **INFO** — shows only INFO
- **WARN** — shows WARN and ERROR
- **ERROR** — shows only ERROR

Filter is applied client-side to the loaded page of entries. Switching tabs does not trigger a new DB query unless the tab's data has not yet been loaded.

### 3.3 Live Search

Search bar filters the visible entries by:
- `action` string (substring match)
- `detail` string (substring match)
- `module_id` (substring match)

Search is client-side against loaded entries. It does not call `query_audit` with a search term — it filters the JS array in memory. This keeps the search instant with no round-trip.

For deep historical search, a future ADR can add server-side FTS. Not in scope for Phase 3.

### 3.4 Auto-Scroll

When **auto-scroll** is ON (default):
- New events emitted via `audit:new_entry` are prepended to the top of the list
- The scroll position stays at the top (newest entries)
- The toggle button is highlighted with `var(--yos-accent)`

When auto-scroll is OFF:
- New events are still prepended but the view does not scroll
- A floating badge appears: `+N new entries ↑` — click to jump to top

### 3.5 Pagination

The initial load shows the 200 most recent entries. Scrolling to the bottom of the list triggers a `load_more` call that fetches the next 200.

```javascript
const observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && !isLoading && hasMore) {
        loadMoreEntries();
    }
}, { threshold: 0.1 });

observer.observe(document.getElementById('load-sentinel'));
```

### 3.6 Export

`[export]` in the title bar exports the **currently filtered view** (not the entire log) to a file. Format: JSON or CSV (user chooses in a dialog).

Export uses the `FileWrite` flow via the standard file dialog. In Amnesic Mode, export is blocked.

Wait — Log Viewer only has `AuditRead`, not `FileWrite`. Export must be handled by triggering the Settings module's export flow, or a new ADR must grant `FileWrite` to Log Viewer. For Phase 3, the export button copies the filtered entries as JSON to the **clipboard** (using `ClipboardWrite` — but that permission is not granted to Log Viewer either).

**Resolution for Phase 3:** The `[export]` button opens a modal showing the filtered entries as formatted JSON text. The user can select-all and copy manually. A `[copy all]` button does `navigator.clipboard.writeText()` — this uses the browser's native clipboard API, not the `ClipboardWrite` Tauri permission. This is acceptable for Phase 3.

**Phase 4:** Add `ClipboardWrite` permission to Log Viewer manifest after ADR review.

### 3.7 Day Separators

When entries span multiple calendar days, a visual separator is inserted between days:

```
─────────────────────── May 9, 2026 ─────────────────────────
```

This is computed client-side from the `ts` field of consecutive entries.

### 3.8 Entry Detail Expansion

Clicking on any row expands it to show the full `detail` string:

```
14:29:03  [security]  security:permission_denied  WARN
▼
  module_id: com.yfitops.terminal
  permission: ExecWrite
  message: Permission denied — module does not have ExecWrite
  profile_id: default
```

The expanded state is a CSS height transition from 0 to auto using a `max-height` approach.

---

## 4. Rust Commands

```rust
#[tauri::command]
#[specta::specta]
pub async fn query_audit(
    module_id: String,
    filter: AuditFilter,
    state: tauri::State<'_, AppState>,
) -> Result<AuditPage, String>

pub struct AuditFilter {
    pub severity:   Option<String>,    // "INFO" | "WARN" | "ERROR" | null (all)
    pub module_id:  Option<String>,    // filter by specific module
    pub profile_id: String,
    pub limit:      u32,               // default 200
    pub before_id:  Option<i64>,       // for pagination cursor
}

pub struct AuditPage {
    pub entries:  Vec<AuditEntry>,
    pub has_more: bool,
    pub total:    i64,
}

pub struct AuditEntry {
    pub id:         i64,
    pub ts:         i64,
    pub module_id:  String,
    pub action:     String,
    pub detail:     Option<String>,
    pub severity:   String,
    pub profile_id: String,
}
```

---

## 5. Events

| Event | Payload | Direction | Action |
|-------|---------|-----------|--------|
| `audit:new_entry` | `AuditEntry` | Rust → Module | Prepend to timeline if auto-scroll enabled |
| `session:mode_changed` | `{ to: String }` | Main → Module | Show Amnesic banner, disable export |

The `audit:new_entry` event is emitted by the `audit::write()` function in Rust after every successful audit log insert. This gives the Log Viewer live updates with zero polling.

```rust
// In audit.rs — after the insert succeeds
let _ = app_handle.emit("audit:new_entry", &entry);
```

---

## 6. Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+F` | Focus the search bar |
| `Ctrl+Shift+A` | Toggle auto-scroll |
| `Ctrl+E` | Open export modal |
| `1` | Switch to All tab |
| `2` | Switch to INFO tab |
| `3` | Switch to WARN tab |
| `4` | Switch to ERROR tab |
| `Escape` | Clear search filter |
| `Home` | Scroll to top (newest entries) |
| `End` | Scroll to bottom (oldest loaded entries) |

---

## 7. Amnesic Mode Behaviour

When session is Amnesic:
- A banner shows at the top: `👻 Amnesic Mode — this log will not be saved`
- The log entries shown are from the in-memory pool — they exist only for this session
- Export is disabled (the export modal copy-to-clipboard method still works)
- Auto-scroll is on by default (useful for watching a short amnesic session)
- On session exit, all log entries are discarded with the memory pool

---

## 8. Performance Considerations

The audit log can grow to 50,000 rows (the rotation threshold). Rendering all entries at once would freeze the UI. The following guardrails are mandatory:

- **Initial load:** 200 entries maximum via `LIMIT 200`
- **Pagination:** `WHERE id < :cursor` pattern for efficient keyset pagination (not OFFSET)
- **Client-side filter:** Applied to the already-loaded JS array — no re-query for search/tab changes
- **Virtual scroll (Phase 4):** If the loaded array exceeds 1,000 entries, replace the DOM list with a virtual scroll implementation to keep the DOM node count bounded

```sql
-- Efficient keyset pagination — no OFFSET
SELECT * FROM audit_log
WHERE profile_id = ?
  AND (? IS NULL OR severity = ?)
  AND (? IS NULL OR id < ?)
ORDER BY id DESC
LIMIT 200
```

---

## 9. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| No server-side text search | Client-side search is sufficient for 200-entry pages | Phase 4 FTS |
| Export requires manual copy for now | No `FileWrite` or `ClipboardWrite` permission | Phase 4 ADR |
| No log deletion from UI | Append-only design | Intentional — use Settings → Clear Audit Log |
| Virtual scroll not implemented | Deferred | Phase 4 |
| No entry timestamp timezone conversion | Stored as Unix epoch, displayed as local time | Future preference setting |
