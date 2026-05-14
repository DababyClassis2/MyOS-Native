# CONTRIBUTING.md — Engineering Standards and Contribution Protocol

**This document is the engineering law of the project.**  
All code, documentation, and decisions must conform to it.

---

## 1. Core Engineering Values

In order of priority when values conflict:

1. **Correctness** — code does exactly what it claims to do, in all cases
2. **Security** — code does not open attack surfaces or weaken the privacy model
3. **Simplicity** — the simplest correct solution is the right solution
4. **Readability** — code is written to be read by the next person (including future you)
5. **Performance** — optimise only when correctness and readability are satisfied

---

## 2. Before Writing Any Code

Complete this checklist before opening any file in an editor:

- [ ] Read the `SPEC.md` for the module being modified
- [ ] Read `ARCHITECTURE.md` to understand the affected layer
- [ ] Check `DECISIONS.md` to confirm no ADR contradicts the planned change
- [ ] Check `security/THREAT_MODEL.md` if the change touches security, permissions, or exec
- [ ] Write down in plain language: *what is the problem, what is the change, what does it break*
- [ ] If it breaks anything — stop and discuss before proceeding

---

## 3. Commit Message Format

```
<type>(<scope>): <imperative summary in 72 characters or less>

[optional body: explain the why, not the what]

[optional footer: Closes #123, Breaks: <component>]
```

**Types:**

| Type | When to use |
|------|-------------|
| `feat` | New capability added |
| `fix` | Bug corrected |
| `refactor` | Code changed without behaviour change |
| `docs` | Documentation only |
| `security` | Security fix or hardening |
| `chore` | Build config, dependencies, tooling |
| `perf` | Performance improvement |
| `test` | Tests added or modified |

**Scopes:** `terminal`, `files`, `notes`, `monitor`, `clipboard`, `settings`, `logs`, `packages`, `shell`, `security`, `db`, `ipc`, `docs`, `build`

**Examples:**
```
feat(notes): add FTS5 full-text search via SQLite virtual table

fix(exec): reject commands with path separators in arguments

security(permission_guard): log denial events to audit_log with WARN severity

docs(decisions): ADR-007 — one DB file per device, profile_id column per table

chore(deps): update tauri-plugin-sql to 2.1.0
```

Bad commit messages that will be rejected:
```
fix things
update
wip
almost done
```

---

## 4. Rust Code Standards

### Formatting
Run `cargo fmt` before every commit. Non-formatted code is rejected.

### Linting
Run `cargo clippy -- -D warnings` before every commit. All warnings are errors.

### Error Handling
- Never use `.unwrap()` in production code paths. Use `?` or explicit `match`.
- Never use `.expect("message")` unless the failure represents a programmer error (unrecoverable bug), not a user-triggered condition.
- All Tauri commands return `Result<T, String>`. The `String` error is the user-facing message.
- Error messages describe what happened, not internal state. Bad: `"index out of bounds"`. Good: `"Note not found: {id}"`.

### Command Pattern
Every Tauri command follows this exact structure:

```rust
#[tauri::command]
#[specta::specta]
pub async fn command_name(
    module_id: String,                       // Always first
    // ... other args ...
    state: tauri::State<'_, AppState>,       // Always last
) -> Result<ReturnType, String> {
    // 1. Permission check — always first, always blocking
    state.permission_guard
        .assert(&module_id, Permission::RequiredPermission)
        .map_err(|e| e.to_string())?;

    // 2. Input validation
    if input.is_empty() {
        return Err("Input cannot be empty".into());
    }

    // 3. Business logic
    let result = do_work().await.map_err(|e| e.to_string())?;

    // 4. Audit log write
    state.write_audit(&module_id, "action_name", Some(&format!("{:?}", result)), "INFO").await;

    // 5. Return
    Ok(result)
}
```

### SQL Pattern
```rust
// CORRECT — prepared statement
sqlx::query_as!(Note, "SELECT * FROM notes WHERE id = ? AND profile_id = ?", id, profile_id)
    .fetch_one(&*db)
    .await
    .map_err(|e| e.to_string())?;

// NEVER — string interpolation
let query = format!("SELECT * FROM notes WHERE id = '{}'", id); // REJECTED
```

### State Access Pattern
```rust
// CORRECT
let db = state.db.lock().await;
let result = sqlx::query!(...).fetch_all(&*db).await?;
drop(db); // release lock before any await after this point

// WRONG — holding the lock across an await
let db = state.db.lock().await;
tokio::time::sleep(Duration::from_secs(1)).await; // deadlock risk
```

### Struct and Enum Naming
- Structs: `PascalCase`
- Enums: `PascalCase` with `PascalCase` variants
- Command functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Files: `snake_case.rs`

---

## 5. JavaScript Code Standards

