# PRIVACY_SHIELD.md — Privacy Shield Specification

**Version:** 0.1 (Draft)  
**Status:** Phase 5 — Do not implement until Phase 4 is complete and audited  
**Last updated:** 2026-05

---

## 1. Purpose

The Privacy Shield is a separate Rust binary that runs as a child process of the main Yfitops OS application. It manages the user's network identity, connection privacy, and data exposure footprint. It is the enforcement arm of the project's sovereignty philosophy.

**Critical design rule:** The Privacy Shield is a separate process. It is not a module, not a Rust function inside the main binary, not a toggle in settings. It is architecturally isolated so that:

1. A crash or compromise of the main process does not automatically compromise the shield
2. The shield can enforce network policies even if the main process is misbehaving
3. The shield can be independently audited, updated, and replaced

---

## 2. Architecture

```
Main Process (Yfitops OS)
        │
        │  Unix socket (Linux)
        │  Named pipe  (Windows)
        │
        ▼
Privacy Shield Process (separate binary)
        │
        ├── NetworkGuardian    → per-process network monitoring
        ├── MACRandomiser      → randomise MAC address on session start
        ├── HeaderScrubber     → modify outbound HTTP headers
        ├── TimezoneGuard      → mask system timezone in headers
        └── PrivacyHeatmap     → tracks data exposure events
```

IPC between the main process and the shield uses a simple line-delimited JSON protocol over a Unix socket (Linux) or named pipe (Windows). There is no HTTP between them.

---

## 3. Components

### 3.1 Network Guardian

Monitors active network connections per process using `/proc/net/tcp` (Linux) or equivalent. Detects unexpected outbound connections from Yfitops OS processes.

Emits events to the main process when:
- An unexpected connection is opened by any Yfitops process
- A connection is blocked by the Guardian's allow-list
- A new network interface comes online

The main process renders these events in the Privacy Heatmap module.

**Default policy:** Allow-list only. No connection is allowed unless explicitly listed by the user or by the system (e.g., local loopback for IPC).

### 3.2 MAC Randomiser

On shield process start (which occurs at app launch), the MAC Randomiser generates a new random MAC address for the active network interface. On shield process exit (app close), it restores the original MAC.

```
Original MAC:  aa:bb:cc:dd:ee:ff
Randomised:    3a:12:f4:89:c1:07
```

**Platform notes:**
- Linux: uses `ip link set <interface> address <mac>`
- Windows: uses `SetAdaptersAddresses` Win32 API
- Requires elevated privileges — the shield process runs with the minimum privilege needed for this operation only

**User control:** The user can disable MAC randomisation from Settings → Privacy. The shield records the decision to the audit log.

### 3.3 Header Scrubber

Intercepts outbound HTTP/HTTPS requests (where possible via proxy injection or OS-level interception) and removes or replaces identifying headers:

| Header | Action |
|--------|--------|
| `User-Agent` | Replace with generic, non-fingerprinting value |
| `Accept-Language` | Replace with `en-US,en;q=0.9` |
| `X-Forwarded-For` | Remove |
| `Via` | Remove |
| `Referer` | Strip to origin only, or remove |
| `Cookie` (third-party) | Block |
| `DNT` | Set to `1` |

**Note:** This component is the most complex because it requires inserting a proxy into the network path. Implementation approach is TBD (options: user-space proxy, eBPF on Linux, WFP on Windows). This is the last component to be implemented.

### 3.4 Timezone Guard

The system timezone can be used for fingerprinting. The Timezone Guard intercepts timezone queries and returns a neutral value.

- Reports UTC offset `+00:00` in all HTTP headers
- Does not modify the actual system timezone (only the reported value)
- Yfitops OS UI shows the user's actual local time but reports UTC externally

### 3.5 Privacy Heatmap

A data structure maintained by the shield that tracks exposure events. The main process queries it on a 5-second interval and renders it in the Privacy Heatmap module.

