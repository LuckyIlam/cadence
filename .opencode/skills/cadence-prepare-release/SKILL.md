---
name: cadence-prepare-release
description: Run all pre-release checks: cargo check, clippy, fmt, tsc typecheck, and vite build. Use before publishing a new version.
license: MIT
compatibility: Requires Rust toolchain, Node.js, and npm.
metadata:
  author: cadence
  version: "2.0"
  generatedBy: "1.5.0"
---

Run all pre-release checks in sequence. If any step fails, stop and report the error.

## Checks

### 1. Rust — cargo check

```bash
cargo check
```

Working directory: `src-tauri`

### 2. Rust — cargo clippy (deny warnings)

```bash
cargo clippy -- -D warnings
```

Working directory: `src-tauri`

### 3. Rust — cargo fmt

```bash
cargo fmt --check
```

Working directory: `src-tauri`

### 4. TypeScript — typecheck

```bash
npm run typecheck
```

### 5. Vite — production build

```bash
npm run build
```

## Execution

Run steps sequentially. If a step fails:
- Print the error
- Stop — do not continue to subsequent steps
- Report which step failed and why

If all steps pass:
- Print a summary table

```
┌──────────────────────┬──────────┐
│ Step                 │ Result   │
├──────────────────────┼──────────┤
│ cargo check          │ ✓ Passed │
│ cargo clippy         │ ✓ Passed │
│ cargo fmt            │ ✓ Passed │
│ tsc typecheck        │ ✓ Passed │
│ vite build           │ ✓ Passed │
└──────────────────────┴──────────┘
```

### 6. Ask — effectuer la release ?

Use the `question` tool to ask:

> Voulez-vous effectuer la release ?

- **Non** → print "Release annulée." and stop.
- **Oui** → continue to step 7.

### 7. Ask — type de version

Use the `question` tool to ask:

> S'agit-il d'une version mineure (Z) ou majeure (Y) ?
>
> Convention : X.Y.Z — Y est la version majeure, Z la version mineure.

Options:
- `"Mineure (Z+1)"` — ex: `0.2.0` → `0.2.1`
- `"Majeure (Y+1)"` — ex: `0.2.0` → `0.3.0`

### 8. Bump version

Read the current version from `package.json` (field `version`).

Parse `X.Y.Z`:
- If **Mineure** → `X.Y.(Z+1)`
- If **Majeure** → `X.(Y+1).0`

Update the version in these 3 files:

| File | Field |
|------|-------|
| `package.json` | `"version": "<new>"` |
| `src-tauri/tauri.conf.json` | `"version": "<new>"` |
| `src-tauri/Cargo.toml` | `version = "<new>"` |

### 9. Update documentation

Run the following commands to update documentation references:

- Update `README.md` if it mentions the version number anywhere
- Update files under `docs/fonctionnel/` if they reference the version

Check each file with grep for the old version string, and update any occurrences found.

### 10. Create release branch, commit, tag, and push

```bash
git checkout -b release/v<NEW_VERSION>
git add -A
git commit -m "release: v<NEW_VERSION>"
git tag v<NEW_VERSION>
git push origin release/v<NEW_VERSION>
git push origin v<NEW_VERSION>
```

Print a success message with the new version and branch name.

## Version files reference

- `package.json` — root directory
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
```
