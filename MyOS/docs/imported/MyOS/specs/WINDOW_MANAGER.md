# WINDOW_MANAGER.md — Window Manager Specification

**Version:** 8.0  
**Status:** Specification — implementation in Phase 4  
**Last updated:** 2026-05

---

## 1. Overview

The Yfitops OS Window Manager (WM) is responsible for the creation, positioning, lifecycle, and visual presentation of all module windows. It runs as a Rust service in the Tauri Gateway layer and is commanded by both the user (via dock clicks and keyboard shortcuts) and the system (via the workspace state restore routine on startup).

It is not a compositing window manager (like Mutter or KWin). It delegates actual window rendering to Tauri's native WebView system and the OS compositor. Its job is management and policy — not rendering.

---

## 2. Architecture

```
User action (dock click / shortcut)
        │
        ▼
open_module(module_id) — Tauri command
        │
        ▼
WindowManager::open(module_id)
        │
    ┌───┴──────────────────────┐
    │                          │
Already open?              Not open
    │                          │
Focus existing             WebviewWindow::builder()
window                     .title(manifest.name)
                           .url(manifest.entry)
                           .inner_size(default_size)
                           .position(restore_or_center)
                           .decorations(false)
                           .transparent(true)
                           .build()
                               │
                           Register in workspace_state
                           Emit 'window:opened' event
```

---

## 3. Module Window Lifecycle

### States

```
           open_module()
               │
               ▼
         ┌──────────┐
         │  OPENING │  — WebviewWindow building
         └────┬─────┘
              │  window ready
              ▼
         ┌──────────┐
    ┌───►│   OPEN   │◄────────────────────────────────┐
    │    └────┬─────┘                                 │
    │         │                                       │
    │    ┌────┼─────────────────────────┐             │
    │    │    │                         │             │
    │  focus  minimize             close/Ctrl+W       │
    │    │    │                         │             │
    │    ▼    ▼                         ▼             │
    │  ┌───────────┐           ┌──────────────┐       │
    │  │  FOCUSED  │           │   CLOSING    │       │
    │  └─────┬─────┘           └──────┬───────┘       │
    │        │                        │               │
    │      restore                  destroy           │
    │        │                    workspace_state     │
    └────────┘                    update              │
                                        │             │
                                   ┌────┴─────┐       │
                                   │  CLOSED  │       │
                                   └────┬─────┘       │
                                        │             │
                                   open_module()──────┘
```

### Lifecycle Events (Tauri emit)

| Event | Payload | When emitted |
|-------|---------|-------------|
| `window:opened` | `{ module_id, window_label }` | After window is built and ready |
| `window:focused` | `{ module_id }` | When window receives focus |
| `window:minimized` | `{ module_id }` | When window is minimized |
| `window:closed` | `{ module_id }` | After window is destroyed |
| `window:moved` | `{ module_id, x, y }` | On move end (debounced 300ms) |
| `window:resized` | `{ module_id, w, h }` | On resize end (debounced 300ms) |

---

## 4. Window Configuration

### Default Properties

```rust
// src-tauri/src/commands/window_manager.rs

pub struct WindowConfig {
    pub decorations: bool,    // false — custom title bar always
    pub transparent: bool,    // true — enables backdrop blur
    pub shadow:      bool,    // true — OS native drop shadow
    pub resizable:   bool,    // true
    pub min_width:   f64,
    pub min_height:  f64,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            decorations: false,
            transparent: true,
            shadow: true,
            resizable: true,
            min_width: 400.0,
            min_height: 300.0,
        }
    }
}
```

### Per-Module Defaults

| Module | Width | Height | Min W | Min H |
|--------|-------|--------|-------|-------|
| Terminal | 720 | 480 | 480 | 300 |
| File Browser | 880 | 560 | 560 | 380 |
| Activity Monitor | 680 | 520 | 500 | 380 |
| Notes | 640 | 540 | 440 | 360 |
| Clipboard | 480 | 400 | 400 | 300 |
| Settings | 720 | 560 | 600 | 440 |
| Log Viewer | 800 | 560 | 600 | 380 |
| Package Manager | 720 | 500 | 560 | 380 |

---

## 5. Position and Restore

Window positions are persisted to the `workspace_state` SQLite table on every `window:moved` and `window:resized` event (debounced 300ms to avoid excessive writes).

```
Key format:  window.<module_id>.position
Value:       { "x": 120, "y": 80, "w": 720, "h": 480 }
```

