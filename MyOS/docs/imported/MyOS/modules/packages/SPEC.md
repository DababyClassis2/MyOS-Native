# Package Manager — Module Specification

**Module ID:** `com.yfitops.packages`  
**Status:** 🔲 Phase 4  
**Version:** 8.0  
**Permissions:** `PackageInstall`, `FileRead`  
**Trusted:** Yes

---

## 1. Purpose

The Package Manager is the extension system for Yfitops OS. It lets users install, manage, enable, and remove `.yops` module packages. It is the controlled gateway through which the system gains new capabilities without abandoning the sovereignty model.

Every package install is:
- **Local** — no network call, no registry, no CDN
- **Auditable** — full permission list shown before install
- **Reversible** — uninstall removes all traces
- **Validated** — manifest checked, signature verified, permission escalation rejected

---

## 2. Layout

```
┌─ ● ● ● ─────── Package Manager ──────────────────── [install .yops] ─┐
├───────────────┬───────────────────────────────────────────────────────┤
│  Installed    │  Installed Packages                       3 packages  │
│  Disabled     │                                                       │
│               │  ┌─────────────────────────────────────────────────┐ │
│               │  │ 🟢 Focus Timer              v1.2.0              │ │
│               │  │ com.yfitops.community.focus                     │ │
│               │  │ Pomodoro timer with audit log session tracking  │ │
│               │  │ Permissions: SystemMonitor                      │ │
│               │  │ Installed: May 9, 2026        [disable] [remove]│ │
│               │  └─────────────────────────────────────────────────┘ │
│               │                                                       │
│               │  ┌─────────────────────────────────────────────────┐ │
│               │  │ 🟢 Markdown Viewer          v2.0.1              │ │
│               │  │ com.example.mdviewer                            │ │
│               │  │ Renders .md files with syntax highlighting      │ │
│               │  │ Permissions: FileRead                           │ │
│               │  │ Installed: May 8, 2026        [disable] [remove]│ │
│               │  └─────────────────────────────────────────────────┘ │
│               │                                                       │
│               │  ┌─────────────────────────────────────────────────┐ │
│               │  │ 🔴 Old Module               v0.9.0  [DISABLED]  │ │
│               │  │ com.example.old                                 │ │
│               │  │ ...                                             │ │
│               │  │                               [enable] [remove] │ │
│               │  └─────────────────────────────────────────────────┘ │
└───────────────┴───────────────────────────────────────────────────────┘
```

**Regions:**
- **Title bar** — traffic lights, "Package Manager", `[install .yops]` button
- **Sidebar** — view filters: Installed / Disabled
- **Package list** — cards, one per installed package
- **Install modal** — overlay shown when `[install .yops]` is clicked

---

## 3. Package Card

Each installed package renders as a card containing:

| Element | Content |
|---------|---------|
| Status indicator | 🟢 green (enabled) / 🔴 red (disabled) |
| Name + version | Large text + semver |
| ID | Small monospace, `com.author.name` |
| Description | One line, truncated |
| Permissions | Comma-separated list of granted permissions |
| Installed date | Human-readable |
| Actions | `[disable]` / `[enable]` + `[remove]` |

---

## 4. Install Flow

The install flow is the most security-critical UI in the system. It must make every step explicit.

### Step 1 — File Selection

User clicks `[install .yops]` in the title bar. A native file picker opens (Tauri dialog API), filtered to `.yops` files only.

```javascript
import { open } from '@tauri-apps/plugin-dialog';

const path = await open({
    title: 'Select a .yops package',
    filters: [{ name: 'Yfitops Package', extensions: ['yops'] }],
    multiple: false,
});
```

### Step 2 — Validation Modal

Before any install, the Rust `validate_package(path)` command is called. It returns a `ValidationResult`:

```rust
pub struct ValidationResult {
    pub manifest:     YopsManifest,
    pub is_valid:     bool,
    pub errors:       Vec<String>,    // empty if valid
    pub warnings:     Vec<String>,    // non-fatal issues
}
```

