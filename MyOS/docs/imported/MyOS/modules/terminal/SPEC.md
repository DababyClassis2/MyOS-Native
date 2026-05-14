# Terminal — Module Specification

**Module ID:** `com.yfitops.terminal`  
**Status:** ✅ Stable  
**Version:** 8.0  
**Permissions:** `ExecWrite`, `ExecRead`, `AuditRead`  
**Trusted:** Yes

---

## 1. Purpose

The Terminal provides the user with a controlled command execution environment. It is not a full shell — it is a curated interface to a hardcoded whitelist of safe, read-oriented system commands. The user sees a terminal UX; the system enforces strict limits underneath.

It is the primary power-user interface of Yfitops OS and the gateway for inspecting system state.

---

## 2. Layout

```
┌─ ● ● ● ──────────── Terminal ──────────────────────── [+] ─┐
│  [Shell 1] [Shell 2] [+]                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  yos $ ls -la /myos                                         │
│  total 48                                                   │
│  drwxr-xr-x  6 root root 4096 May  9 2026 .                │
│  drwxr-xr-x 18 root root 4096 May  9 2026 ..               │
│  -rw-r--r--  1 root root  912 May  9 2026 clipboard.txt    │
│  drwxr-xr-x  2 root root 4096 May  9 2026 shell           │
│                                                             │
│  yos $ █                                                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  yos $ ▌                                          [clear]   │
└─────────────────────────────────────────────────────────────┘
```

**Regions:**
- **Title bar** — macOS traffic lights, "Terminal" label, tab open button
- **Tab bar** — named shell tabs (up to 5 simultaneous)
- **Output area** — scrollable, monospace, line-numbered on hover
- **Input bar** — fixed bottom, always visible, single-line input

---

## 3. Features

### 3.1 Command Execution
- User types a command and presses Enter
- Input is split on whitespace into `cmd` + `args[]`
- Sent to `exec_command(module_id, cmd, args)` Rust command
- Output (stdout + stderr) rendered line by line in the output area
- Cursor returns to input bar on completion

### 3.2 Command Whitelist (displayed on `help`)
```
ls       — list directory contents
cat      — display file contents
pwd      — print working directory
whoami   — current user
hostname — machine hostname
date     — current date and time
uptime   — system uptime
free     — memory usage
df       — disk usage
ps       — running processes
clear    — clear terminal output (local — no exec)
help     — show this message
history  — show command history (local — no exec)
```

Any command not on this list returns:
```
yos: command not permitted: <cmd>
Type 'help' for available commands.
```

### 3.3 Multi-Tab Support
- Up to 5 simultaneous shell tabs
- Each tab has its own output buffer (stored in module JS memory, not persisted)
- Tabs are named "Shell 1", "Shell 2", etc. (future: rename on double-click)
- Closing a tab clears its output buffer
- The `+` button in the tab bar creates a new tab if under the limit

### 3.4 Command History
- Last 100 commands stored in module memory (not persisted to disk)
- Up arrow: navigate backward through history
- Down arrow: navigate forward
- `history` command: print all stored history with index numbers
- `history clear`: clear history (local operation, no exec)
- History is per-tab, not shared

### 3.5 Output Rendering
- Line limit: 5000 lines per tab (oldest lines trimmed from the top)
- stdout: `var(--yos-text)` colour
- stderr: `var(--yos-danger)` colour
- System messages (permission denied, not permitted): `var(--yos-warn)` colour
- Long lines wrap at the output area boundary
- No HTML rendering in output — all output is `textContent`

### 3.6 Working Directory
- A `cwd` (current working directory) concept is maintained in JS module state
- `cd <path>` is a local operation — it changes the `cwd` variable and passes it to subsequent `ls` and `cat` commands as a path prefix
- The prompt reflects the cwd: `yos /myos/shell $ █`
- `cd` without path resets to the default root (`/myos`)

### 3.7 Clear
`clear` is handled entirely in the frontend — it does not call the Rust exec service. It clears the output area's DOM content.

---

## 4. Rust Command: exec_command

```rust
#[tauri::command]
#[specta::specta]
pub async fn exec_command(
    module_id: String,
    cmd: String,
    args: Vec<String>,
    cwd: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<ExecResult, String>

pub struct ExecResult {
    pub stdout:    String,
    pub stderr:    String,
    pub exit_code: i32,
    pub truncated: bool,   // true if output exceeded 64KB
}
```

**Internal implementation rules:**
- `module_id` is checked against Permission Guard first
- `cmd` is checked against `ALLOWED_COMMANDS` const (compile-time)
- `cwd` is validated as an allowed root path before being set
- `args` are passed as-is to `Command::new(&cmd).args(&args)` — no shell, no interpolation
- `stdout` + `stderr` are each capped at 32KB — truncated flag set if exceeded
- Execution timeout: 10 seconds. Process killed after timeout.
- All executions are logged to `audit_log`

---

## 5. API Calls

| Call | When |
|------|------|
| `yosApi.execCommand(MODULE_ID, cmd, args, cwd)` | On Enter in input bar |
| `yosApi.queryAudit(MODULE_ID, { module_id: MODULE_ID, limit: 50 })` | When `history` command run |

---

## 6. Events Listened

| Event | Action |
|-------|--------|
| `session:mode_changed` | Show Amnesic Mode banner in output area |

---

## 7. Events Emitted

None. The Terminal does not emit events to other modules.

---

## 8. Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Execute command |
| `↑` Arrow | Previous command in history |
| `↓` Arrow | Next command in history |
| `Ctrl+C` | Clear current input (does not kill running command — no background processes) |
| `Ctrl+L` | Clear terminal output (same as `clear`) |
| `Tab` | Autocomplete from allowed commands list |
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close current tab (or window if last tab) |
| `Ctrl+[1–5]` | Switch to tab N |

---

## 9. Window Configuration

| Property | Value |
|----------|-------|
| Default width | 720px |
| Default height | 480px |
| Min width | 480px |
| Min height | 300px |
| Font | `var(--yos-font-mono)` throughout |

---

## 10. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| No background processes | Exec is synchronous — no `&` operator | Phase 4: async job queue |
| No piping (`\|`) | Would require shell interpreter | No — by design |
| No environment variable expansion | Same reason | No — by design |
| History not persisted across sessions | Phase 3: SQLite terminal_history table | Phase 3 |
| No stdin input to running commands | Exec is one-shot | Phase 4: interactive mode |
| Tab output not persisted | Memory only | Intentional |
