# Notes — Module Specification

**Module ID:** `com.yfitops.notes`  
**Status:** 🔶 Phase 3  
**Version:** 8.0  
**Permissions:** `FileRead`, `ClipboardWrite`  
**Trusted:** Yes

---

## 1. Purpose

Notes is the user's private, local, searchable text workspace. All notes are stored in SQLite — never in files, never in the cloud. It is the first module that demonstrates the full Phase 3 data stack: SQLite persistence, FTS5 search, and profile scoping.

---

## 2. Layout

```
┌─ ● ● ● ──────────── Notes ─────────────────── [new] [export] ─┐
├───────────────┬────────────────────────────────────────────────┤
│  🔍 Search... │                                                │
│               │  Planning Session                              │
│  📌 Pinned    │  ─────────────────────────────────────────    │
│  Planning...  │                                                │
│               │  - Review module permission manifests          │
│  All Notes    │  - Finish Phase 3 data layer                   │
│  Planning...  │  - Write AUDIT_POLICY.md                       │
│  Arch notes   │  - Start THEMING.md                           │
│  Ideas        │  - SQLite FTS5 for search                     │
│  Quick note   │                                                │
│               │  ─────────────────────────────────────────    │
│               │  Created: May 9 2026  Modified: May 10 2026   │
│  ────         │                                    [copy] [🗑] │
│  [new note]   │                                                │
└───────────────┴────────────────────────────────────────────────┘
```

**Regions:**
- **Sidebar** — search bar, pinned notes, all notes list
- **Editor pane** — full-width text editor for selected note
- **Status bar** — created/modified timestamps, copy and delete buttons

---

## 3. Features

### 3.1 Note CRUD
- **Create** — click `[new note]` in sidebar or press `Ctrl+N`
- **Read** — click any note in the sidebar list
- **Update** — type in the editor pane; auto-saves after 1.5 seconds of inactivity (debounced)
- **Delete** — click 🗑 in status bar, requires confirmation dialog

### 3.2 Auto-Save
```javascript
let saveTimer = null;

editorEl.addEventListener('input', () => {
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
        saveNote(currentNoteId, editorEl.value);
    }, 1500);
});
```

A small "Saving..." indicator appears in the status bar during the debounce window. It changes to "Saved ✓" on successful write and "Save failed ⚠" on error.

### 3.3 Pinning
- Click the 📌 icon on any note card in the sidebar to pin it
- Pinned notes appear in the "Pinned" section at the top of the sidebar
- Pins are stored in the `is_pinned` column in the `notes` table

### 3.4 Full-Text Search
- Search bar at the top of the sidebar filters notes in real-time
- Searches both `title` and `body` via SQLite FTS5
- Results ranked by relevance (FTS5 BM25 ranking)
- Search terms highlighted in the matching note list entries
- Empty query: show all notes sorted by `updated_at DESC`

```rust
// FTS5 search query
sqlx::query_as!(NoteSearchResult,
    "SELECT n.id, n.title, n.updated_at,
            snippet(notes_fts, 1, '<mark>', '</mark>', '...', 20) AS excerpt
     FROM notes_fts
     JOIN notes n ON notes_fts.rowid = n.rowid
     WHERE notes_fts MATCH ? AND n.profile_id = ?
     ORDER BY rank",
    fts_query, profile_id
)
```

### 3.5 Export
- `[export]` button in title bar opens a file save dialog
- Options: export as `.md` (Markdown), `.txt` (plain text), `.json` (structured)
- JSON export format:
```json
{
  "exported_at": "2026-05-10T14:00:00Z",
  "notes": [
    {
      "id": "uuid",
      "title": "Planning Session",
      "body": "- Review module...",
      "created_at": 1746000000,
      "updated_at": 1746086400,
      "is_pinned": false
    }
  ]
}
```

---

## 4. Rust Commands

| Command | Signature | Notes |
|---------|-----------|-------|
| `list_notes` | `(module_id, profile_id) → Vec<NoteSummary>` | Returns title + dates only |
| `get_note` | `(module_id, id, profile_id) → Note` | Returns full body |
| `save_note` | `(module_id, note) → Note` | Create or update (upsert by id) |
| `delete_note` | `(module_id, id, profile_id) → ()` | Logged as WARN in audit |
| `search_notes` | `(module_id, query, profile_id) → Vec<NoteSearchResult>` | FTS5 ranked search |
| `pin_note` | `(module_id, id, pinned, profile_id) → ()` | Toggle pin |

```rust
pub struct Note {
    pub id:         String,     // UUID v4
    pub title:      String,
    pub body:       String,
    pub is_pinned:  bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct NoteSummary {
    pub id:         String,
    pub title:      String,
    pub is_pinned:  bool,
    pub updated_at: i64,
}

pub struct NoteSearchResult {
    pub id:         String,
    pub title:      String,
    pub excerpt:    String,     // FTS5 snippet with <mark> highlights
    pub updated_at: i64,
}
```

---

## 5. Events

| Event | Payload | Direction |
|-------|---------|-----------|
| `notes:created` | `NoteSummary` | Rust → Module |
| `notes:updated` | `NoteSummary` | Rust → Module |
| `notes:deleted` | `{ id: String }` | Rust → Module |
| `session:mode_changed` | `{ to: String }` | Main → Module (show amnesic banner) |

---

## 6. Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+N` | New note |
| `Ctrl+S` | Force save immediately (bypass debounce) |
| `Ctrl+F` | Focus search bar |
| `Ctrl+Shift+C` | Copy current note body to clipboard |
| `Delete` | Delete selected note (when sidebar item focused) |
| `Escape` | Clear search and show all notes |

---

## 7. Amnesic Mode Behaviour

In Amnesic Mode, notes are written to the in-memory SQLite pool. The sidebar shows:
```
👻 Amnesic — notes will not be saved
```
The export function is blocked in Amnesic Mode (it requires `FileWrite`).

---

## 8. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| Plain text only — no rich text | Markdown rendering needs DOMPurify (ADR required) | Phase 4 |
| No note folders or tags | Deferred | Phase 4 |
| No note sharing | Out of scope — local-first | Never |
| Export requires FileRead — blocked in Amnesic | By design | No |
| No import from clipboard | ClipboardRead permission not granted | ADR required |
