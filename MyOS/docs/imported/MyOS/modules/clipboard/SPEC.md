# Clipboard — Module Specification

**Module ID:** `com.yfitops.clipboard`  
**Status:** ✅ Stable  
**Version:** 8.0  
**Permissions:** `ClipboardRead`, `ClipboardWrite`  
**Trusted:** Yes

---

## 1. Purpose

The Clipboard module gives the user visibility and control over the system clipboard. It shows what is currently on the clipboard, allows editing it, and provides a staging area for text manipulation before pasting elsewhere.

It is intentionally simple — a single focused view with no history (to avoid persisting potentially sensitive clipboard data).

---

## 2. Layout

```
┌─ ● ● ● ──── Clipboard ─────────────────── [pull] [push] [clear] ─┐
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                                                              │ │
│  │  Hello, this is the current clipboard content.              │ │
│  │  It can span multiple lines.                                │ │
│  │                                                              │ │
│  │                                                              │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                        47 chars    │
└────────────────────────────────────────────────────────────────────┘
```

**Regions:**
- **Title bar** — traffic lights, "Clipboard", action buttons
- **Editor area** — editable textarea showing clipboard contents
- **Status bar** — character count, last sync time

---

## 3. Features

### 3.1 Pull
Reads the current system clipboard and loads it into the editor area. Button: `[pull]`. Keyboard: `Ctrl+R`.

### 3.2 Push
Writes the current editor area content to the system clipboard. Button: `[push]`. Keyboard: `Ctrl+Shift+V`.

### 3.3 Clear
Empties both the editor area and the system clipboard. Button: `[clear]`. Keyboard: `Ctrl+Shift+X`. Requires confirmation:
```
Clear the clipboard?
This will erase its current content.
[Cancel]  [Clear]
```

### 3.4 Auto-Pull on Open
When the module window opens, it immediately calls `pull` to show the current clipboard state. This is a single read — it does not poll continuously.

### 3.5 Character Count
Shows the character count of the editor content in the status bar. Updates on every keystroke.

---

## 4. Rust Commands

```rust
#[tauri::command]
pub async fn read_clipboard(module_id: String, state: ...) -> Result<String, String>

#[tauri::command]
pub async fn write_clipboard(module_id: String, content: String, state: ...) -> Result<(), String>
```

Both commands use the `arboard` crate. Both are logged to `audit_log` (content length only — not the actual content).

---

## 5. Security Note

The clipboard editor area is a `<textarea>` — it accepts user-typed content directly. When pulling from the clipboard, the content is set via `textarea.value = content` — this is safe (no HTML interpretation). It is never rendered via `innerHTML`.

---

## 6. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| No clipboard history | Would require FileWrite or DB — ADR needed | Phase 4 ADR |
| Text only — no images | arboard image support deferred | Phase 4 |
| No auto-sync on clipboard change | Would require background polling — privacy concern | Deliberate |
