# File Browser — Module Specification

**Module ID:** `com.yfitops.files`  
**Status:** ✅ Stable (read-only) → Phase 3 adds write ops  
**Version:** 8.0  
**Permissions:** `FileRead`, `FileWrite`, `FileWatch`, `ClipboardWrite`  
**Trusted:** Yes

---

## 1. Purpose

The File Browser provides a visual interface to the user's filesystem within sandboxed root paths. It is the primary file management tool in Yfitops OS. It is not a full system file manager — it operates within defined roots and enforces path boundaries.

---

## 2. Layout

```
┌─ ● ● ● ─────────── Files ─────────────────── [grid] [list] ─┐
├──────────┬──────────────────────────────────────────────────┤
│          │  /myos                                    [↑ up]  │
│  📁 myos │                                                   │
│  🏠 home │  📁 shell          📁 modules        📄 clip...   │
│  📦 tmp  │  📁 core           📄 settings.json              │
│          │                                                   │
│  ──────  │  ─────────────────────────────────────────────── │
│          │  settings.json                                    │
│  Pinned  │  ┌─────────────────────────────────────────┐     │
│          │  │ {                                        │     │
│          │  │   "schema_version": 2,                  │     │
│          │  │   "shell": { "port": 8080 }             │     │
│          │  │ }                                        │     │
│          │  └─────────────────────────────────────────┘     │
│          │  Size: 912 bytes  Modified: May 9 2026  [copy]   │
└──────────┴──────────────────────────────────────────────────┘
```

**Regions:**
- **Title bar** — traffic lights, "Files", view toggle (grid/list)
- **Sidebar** — allowed root shortcuts, pinned locations
- **Main content** — file/folder grid or list view
- **Preview pane** — shown when a file is selected (bottom-right split)
- **Status bar** — selected item info, copy path button

---

## 3. Allowed Root Paths

The File Browser never operates outside these paths. The Rust `list_dir` and `read_file` commands validate the path against this list before any operation:

```rust
const ALLOWED_ROOTS: &[&str] = &[
    "/myos",
    "/home",
    "/tmp",
];
```

Path traversal attempts (`../`, symlinks outside roots) are rejected with:
```
Access denied: path outside allowed roots
```

---

## 4. Views

### Grid View
- 4 columns (adjusts with window width)
- Each item: icon (folder 📁 or file 📄 by type) + name (truncated at 18 chars) + extension badge
- Folder items: click to navigate into
- File items: click to show preview pane

### List View
- Columns: Icon | Name | Size | Modified | Type
- Sortable by clicking column header
- Click to select, double-click folders to navigate

### Preview Pane
- Shown in bottom third of main content when a file is selected
- Shows file content for: `.txt`, `.md`, `.json`, `.log`, `.sh`, `.js`, `.css`, `.html`, `.toml`, `.yaml`
- Shows "Binary file — preview not available" for other types
- Content rendered as `textContent` inside a `<pre>` — no markdown rendering
- Max preview size: 64KB (shows first 64KB with a "file truncated" notice)

---

## 5. Features

### 5.1 Navigation
- Sidebar items navigate to their root path
- Breadcrumb trail at top of main content shows current path
- `[↑ up]` button navigates to parent (blocked at root)
- Browser-style history: back/forward via `Alt+Left` / `Alt+Right`
- Path bar: click to type a path directly

### 5.2 Context Menu (Phase 3)
Right-click any item to open a native context menu:

**Folders:**
- Open
- Copy Path
- Pin to Sidebar
- Rename *(Phase 3)*
- Delete *(Phase 3, with confirmation)*

**Files:**
- Open Preview
- Copy Path
- Copy to Clipboard (content)
- Rename *(Phase 3)*
- Delete *(Phase 3, with confirmation)*

### 5.3 File Operations (Phase 3)

| Operation | UI trigger | Rust command |
|-----------|-----------|-------------|
| Rename | F2 or context menu | `rename_file(old_path, new_name)` |
| Delete | Delete key or context menu | `delete_file(path)` — confirmation required |
| Create folder | Toolbar button | `create_dir(path, name)` |
| Create file | Toolbar button | `create_file(path, name)` |

Delete always shows a confirmation dialog:
```
Delete "settings.json"?
This cannot be undone.
[Cancel]  [Delete]
```

### 5.4 File Watching
The Rust `watch_dir` command uses the `notify` crate to watch the current directory for changes. When a change is detected, it emits a `files:changed` event. The module reloads the directory listing.

```rust
#[tauri::command]
pub async fn watch_dir(
    module_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String>
// Emits: files:changed { path, kind: "create"|"modify"|"delete"|"rename" }
```

### 5.5 Copy Path to Clipboard
Selecting any item and pressing `Ctrl+Shift+C` copies the full path to the system clipboard via the `ClipboardWrite` permission.

---

## 6. Rust Commands

| Command | Signature | Description |
|---------|-----------|-------------|
| `list_dir` | `(module_id, path) → Vec<FileEntry>` | List directory contents |
| `read_file` | `(module_id, path) → String` | Read file contents (max 64KB) |
| `rename_file` | `(module_id, path, new_name) → ()` | Rename file or folder |
| `delete_file` | `(module_id, path) → ()` | Delete file or folder |
| `create_dir` | `(module_id, path, name) → ()` | Create directory |
| `create_file` | `(module_id, path, name) → ()` | Create empty file |
| `watch_dir` | `(module_id, path) → ()` | Start watching directory |

```rust
pub struct FileEntry {
    pub name:        String,
    pub path:        String,
    pub is_dir:      bool,
    pub size_bytes:  Option<u64>,
    pub modified_ts: Option<i64>,
    pub extension:   Option<String>,
}
```

---

## 7. Events

| Event | Payload | Direction |
|-------|---------|-----------|
| `files:changed` | `{ path: String, kind: String }` | Rust → Module |

---

## 8. Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+G` | Toggle grid/list view |
| `Ctrl+Shift+C` | Copy path of selected item |
| `F2` | Rename selected item |
| `Delete` | Delete selected item (with confirmation) |
| `Alt+Left` | Navigate back |
| `Alt+Right` | Navigate forward |
| `Alt+Up` | Navigate to parent directory |
| `Escape` | Clear selection |

---

## 9. Amnesic Mode Behaviour

In Amnesic Mode, `FileWrite` is blocked by the Session Manager. The File Browser's write operations (rename, delete, create) will return an error. The UI shows a banner:

```
👻 Amnesic Mode — File writes are disabled
```

Read operations (`list_dir`, `read_file`) are not affected.

---

## 10. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| No drag-and-drop | Tauri WebView drag events complex | Phase 4 |
| No multi-select | Deferred | Phase 4 |
| No cut/copy/paste for files | Deferred | Phase 4 |
| Read-only in current build | Phase 3 work item | Phase 3 |
| No search within browser | Global search is the Search Overlay module | Phase 3 |
| Preview: text files only | Binary preview requires separate viewer | Phase 5 |
