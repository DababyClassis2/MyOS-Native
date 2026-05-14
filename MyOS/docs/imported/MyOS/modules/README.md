# modules/ — Module Documentation Index

Each directory contains a `SPEC.md` for that module. These are the authoritative specifications — UI, API contract, events, permissions, and known limitations.

## System Modules

| Module | ID | Status | Spec |
|--------|----|--------|------|
| Terminal | `com.yfitops.terminal` | ✅ Stable | [terminal/SPEC.md](terminal/SPEC.md) |
| File Browser | `com.yfitops.files` | ✅ Stable (read-only) | [files/SPEC.md](files/SPEC.md) |
| Activity Monitor | `com.yfitops.monitor` | ✅ Stable | [monitor/SPEC.md](monitor/SPEC.md) |
| Notes | `com.yfitops.notes` | 🔶 Phase 3 | [notes/SPEC.md](notes/SPEC.md) |
| Clipboard | `com.yfitops.clipboard` | ✅ Stable | [clipboard/SPEC.md](clipboard/SPEC.md) |
| Settings | `com.yfitops.settings` | 🔶 Phase 3 | [settings/SPEC.md](settings/SPEC.md) |
| Log Viewer | `com.yfitops.logs` | 🔶 Phase 3 | [logs/SPEC.md](logs/SPEC.md) |
| Package Manager | `com.yfitops.packages` | 🔲 Phase 4 | [packages/SPEC.md](packages/SPEC.md) |

## Rules for All Modules
- Read `docs/MODULES.md` before creating or modifying any module
- Read `security/PERMISSION_MANIFEST.md` to confirm the permission scope
- Read `specs/THEMING.md` for all CSS tokens and component patterns
- No module imports or depends on another module
- No module makes direct `invoke()` calls — always through `yos-api.ts`
