---
name: cadence-prepare-release
description: Run all pre-release checks: cargo check, clippy, fmt, tsc typecheck, and vite build. Use before publishing a new version.
license: MIT
compatibility: Requires Rust toolchain, Node.js, and npm.
metadata:
  author: cadence
  version: "1.0"
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

Ready for release.
```
