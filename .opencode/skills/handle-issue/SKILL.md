---
name: handle-issue
description: Handle a GitHub issue: explore, propose, implement, verify, and comment. Pass the issue number as parameter.
license: MIT
compatibility: Requires Rust toolchain, Node.js, npm, and GitHub CLI (gh).
metadata:
  author: cadence
  version: "1.0"
  generatedBy: "1.5.0"
---

Handle a GitHub issue from reading to closing. The issue number is passed as `<issue-number>`.

## Workflow

### 1. Read the issue

```bash
gh issue view <issue-number>
```

Parse title, description, labels, and comments. Understand the problem and the expected behavior.

### 2. Explore the codebase

Search for relevant files using grep, glob, and read to understand the current implementation. Identify all files that need to change.

### 3. Propose a solution

Summarize the problem and your proposed solution to the user. Wait for their approval before implementing.

### 4. Implement the changes

Make the necessary code changes following project conventions:
- Rust backend: domain → repositories → commands → lib.rs
- Frontend: types → components → pages

### 5. Verify

Run all checks sequentially. If any fails, stop and fix.

**Rust:**
Working directory: `src-tauri`

```bash
cargo check
```

```bash
cargo clippy -- -D warnings
```

**TypeScript / Vite:**

```bash
npm run typecheck
```

```bash
npm run build
```

### 6. Comment the issue

Write a summary of changes and post it on the issue:

```bash
gh issue comment <issue-number> --body-file <temp-file>
```

The comment should include:
- What was fixed/changed
- List of modified files with brief descriptions
- Benefits of the change

### 7. Propose to close

Ask the user if they want to close the issue.

## Execution

- After step 1, present the issue summary to the user
- After step 3, wait for user approval before implementing
- If any verification step fails, report the error and fix before continuing
- Always get user confirmation before closing the issue