If `is_valid` is false, the install is blocked and the errors are shown:

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠ Package validation failed                               │
│                                                             │
│  This package cannot be installed:                         │
│                                                             │
│  • Signature mismatch — package may have been tampered     │
│  • Requires OS version 9.0.0 (current: 8.0.0)             │
│                                                             │
│                                              [Close]        │
└─────────────────────────────────────────────────────────────┘
```

### Step 3 — Permission Review

If the package is valid, a permission review modal is shown. The user must explicitly confirm before anything is written to disk:

```
┌─────────────────────────────────────────────────────────────────┐
│  Install "Focus Timer" v1.2.0?                                 │
│  by: yfitops-community                                         │
│                                                                 │
│  Pomodoro timer with audit log session tracking.               │
│                                                                 │
│  This package requests these permissions:                      │
│                                                                 │
│  ● SystemMonitor    read CPU and RAM usage                     │
│                                                                 │
│  ⚠ Once installed, this module can read system resources.     │
│                                                                 │
│  [Cancel]                        [Allow and Install]           │
└─────────────────────────────────────────────────────────────────┘
```

Layout rules for this modal:
- Never auto-dismiss. The user must click a button.
- `[Cancel]` is on the left (safe default)
- `[Allow and Install]` is on the right, styled as `yos-btn--primary`
- If the package has zero permissions, the message changes to: `"This package requests no special permissions."`
- If ANY warning exists (even for a valid package), show them in amber below the permissions list

### Step 4 — Installation

After confirmation, `install_package(path)` is called. A progress indicator replaces the modal content:

```
Installing Focus Timer...
  ✓ Manifest validated
  ✓ Signature verified
  ✓ Permissions checked
  ● Extracting files...
  ● Registering in database...
```

On success: modal closes, package card appears in the list with a subtle entrance animation, audit log receives `packages:installed` entry.

On failure: error message shown in the modal with a `[retry]` option.

---

## 5. Disable / Enable

Disabling a package:
1. Sets `enabled = 0` in the `packages` table
2. The Window Manager refuses to open this module (`open_module` returns an error)
3. The dock icon (if pinned) shows a 🔴 indicator
4. The package card moves to the "Disabled" sidebar filter view

Enabling reverses all of the above.

Disable does not uninstall. The files remain on disk.

---

## 6. Uninstall Flow

Clicking `[remove]` on a package card:

1. Confirmation dialog:
```
┌──────────────────────────────────────────────────────┐
│  Remove "Focus Timer"?                              │
│                                                      │
│  This will delete the module and all its data.     │
│  This cannot be undone.                             │
│                                                      │
│  [Cancel]                        [Remove]            │
└──────────────────────────────────────────────────────┘
```

2. If confirmed, `uninstall_package(id)` is called:
   - Deletes the package directory from `$APPDATA/yfitops/modules/<id>/`
   - Deletes the row from the `packages` table
   - Closes the module window if it is open
   - Removes from dock if pinned
   - Emits `packages:uninstalled` event
   - Writes to audit log with `WARN` severity

3. Package card disappears with a fade-out animation.

---

## 7. Rust Commands

```rust
#[tauri::command]
#[specta::specta]
pub async fn list_packages(
    module_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<InstalledPackage>, String>

#[tauri::command]
#[specta::specta]
pub async fn validate_package(
    module_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ValidationResult, String>

#[tauri::command]
#[specta::specta]
pub async fn install_package(
    module_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<InstalledPackage, String>

#[tauri::command]
#[specta::specta]
pub async fn uninstall_package(
    module_id: String,
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
#[specta::specta]
pub async fn toggle_package(
    module_id: String,
    package_id: String,
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<InstalledPackage, String>
```

---

## 8. Events

| Event | Payload | Direction |
|-------|---------|-----------|
| `packages:installed` | `InstalledPackage` | Rust → All modules |
| `packages:uninstalled` | `{ id: String }` | Rust → All modules |
| `packages:toggled` | `{ id: String, enabled: bool }` | Rust → All modules |

The desktop shell listens for `packages:installed` to add the new module's icon to the dock. It listens for `packages:uninstalled` to remove it.

---

## 9. Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+O` | Open file picker to install a package |
| `Escape` | Close any open modal |

---

## 10. Security Notes

The Package Manager is one of only two modules with an elevated permission (`PackageInstall`). Every install path is guarded:

1. **Validation before display** — the permission review modal only appears for valid packages
2. **No silent installs** — the permission review modal cannot be bypassed programmatically
3. **Signature verification** — blocks tampered packages before any files are extracted
4. **Permission escalation rejection** — `NetworkControl`, `PackageInstall`, `PrivacyControl` rejected at the Rust level before the modal is shown
5. **Atomic install** — if extraction succeeds but DB write fails, the extracted files are cleaned up (rollback)

---

## 11. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| No package update flow | Requires version comparison and file replacement logic | Phase 4+ |
| No online marketplace | Local-first — by design | Never (local registry file is the future) |
| No package signing authority | Cryptographic author verification not implemented | Phase 5 |
| Package data not backed up | User responsibility | Future: export installed packages list |
| Disabled in Amnesic Mode | `PackageInstall` blocked when no persistence available | By design |
