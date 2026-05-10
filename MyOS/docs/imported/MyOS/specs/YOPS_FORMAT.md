# YOPS_FORMAT.md — .yops Package Format Specification

**Version:** 1.0  
**Status:** Specification — Phase 4  
**Last updated:** 2026-05

---

## 1. Overview

`.yops` is the native package format for Yfitops OS. A `.yops` file is a self-contained module bundle: a manifest describing the module's identity and permissions, and the HTML/CSS/JS assets that make up the module's UI.

Design goals for the format:
- **Human-readable manifest** — JSON, no binary encoding
- **Verifiable** — SHA-256 signature of the asset bundle
- **Minimal** — no dependencies beyond the manifest and assets
- **Installable offline** — no network call during install
- **Auditable** — all permissions visible to the user before install

---

## 2. File Format

A `.yops` file is a renamed ZIP archive containing:

```
my-module.yops
├── manifest.json          # Required — module identity and permissions
├── index.html             # Required — module entry point
├── app.js                 # Optional — module logic
├── style.css              # Optional — module styles
├── assets/                # Optional — images, fonts, etc.
│   └── ...
└── CHECKSUMS.sha256       # Required — SHA-256 hash of every file
```

The `.yops` extension is cosmetic. The file is a standard ZIP. It can be inspected with any ZIP tool.

---

## 3. manifest.json Schema

```json
{
  "manifest_version": 1,
  "id": "com.example.my-module",
  "name": "My Module",
  "version": "1.0.0",
  "description": "A short description of what this module does.",
  "author": "Author Name or Handle",
  "entry": "index.html",
  "permissions": [
    "SystemMonitor",
    "ClipboardWrite"
  ],
  "trusted": false,
  "min_os_version": "8.0.0",
  "signature": "sha256:abc123def456..."
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `manifest_version` | integer | Yes | Always `1` for this format version |
| `id` | string | Yes | Reverse-DNS style unique ID. Pattern: `^[a-z0-9]+(\.[a-z0-9]+){2,}$` |
| `name` | string | Yes | Display name shown to user. Max 32 characters. |
| `version` | string | Yes | Semver: `MAJOR.MINOR.PATCH` |
| `description` | string | Yes | One-sentence description. Max 120 characters. |
| `author` | string | Yes | Author name or handle. Max 64 characters. |
| `entry` | string | Yes | Path to the HTML entry point within the package. Always relative. |
| `permissions` | array | Yes | List of `Permission` enum values. May be empty `[]`. |
| `trusted` | boolean | Yes | Always `false` for third-party packages. `true` is reserved for system modules. |
| `min_os_version` | string | Yes | Minimum Yfitops OS version required. Semver. |
| `signature` | string | Yes | `sha256:` prefix + hex-encoded SHA-256 of the asset bundle (all files except `manifest.json`). |

---

## 4. Validation Rules

The Rust `install_package()` command runs these checks in order. Any failure aborts installation with a specific error message.

### 4.1 Manifest Validation
- [ ] `manifest.json` is valid JSON
- [ ] All required fields are present
- [ ] `id` matches the pattern `^[a-z0-9]+(\.[a-z0-9]+){2,}$`
- [ ] `version` is valid semver
- [ ] `manifest_version` equals `1`
- [ ] `trusted` is `false` (third-party packages cannot be trusted)
- [ ] `name` length ≤ 32 characters
- [ ] `description` length ≤ 120 characters
- [ ] `min_os_version` ≤ current Yfitops OS version

### 4.2 Permission Validation
- [ ] All declared permissions are valid `Permission` enum values
- [ ] No elevated permissions: `NetworkControl`, `PackageInstall`, `PrivacyControl` are rejected for `trusted: false` packages
- [ ] Permission list contains no duplicates

### 4.3 Asset Validation
- [ ] `entry` file exists in the ZIP
- [ ] `CHECKSUMS.sha256` exists in the ZIP
- [ ] All files listed in `CHECKSUMS.sha256` exist and match their hashes
- [ ] `signature` in manifest matches the SHA-256 of the ZIP content (excluding `manifest.json`)
- [ ] No files attempt path traversal (no `../` in paths)
- [ ] Total unzipped size ≤ 50 MB

### 4.4 Duplicate Check
- [ ] No package with the same `id` is already installed (if upgrading, a separate `upgrade_package()` flow is used)

---

## 5. CHECKSUMS.sha256 Format

Standard SHA-256 checksum file format:

```
abc123def456...  index.html
789ghi012jkl...  app.js
345mno678pqr...  style.css
567stu890vwx...  assets/icon.png
```

One entry per file. Paths are relative to the ZIP root. `manifest.json` and `CHECKSUMS.sha256` itself are excluded.

---

## 6. Signature Scheme

The `signature` field in `manifest.json` is the SHA-256 hash of the binary content of all files in the ZIP except `manifest.json`, concatenated in lexicographic path order.

```
signature = sha256(
    sort_lexicographic([all_files_except_manifest]) 
    |> for each: read_bytes()
    |> concatenate_all_bytes()
)
```

This means:
- Changing any asset file invalidates the signature
- The signature is computed by the package author before distribution
- The installer verifies it on install, not on every launch

**Important:** This is integrity verification, not authenticity verification. It proves the package has not been tampered with since signing, but does not prove who signed it. Cryptographic author signing (GPG / Ed25519) is a Phase 5+ feature.

---

## 7. Package ID Namespace

| Namespace prefix | Reserved for |
|-----------------|-------------|
| `com.yfitops.*` | System modules — reserved |
| `com.yfitops.community.*` | Community-reviewed modules |
| `com.example.*` | Development and testing only |
| Everything else | Third-party packages |

The installer rejects packages with `id` starting with `com.yfitops.` unless `trusted: true` (which is only set by the build system for system modules).

---

## 8. Installation Directory

Installed packages are extracted to:

```
$APPDATA/yfitops/modules/<package-id>/
├── manifest.json
├── index.html
├── app.js
└── ...
```

The Tauri Window Manager loads the module from this path when `open_module()` is called with the package's ID.

---

## 9. Rust Structures

```rust
// src-tauri/src/commands/packages.rs

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct YopsManifest {
    pub manifest_version: u32,
    pub id:               String,
    pub name:             String,
    pub version:          String,
    pub description:      String,
    pub author:           String,
    pub entry:            String,
    pub permissions:      Vec<Permission>,
    pub trusted:          bool,
    pub min_os_version:   String,
    pub signature:        String,
}

