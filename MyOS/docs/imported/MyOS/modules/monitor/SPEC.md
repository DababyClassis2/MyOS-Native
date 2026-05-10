# Activity Monitor — Module Specification

**Module ID:** `com.yfitops.monitor`  
**Status:** ✅ Stable  
**Version:** 8.0  
**Permissions:** `SystemMonitor`, `NetworkRead`  
**Trusted:** Yes

---

## 1. Purpose

The Activity Monitor gives the user a live view of their machine's resource consumption and running processes. It is the system's self-awareness interface — showing what the OS is doing at any moment. Inspired by macOS Activity Monitor and htop, but designed to fit the Yfitops aesthetic.

---

## 2. Layout

```
┌─ ● ● ● ──────── Activity Monitor ─────────── [cpu] [mem] [net] ─┐
├─────────────────────────────────────────────────────────────────┤
│  CPU                                                 23%         │
│  ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   6 cores     │
│                                                                  │
│  Memory                                             4.2 / 16 GB │
│  ██████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   26%         │
│                                                                  │
│  Disk  /                                          38 / 120 GB   │
│  ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░   32%         │
│                                                                  │
│  Network   ↑ 0.0 KB/s    ↓ 12.4 KB/s                           │
├─────────────────────────────────────────────────────────────────┤
│  Processes                                       [filter...]    │
│  PID    Name                    CPU%    MEM                     │
│  1      init                     0.0    1.2 MB                  │
│  842    node shell.js             1.2   48.6 MB                 │
│  1044   yfitops-os               3.1   112.4 MB                 │
│  1891   ps                       0.0    0.4 MB                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Features

### 3.1 Resource Bars
- CPU, Memory, Disk bars update every 2 seconds
- Bar fills proportionally to percentage
- Colour transitions: green (0–60%) → amber (60–85%) → red (85–100%)
- Numbers show both absolute and percentage

### 3.2 Process List
- Shows all running processes from `sysinfo`
- Columns: PID, Name, CPU%, Memory (RSS)
- Default sort: CPU% descending
- Click column header to sort ascending/descending
- Filter input: text filter on process name (client-side, no Rust call)
- Update interval: 5 seconds (slower than resource bars to reduce noise)

### 3.3 Network Tab
- Active network interfaces and their current throughput
- Upload and download rates in KB/s or MB/s (auto-scaled)
- Interface name and IP address

---

## 4. Rust Command

```rust
#[tauri::command]
#[specta::specta]
pub async fn get_system_health(
    module_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<SystemHealth, String>

pub struct SystemHealth {
    pub cpu_percent:    f32,
    pub cpu_cores:      usize,
    pub mem_used_gb:    f32,
    pub mem_total_gb:   f32,
    pub disk_used_gb:   f32,
    pub disk_total_gb:  f32,
    pub disk_path:      String,
    pub net_up_kbps:    f32,
    pub net_down_kbps:  f32,
    pub processes:      Vec<ProcessEntry>,
}

pub struct ProcessEntry {
    pub pid:        u32,
    pub name:       String,
    pub cpu_pct:    f32,
    pub mem_bytes:  u64,
}
```

---

## 5. Polling Architecture

The Activity Monitor uses a frontend `setInterval` to poll `get_system_health` every 2 seconds. The process list is refreshed every 5 seconds.

Future (Phase 4): Replace polling with a Rust `emit()` loop so the health data is pushed rather than pulled. This reduces IPC overhead.

---

## 6. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| No kill process button | ExecWrite needed; deliberate omission | Phase 4 ADR required |
| No per-core CPU breakdown | sysinfo provides aggregate only | Phase 4 |
| No historical graphs (sparklines) | Deferred | Phase 4 |
| No disk I/O rate | sysinfo limitation | Phase 4 |