On `open_module()`:
1. Query `workspace_state` for the module's last position
2. If found — open at that position
3. If not found — open centred on the primary display
4. After open, clamp to visible screen bounds (avoid off-screen windows after display change)

---

## 6. Snap Zones

Snap zones allow quick window tiling. Triggered by dragging a window to a screen edge or using keyboard shortcuts.

```
┌──────────────────────────────────────────┐
│              TOP HALF                   │
│ ┌──────────────┐  ┌──────────────────┐  │
│ │   LEFT HALF  │  │   RIGHT HALF     │  │
│ │              │  │                  │  │
│ └──────────────┘  └──────────────────┘  │
│              BOTTOM HALF                │
└──────────────────────────────────────────┘
```

| Snap Zone | Keyboard Shortcut | Result |
|-----------|------------------|--------|
| Left half | `Ctrl+Alt+Left` | Window fills left 50% of screen |
| Right half | `Ctrl+Alt+Right` | Window fills right 50% of screen |
| Top half | `Ctrl+Alt+Up` | Window fills top 50% of screen |
| Bottom half | `Ctrl+Alt+Down` | Window fills bottom 50% of screen |
| Maximise | `Ctrl+Alt+M` | Window fills entire screen (no title bar) |
| Restore | `Ctrl+Alt+R` | Returns to pre-snap size and position |
| Top-left quarter | `Ctrl+Alt+Numpad7` | Window fills top-left 25% |
| Top-right quarter | `Ctrl+Alt+Numpad9` | Window fills top-right 25% |
| Bottom-left quarter | `Ctrl+Alt+Numpad1` | Window fills bottom-left 25% |
| Bottom-right quarter | `Ctrl+Alt+Numpad3` | Window fills bottom-right 25% |

Snap is implemented in Rust by calling `window.set_position()` and `window.set_size()` via the Tauri window API. Snap state is saved to `workspace_state`.

---

## 7. Z-Order Management

Windows maintain a z-order stack. Clicking a window brings it to the top of the stack.

```rust
pub struct ZOrderStack {
    stack: Vec<String>,  // module_ids, bottom to top
}

impl ZOrderStack {
    pub fn bring_to_front(&mut self, module_id: &str) {
        self.stack.retain(|id| id != module_id);
        self.stack.push(module_id.to_string());
    }
}
```

The dock icon for the frontmost module receives a `active` CSS class for visual feedback.

---

## 8. Focus Mode Integration

When Focus Mode is active:

1. The focused module's window remains at full opacity
2. All other open module windows drop to 30% opacity (`window.set_effects()` or CSS overlay)
3. New window opens are allowed but open at 30% opacity
4. Keyboard shortcuts still work for all windows
5. The `workspace_state` key `focus_mode.active_module` records which module has focus

Focus Mode is toggled via `Ctrl+Shift+F` (global shortcut, registered in Rust).

---

## 9. Tauri Command Interface

```rust
// Commands exposed to frontend via IPC

#[tauri::command]
pub async fn open_module(module_id: String, state: ...) -> Result<String, String>
// Returns: window_label of opened/focused window

#[tauri::command]
pub async fn close_module(module_id: String, state: ...) -> Result<(), String>

#[tauri::command]
pub async fn minimize_module(module_id: String, state: ...) -> Result<(), String>

#[tauri::command]
pub async fn list_open_modules(state: ...) -> Result<Vec<OpenModule>, String>
// Returns: Vec of { module_id, window_label, position, size, is_focused }

#[tauri::command]
pub async fn snap_module(module_id: String, zone: SnapZone, state: ...) -> Result<(), String>

#[tauri::command]
pub async fn toggle_focus_mode(state: ...) -> Result<FocusModeState, String>
```

---

## 10. Dock Integration

The desktop shell dock (`src/index.html`) subscribes to window lifecycle events to show visual state:

| Module state | Dock icon appearance |
|-------------|---------------------|
| Closed | Normal icon, no indicator |
| Open (not focused) | Dot below icon (white) |
| Open (focused) | Dot below icon (accent green) |
| Minimized | Dot below icon (dim) |

```javascript
// Desktop shell event subscriptions
await listen('window:opened',   (e) => dock.setActive(e.payload.module_id));
await listen('window:closed',   (e) => dock.setInactive(e.payload.module_id));
await listen('window:focused',  (e) => dock.setFocused(e.payload.module_id));
await listen('window:minimized',(e) => dock.setMinimized(e.payload.module_id));
```
