# MODULES.md — Module System Specification

**Version:** 8.0  
**Status:** Authoritative — all modules must conform to this contract

---

## 1. What Is a Module

A module is a self-contained unit of functionality in Yfitops OS. It is:

- An HTML page (`index.html`) loaded in a native Tauri window
- Bound to a `ModuleManifest` that declares its identity and permissions
- Communicating exclusively through `yos-api.ts`
- Styled exclusively through tokens from `shared/theme.css`
- Isolated from other modules — no shared DOM, no shared JS scope

A module is not:
- A React component
- A server-side rendered page
- An iframe inside another module
- A module that reaches outside its declared permissions

---

## 2. Module Manifest (Rust)

Every module must have a corresponding `ModuleManifest` in the Rust backend. System modules are defined at compile time. Third-party modules are loaded from the `packages` database table.

```rust
// src-tauri/src/security/permission_guard.rs

pub struct ModuleManifest {
    pub id:          String,          // e.g. "com.yfitops.terminal"
    pub name:        String,          // e.g. "Terminal"
    pub version:     String,          // semver
    pub entry:       String,          // e.g. "modules/terminal/index.html"
    pub permissions: Vec<Permission>, // declared capabilities
    pub trusted:     bool,            // true for system modules only
    pub signature:   Option<String>,  // SHA-256 for .yops packages
}
```

---

## 3. Permission Enum

```rust
pub enum Permission {
    ExecCommand,      // run whitelisted exec commands
    ListDir,          // list directories within sandbox
    ReadFile,         // read files within sandbox
    GetSystemHealth,  // read CPU, RAM, disk, process list
    GetSetting,       // read a setting
    SetSetting,       // write a setting
    ListSettings,     // list all settings
    ListNotes,        // list user notes
    SaveNote,         // create or update a note
    DeleteNote,       // delete a note
    SearchNotes,      // full-text search notes
    OpenModule,       // spawn a new module window
    WriteAudit,       // write to the audit log
    QueryAudit,       // query the audit log
    QueryAI,          // query the AI assistant (Phase 5+)
}
```

---

## 4. System Module Manifests

### Terminal
```
ID:          terminal
Permissions: ExecCommand, GetSystemHealth, WriteAudit
Trusted:     true
```

### File Browser
```
ID:          files
Permissions: ListDir, ReadFile, WriteAudit
Trusted:     true
```

### Activity Monitor
```
ID:          activity
Permissions: GetSystemHealth, WriteAudit
Trusted:     true
```

### Notes
```
ID:          notes
Permissions: ListNotes, SaveNote, DeleteNote, SearchNotes, WriteAudit
Trusted:     true
```

### Settings
```
ID:          settings
Permissions: ListSettings, GetSetting, SetSetting, WriteAudit
Trusted:     true
```

### Audit Logs
```
ID:          logs
Permissions: QueryAudit, WriteAudit
Trusted:     true
```

### Desktop Shell
```
ID:          desktop
Permissions: GetSystemHealth, OpenModule, GetSetting, SetSetting, WriteAudit, QueryAudit
Trusted:     true
```

---

## 5. Module File Structure

Every module lives in `src/modules/<name>/` and contains:

```
src/modules/<name>/
├── index.html      # Entry point — must include shared titlebar and theme
├── app.js          # Module-specific JS (vanilla only)
├── style.css       # Module-specific styles (imports shared/theme.css)
└── SPEC.md         # Link to modules/<name>/SPEC.md in docs
```

---

## 6. HTML Contract

Every module `index.html` must:

1. Include `<link rel="stylesheet" href="../../shared/theme.css">` as the first stylesheet
2. Load `<script src="../../shared/titlebar.js"></script>` before `app.js`
3. Set `<meta name="module-id" content="com.yfitops.<name>">` in `<head>`
4. Have a root element `<div id="app">` that wraps all content
5. Call `initTitleBar('<Module Name>')` on DOM load