```rust
pub struct ExposureEvent {
    pub ts:          i64,
    pub category:    ExposureCategory,
    pub module_id:   String,
    pub detail:      String,
    pub risk_level:  RiskLevel,
}

pub enum ExposureCategory {
    NetworkCall,
    DiskWrite,
    ClipboardAccess,
    SystemInfoRead,
    PermissionRequest,
}

pub enum RiskLevel {
    Low,    // normal operation
    Medium, // unusual but not a threat
    High,   // potential privacy violation
}
```

---

## 4. IPC Protocol

All messages are line-delimited JSON. Each line is one message.

### Main → Shield Commands

```json
{ "cmd": "get_status" }
{ "cmd": "set_mac_random", "enabled": true }
{ "cmd": "set_timezone_guard", "enabled": true }
{ "cmd": "allow_connection", "process": "yfitops-os", "host": "127.0.0.1", "port": 8080 }
{ "cmd": "block_connection", "process": "yfitops-os", "host": "example.com" }
{ "cmd": "get_heatmap", "since_ts": 1717200000 }
{ "cmd": "shutdown" }
```

### Shield → Main Events

```json
{ "event": "status", "mac_random": true, "tz_guard": true, "connections_blocked": 0 }
{ "event": "connection_detected", "process": "yfitops-os", "host": "example.com", "port": 443, "blocked": false }
{ "event": "connection_blocked", "process": "yfitops-os", "host": "tracking.example.com", "port": 443 }
{ "event": "exposure", "category": "NetworkCall", "module_id": "com.yfitops.terminal", "detail": "example.com:443", "risk_level": "Low" }
{ "event": "mac_changed", "from": "aa:bb:cc:dd:ee:ff", "to": "3a:12:f4:89:c1:07" }
{ "event": "error", "code": "MAC_PERMISSION_DENIED", "message": "..." }
```

---

## 5. Fail-Safe Behaviour

The Privacy Shield must fail closed. If the shield process crashes or becomes unresponsive:

1. The main process detects the disconnect (read error on the socket)
2. Emits `privacy_shield:disconnected` event to the frontend
3. The Privacy HUD in the top bar turns red with a warning icon
4. The main process attempts to restart the shield process (3 retries, 2-second intervals)
5. If restart fails: displays a persistent error banner and optionally blocks network access

```rust
// Main process shield supervision
loop {
    match shield_process.wait_with_output() {
        Ok(_) => {
            // Shield exited — attempt restart
            try_restart_shield(&app_handle, &mut retry_count).await;
        }
        Err(e) => {
            app_handle.emit("privacy_shield:error", e.to_string());
        }
    }
}
```

---

## 6. Non-Goals

The Privacy Shield explicitly does not:

- Evade bans or restrictions on platforms or services
- Impersonate other users or devices
- Bypass authentication or security controls on external services
- Provide anonymity against a nation-state-level adversary
- Protect against physical attacks or hardware-level surveillance

Its goal is protecting the user from casual fingerprinting and tracking by ordinary web services and data brokers. It is a privacy tool, not an operational security tool.

---

## 7. Platform Considerations

| Feature | Linux | Windows |
|---------|-------|---------|
| MAC randomisation | `ip link` | `SetAdaptersAddresses` (requires admin or UAC) |
| Network monitoring | `/proc/net/tcp` | `GetExtendedTcpTable` Win32 API |
| Proxy injection | `iptables` redirect or `LD_PRELOAD` | WFP (Windows Filtering Platform) |
| Unix socket IPC | Native | Named pipe (`\\.\pipe\yfitops-shield`) |
| Process isolation | `setuid` + `seccomp` | Windows ACLs + Job Objects |

Linux implementation is prioritised first (matches development environment). Windows implementation follows.

---

## 8. Implementation Prerequisites

Before implementation begins, the following must be in place:

- [ ] Phase 4 complete and stable (all modules, permission guard, Amnesic Mode)
- [ ] `THREAT_MODEL.md` updated with Privacy Shield threat analysis
- [ ] ADR created for the IPC protocol choice
- [ ] ADR created for the proxy injection approach (Linux)
- [ ] Legal review of MAC randomisation implications per jurisdiction
- [ ] User-facing documentation explaining what the shield does and does not do
