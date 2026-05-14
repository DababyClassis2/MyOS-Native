# Settings — Module Specification

**Module ID:** `com.yfitops.settings`  
**Status:** 🔶 Phase 3 (migrating to SQLite backend)  
**Version:** 8.0  
**Permissions:** `FileRead`, `NetworkRead`, `PrivacyControl`  
**Trusted:** Yes

---

## 1. Purpose

Settings is the control panel of Yfitops OS. It is where the user configures every persistent preference, manages profiles, controls privacy features, and reviews system information. It is the only module with `PrivacyControl` permission, making it the gatekeeper for Privacy Shield configuration.

---

## 2. Layout

```
┌─ ● ● ● ──────── Settings ──────────────────────────────────────┐
├───────────────┬────────────────────────────────────────────────┤
│  Appearance   │  Appearance                                    │
│  Window Mgr   │  ───────────────────────────────────────────  │
│  Privacy      │  Theme                                        │
│  Session      │    ● Sovereign Dark    ○ Custom               │
│  Profiles     │                                               │
│  Modules      │  Accent Colour                                │
│  About        │    [████] #00ff88                             │
│               │                                               │
│               │  Font Size                                    │
│               │    ○ 11px  ● 13px  ○ 15px                    │
│               │                                               │
│               │  Dock Position                                │
│               │    ● Bottom  ○ Left  ○ Right                 │
│               │                                               │
│               │  Wallpaper                                    │
│               │    [Choose File]  /myos/wallpaper.jpg         │
└───────────────┴────────────────────────────────────────────────┘
```

---

## 3. Settings Sections

### 3.1 Appearance
| Setting key | Type | Default | Description |
|-------------|------|---------|-------------|
| `appearance.theme` | enum | `sovereign-dark` | Theme name |
| `appearance.accent_color` | string | `#00ff88` | CSS hex colour |
| `appearance.font_size` | enum | `13` | Base font size in px |
| `appearance.dock_position` | enum | `bottom` | Dock placement |
| `appearance.wallpaper_path` | string | `""` | Path to wallpaper file |

### 3.2 Window Manager
| Setting key | Type | Default | Description |
|-------------|------|---------|-------------|
| `wm.restore_on_launch` | bool | `true` | Restore window positions on launch |
| `wm.snap_enabled` | bool | `true` | Enable snap zones |
| `wm.focus_mode_duration_min` | int | `25` | Pomodoro timer length in minutes |
| `wm.animations_enabled` | bool | `true` | Enable window open/close animations |

### 3.3 Privacy
| Setting key | Type | Default | Description |
|-------------|------|---------|-------------|
| `privacy.mac_randomize` | bool | `false` | Randomise MAC on session start |
| `privacy.timezone_guard` | bool | `false` | Mask timezone in headers |
| `privacy.network_guardian` | bool | `false` | Enable Network Guardian |
| `privacy.shield_autostart` | bool | `false` | Start Privacy Shield with app |

### 3.4 Session
| Setting key | Type | Default | Description |
|-------------|------|---------|-------------|
| `session.default_mode` | enum | `Sovereign` | Default mode on launch |
| `session.confirm_mode_switch` | bool | `true` | Show confirmation on mode switch |
| `session.audit_retention` | int | `50000` | Max audit log rows |

### 3.5 Profiles
- List of all profiles (`list_profiles` command)
- Active profile indicator
- Create new profile button
- Switch profile button
- Delete profile button (with confirmation — cannot delete the currently active profile)

### 3.6 Modules
- List of installed `.yops` packages (links to Package Manager)
- Each entry shows module name, version, enabled toggle
- No install/uninstall from here — that is the Package Manager's responsibility

### 3.7 About
- Yfitops OS version
- Build date
- Tauri version
- Rust version (at build time)
- Open source acknowledgements

---

## 4. Rust Commands

| Command | Signature |
|---------|-----------|
| `get_setting` | `(module_id, key, profile_id) → Option<String>` |
| `set_setting` | `(module_id, key, value, profile_id) → ()` |
| `list_settings` | `(module_id, profile_id) → Vec<Setting>` |
| `get_all_settings` | `(module_id, profile_id) → HashMap<String, String>` |
| `reset_settings` | `(module_id, profile_id) → ()` — resets to defaults |
| `export_settings` | `(module_id, profile_id) → String` — JSON export |

Settings changes emit a `settings:changed` event immediately after the DB write so other modules can react (e.g., dock repositions if `appearance.dock_position` changes).

---

## 5. Events

| Event | Payload | Direction |
|-------|---------|-----------|
| `settings:changed` | `{ key: String, value: String, profile_id: String }` | Rust → All modules |
| `theme:updated` | `ThemePalette` | Rust → All modules (on wallpaper change) |
| `profile:switched` | `{ profile_id: String }` | Rust → All modules |

The desktop shell (`index.html`) listens for `settings:changed` and applies changes to the dock, top bar, and wallpaper without requiring a restart.

---

## 6. Settings Change Propagation

When a setting changes, the desktop shell and all open module windows receive `settings:changed`. Each module is responsible for applying the setting that is relevant to it.

```javascript
// In every module's app.js
await listen('settings:changed', (event) => {
    const { key, value } = event.payload;
    if (key === 'appearance.font_size') {
        document.documentElement.style.setProperty(
            '--yos-text-base', `${value}px`
        );
    }
    if (key === 'appearance.accent_color') {
        document.documentElement.style.setProperty('--yos-accent', value);
    }
});
```

---

## 7. Known Limitations

| Limitation | Reason | Future fix? |
|-----------|--------|-------------|
| Privacy section inactive until Phase 5 | Privacy Shield not built | Phase 5 |
| No setting search | Deferred | Phase 4 |
| Import settings blocked in Amnesic Mode | FileRead partially blocked | By design |
| No per-module settings UI | Modules define their own settings | Future: module-local settings store |