#[derive(Serialize, Deserialize, Type)]
pub struct InstalledPackage {
    pub id:           String,
    pub name:         String,
    pub version:      String,
    pub description:  String,
    pub author:       String,
    pub permissions:  Vec<Permission>,
    pub enabled:      bool,
    pub installed_at: i64,
}
```

---

## 10. Creating a .yops Package (Author Guide)

Step-by-step to create a valid `.yops` package:

```bash
# 1. Create your module directory
mkdir my-module
cd my-module

# 2. Create your files
# index.html, app.js, style.css ...

# 3. Generate CHECKSUMS.sha256
find . -type f ! -name 'manifest.json' ! -name 'CHECKSUMS.sha256' \
  | sort \
  | xargs sha256sum \
  > CHECKSUMS.sha256

# 4. Compute the bundle signature
find . -type f ! -name 'manifest.json' ! -name 'CHECKSUMS.sha256' \
  | sort \
  | xargs cat \
  | sha256sum
# Copy the hash output

# 5. Create manifest.json with the signature hash
cat > manifest.json << EOF
{
  "manifest_version": 1,
  "id": "com.yourname.my-module",
  "name": "My Module",
  "version": "1.0.0",
  "description": "Does something useful.",
  "author": "Your Name",
  "entry": "index.html",
  "permissions": [],
  "trusted": false,
  "min_os_version": "8.0.0",
  "signature": "sha256:<paste hash here>"
}
EOF

# 6. Pack into .yops (ZIP)
cd ..
zip -r my-module.yops my-module/

# 7. Verify with the Yfitops validator (CLI tool — Phase 4)
yops validate my-module.yops
```

---

## 11. Future: CLI Validator Tool

A standalone CLI tool `yops` will be provided in Phase 4 to validate, inspect, and create `.yops` packages:

```bash
yops validate my-module.yops     # Full validation
yops inspect my-module.yops      # Show manifest and file list
yops sign my-module.yops         # Compute and embed signature
yops info my-module.yops         # Short summary
```
