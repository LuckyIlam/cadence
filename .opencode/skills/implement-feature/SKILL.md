---
name: implement-feature
description: Implement a simple feature from a conversation prompt: analyse, branch, propose, test, verify, document, commit.
license: MIT
compatibility: Requires Rust toolchain, Node.js, npm.
metadata:
  author: cadence
  version: "1.0"
  generatedBy: "1.5.0"
---

Implement a feature request that comes directly from a conversation prompt (not from a GitHub issue).  
For complex features (new module, architectural change, specs to draft), use OpenSpec instead.

## Workflow

### 1. Analyse the request

Extract from the conversation: what is the feature, expected behavior, constraints.  
Reformulate to the user for validation before proceeding.

### 2. Create a branch

```bash
git checkout -b feat/<short-descriptive-name>
```

### 3. Explore the codebase

Search for relevant files using grep, glob, and read to understand the current implementation.  
If the request references an existing feature, read its specs and docs too.

### 4. Propose a solution

Present the plan to the user: which files will change, the technical approach.  
Wait for explicit approval before implementing.

### 5. Implement the changes

Make the necessary code changes following project conventions:
- Rust backend: domain → repositories → commands → lib.rs
- Frontend: types → components → pages

Write **systematic tests for every new business function on the Rust side** (unit or integration tests following existing patterns: `#[cfg(test)]` in the module, in-memory SQLite database).  
→ If the user considers a test unnecessary for a specific function, they must explicitly validate it before commit.

### 6. Verify

Run all checks sequentially. If any fails, stop and fix before continuing.

**Rust:**
Working directory: `src-tauri`

```bash
cargo check
```

```bash
cargo clippy -- -D warnings
```

```bash
cargo fmt --check
```

```bash
cargo test
```

**TypeScript / Vite:**

```bash
npm run typecheck
```

```bash
npm run build
```

### 7. Update functional documentation

If the change impacts functional behavior (new feature, modified rule, UI change), update the corresponding file in `docs/fonctionnel/` to keep user-facing documentation in sync with the code.

### 8. Commit

Stage all changed files, write a descriptive commit message, then present to the user for push/merge decision:

```bash
git add -A
git commit -m "<description of the feature>"
```

### Integration with OpenSpec

- If during exploration the feature turns out to be **more complex than expected** (new module, architectural change, requires formal specs), suggest switching to the OpenSpec workflow.
- Simple features that modify or extend existing behavior stay in this skill.
