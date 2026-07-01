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

### 2. Create a branch

```bash
git checkout -b fix/issue-<issue-number>
```

### 3. Explore the codebase

Search for relevant files using grep, glob, and read to understand the current implementation. Identify all files that need to change.

### 4. Propose a solution

Summarize the problem and your proposed solution to the user. Wait for their approval before implementing.

### 5. Implement the changes

Make the necessary code changes following project conventions:
- Rust backend: domain → repositories → commands → lib.rs
- Frontend: types → components → pages

### 6. Verify

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

### 7. Commit and push

Stage all changed files, write a concise commit message describing the fix, then push:

```bash
git add -A
git commit -m "<description of the fix>"
git push -u origin fix/issue-<issue-number>
```

### 8. Comment the issue

Write a summary of changes and post it on the issue:

```bash
gh issue comment <issue-number> --body-file <temp-file>
```

The comment should include:
- What was fixed/changed
- List of modified files with brief descriptions
- Benefits of the change

### 9. Propose to close

Ask the user if they want to close the issue.

## Execution

- After step 1, present the issue summary to the user and create the fix branch
- After step 4, wait for user approval before implementing
- If any verification step fails, report the error and fix before continuing
- Commit with a descriptive message, not "fix" — explain what and why
- Always get user confirmation before closing the issue