Template:
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="module-id" content="com.yfitops.MODULENAME">
  <title>Module Name — Yfitops OS</title>
  <link rel="stylesheet" href="../../shared/theme.css">
  <link rel="stylesheet" href="./style.css">
</head>
<body>
  <div class="yos-titlebar" id="titlebar"></div>
  <div id="app">
    <!-- module content -->
  </div>
  <script src="../../shared/titlebar.js"></script>
  <script src="../../yos-api.js"></script>
  <script src="./app.js"></script>
</body>
</html>
```

---

## 7. JavaScript Contract

Every module `app.js` must:

1. Read its module ID from the meta tag — never hardcode it
2. Pass `module_id` as the first argument to every `yos-api` call
3. Handle the loading state — show a skeleton, not a blank screen
4. Handle errors — show the error message in the UI, never `console.error` silently
5. Clean up event listeners in `window.addEventListener('unload', ...)`

```javascript
// Pattern: read module ID from meta tag
const MODULE_ID = document.querySelector('meta[name="module-id"]')
    ?.getAttribute('content') ?? 'unknown';

// Pattern: API call with module_id
async function loadData() {
    showSkeleton();
    try {
        const result = await window.yosApi.listNotes(MODULE_ID);
        renderNotes(result);
    } catch (err) {
        showError(err.message);
    } finally {
        hideSkeleton();
    }
}

// Pattern: event listener with cleanup
const unlisten = await window.__TAURI__.event.listen(
    'notes:created',
    (event) => renderNewNote(event.payload)
);
window.addEventListener('unload', () => unlisten());
```

---

## 8. Keyboard Shortcuts Contract

Every module must handle these global shortcuts:
- `Ctrl+W` or `Cmd+W` — close the module window
- `Escape` — close modal/overlay if open; else close the window

Module-specific shortcuts must not conflict with the system shortcuts defined in `CLAUDE.md §8`.

---

## 9. Loading and Error State Contract

Every module that fetches data must implement loading and error states.

**Loading skeleton** — shown immediately on mount, before any data:
```css
.skeleton {
    background: linear-gradient(
        90deg,
        var(--yos-surface) 25%,
        var(--yos-surface-2) 50%,
        var(--yos-surface) 75%
    );
    background-size: 200% 100%;
    animation: skeleton-pulse 1.5s infinite;
    border-radius: var(--yos-radius-sm);
    height: 1em;
}
@keyframes skeleton-pulse {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
}
```

**Error state** — shown on API failure:
```html
<div class="yos-error">
    <span class="yos-error-icon">⚠</span>
    <span class="yos-error-message" id="error-msg"></span>
    <button class="yos-btn" onclick="retryLoad()">Retry</button>
</div>
```

---

## 10. Module Window Configuration

Window properties are set in the Rust `open_module` command. Standard values:

| Property | Value | Notes |
|----------|-------|-------|
| `decorations` | `false` | Custom title bar used |
| `transparent` | `true` | Allows blur effects |
| `min_width` | 400 | Minimum for usability |
| `min_height` | 300 | Minimum for usability |
| `center` | `true` on first open | Subsequent opens restore last position |
| `shadow` | `true` | OS native shadow |

Default sizes per module:

| Module | Width | Height |
|--------|-------|--------|
| Terminal | 720 | 480 |
| File Browser | 880 | 560 |
| Activity Monitor | 680 | 520 |
| Notes | 640 | 540 |
| Clipboard | 480 | 400 |
| Settings | 720 | 560 |
| Log Viewer | 800 | 560 |
| Package Manager | 720 | 500 |

---

## 11. Third-Party Module Requirements

A `.yops` module must additionally:
1. Not declare `NetworkControl`, `PackageInstall`, or `PrivacyControl` permissions
2. Include a valid SHA-256 signature in its manifest
3. Not include any network calls in its HTML/JS (enforced by CSP headers)
4. Declare a `min_os_version` compatible with the current runtime
5. Pass the module validator (`validate_yops_manifest()` Rust function) before installation