### No eval, no innerHTML from API data
```javascript
// CORRECT
const el = document.createElement('div');
el.textContent = apiResponse.message; // sanitised

// NEVER
el.innerHTML = apiResponse.message; // XSS vector — REJECTED
eval(apiResponse.code);             // Never — REJECTED
```

### API calls always go through yos-api.ts
```javascript
// CORRECT
const notes = await window.yosApi.listNotes(MODULE_ID);

// NEVER
const result = await window.__TAURI__.core.invoke('list_notes', { module_id: MODULE_ID });
// Direct invoke bypasses the API wrapper — REJECTED
```

### Async error handling — no unhandled rejections
```javascript
// CORRECT
async function loadData() {
    try {
        const data = await window.yosApi.getData(MODULE_ID);
        render(data);
    } catch (err) {
        showError(err.message ?? 'Unknown error');
    }
}

// NEVER
window.yosApi.getData(MODULE_ID).then(render); // unhandled rejection — REJECTED
```

### Event listener cleanup — always
```javascript
// CORRECT
let unlisten;
window.addEventListener('DOMContentLoaded', async () => {
    unlisten = await window.__TAURI__.event.listen('event:name', handler);
});
window.addEventListener('unload', () => {
    if (unlisten) unlisten();
});

// NEVER
window.__TAURI__.event.listen('event:name', handler); // leaked listener — REJECTED
```

### Variable declarations
- `const` by default
- `let` only when reassignment is needed
- `var` never

---

## 6. CSS Standards

### Token usage — never hardcode colours
```css
/* CORRECT */
.my-element {
    color: var(--yos-text);
    background: var(--yos-surface);
    border: 1px solid var(--yos-border);
    border-radius: var(--yos-radius-md);
}

/* NEVER */
.my-element {
    color: #e8e8e8;           /* hardcoded — REJECTED */
    background: #111111;       /* hardcoded — REJECTED */
}
```

### Module CSS scope
Every module CSS file must prefix its selectors with the module root:
```css
/* CORRECT — scoped to module */
#app .note-list { ... }
#app .note-item { ... }

/* WRONG — global selector */
.note-list { ... }  /* pollutes global scope if modules ever share context */
```

### No `!important` except in `shared/theme.css` override rules.

---

## 7. Documentation Standards

### Code comments
- Explain *why*, not *what*. The code explains what.
- Every Rust `pub fn` gets a doc comment (`///`).
- Every public struct gets a doc comment describing its purpose and invariants.
- Every SQL migration gets a comment explaining the business reason.

```rust
/// Asserts that the calling module has the required permission.
/// Returns Ok(()) if the module is allowed, Err(denial_message) if not.
/// Denial is logged to the audit log automatically.
pub fn assert(&self, module_id: &str, permission: Permission) -> Result<(), String> {
```

### Spec files
Every module's `SPEC.md` must contain:
- Purpose and scope
- Permission declaration (matches Rust manifest)
- UI layout description with ASCII art or prose
- All API calls the module makes
- All events the module emits and subscribes to
- Known limitations

---

## 8. Security Review Checklist

Any change to the following areas requires a second review pass against this checklist:

**Exec service changes:**
- [ ] Is the command whitelist still hardcoded?
- [ ] Does the new code path accept user-controlled strings?
- [ ] Is the permission guard still the first operation?

**Database changes:**
- [ ] Are all new queries using prepared statements?
- [ ] Does every query on a profile-scoped table include `WHERE profile_id = ?`?
- [ ] Is the migration reversible?

**New module additions:**
- [ ] Is the manifest declaring minimum required permissions only?
- [ ] Is the module HTML using `textContent` not `innerHTML` for API data?
- [ ] Does the module clean up its event listeners?

**New permission additions:**
- [ ] Has an ADR been created for the new permission type?
- [ ] Is the new permission blocked from untrusted (.yops) modules if elevated?

---

## 9. Deployment Protocol

A change is not complete until it is reflected in `import.txt`.

1. Develop and test the change in `MyOS-Native/`
2. Verify `cargo build --release` succeeds with zero warnings
3. Verify `npm run tauri dev` starts cleanly
4. Update `import.txt` with the change
5. Test the full deployment path:
   ```bash
   cp /mnt/shared/import.txt /myos/shell/setup.sh
   cd /myos/shell && tr -d '\r' < setup.sh | sh
   ```
6. Document the change in the appropriate `SPEC.md` or `DECISIONS.md`
7. Commit with a correct commit message

---

## 10. What Needs an ADR

Create a new ADR in `DECISIONS.md` before making changes in any of these categories:

- Adopting a new dependency (Rust crate or JS package)
- Adding a new permission type to the `Permission` enum
- Expanding the exec whitelist
- Changing the database schema in a breaking way
- Changing the IPC event naming scheme
- Adopting a frontend framework
- Changing the session model
- Adding any form of network call
