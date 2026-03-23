# GitHub CLI (`gh`) Commands

A practical reference of `gh` commands for branch management, pull requests, and repository operations. All examples below were used in this project.

## Setup

```bash
# Install gh (macOS)
brew install gh

# Authenticate
gh auth login
```

---

## Branch Comparison

### Compare two branches (summary)

```bash
# Get ahead/behind counts and status
gh api repos/{owner}/{repo}/compare/{base}...{head} \
  --jq '{ahead_by: .ahead_by, behind_by: .behind_by, total_commits: .total_commits, status: .status}'
```

**Example:**
```bash
gh api repos/azharmgh/metamcp_rust/compare/main...develop \
  --jq '{ahead_by: .ahead_by, behind_by: .behind_by, total_commits: .total_commits, status: .status}'
```

**Output:**
```json
{"ahead_by":1,"behind_by":3,"status":"diverged","total_commits":1}
```

**Status values:**
- `ahead` — head has commits not in base
- `behind` — base has commits not in head
- `diverged` — both branches have unique commits
- `identical` — branches point to the same commit

### List changed files between branches

```bash
# Files that differ between two branches
gh api repos/{owner}/{repo}/compare/{base}...{head} \
  --jq '.files[] | "\(.status)\t\(.filename)\t+\(.additions)/-\(.deletions)"'
```

**Example:**
```bash
gh api repos/azharmgh/metamcp_rust/compare/main...develop \
  --jq '.files[] | "\(.status)\t\(.filename)\t+\(.additions)/-\(.deletions)"'
```

**Output:**
```
modified    .env.example    +9/-0
modified    Cargo.toml      +1/-1
modified    README.md       +32/-18
...
```

### List commits between branches

```bash
# Commits in head that are not in base
gh api repos/{owner}/{repo}/compare/{base}...{head} \
  --jq '.commits[] | "\(.sha[:7]) \(.commit.message)"'
```

**Example:**
```bash
# What commits does main have that develop doesn't?
gh api repos/azharmgh/metamcp_rust/compare/develop...main \
  --jq '.commits[] | "\(.sha[:7]) \(.commit.message)"'
```

---

## Pull Requests

### View a PR

```bash
# Quick summary
gh pr view {number}

# Get specific fields as JSON
gh pr view {number} --json title,body,files

# View PR in browser
gh pr view {number} --web
```

**Example:**
```bash
gh pr view 1 --json title,body,files --repo azharmgh/metamcp_rust
```

### Create a PR

```bash
gh pr create --title "Your title" --body "$(cat <<'EOF'
## Summary
- Change 1
- Change 2

## Test plan
- [ ] Verify X
- [ ] Check Y
EOF
)"
```

**Options:**
```bash
--base main          # Target branch (default: repo default branch)
--head develop       # Source branch (default: current branch)
--draft              # Create as draft PR
--reviewer user1     # Request review
--label "bug"        # Add labels
```

**Example:**
```bash
gh pr create --base main --head develop --title "Fix CI failures and add auth, config improvements" --body "$(cat <<'EOF'
   ## Summary
   - Fix all CI workflow failures: formatting (`cargo fmt`), clippy warnings, and a broken test assertion in `test_cloud_metadata_blocked`
   - Add direct API key authentication, local URL configuration, and trailing slash support
   - Add GitHub Actions CI workflow with format, clippy, build, and test jobs
   - Add GitHub workflow and gh CLI learning guide

   ## Test plan
   - [ ] Verify CI pipeline passes (format, clippy, build, test jobs)
   - [ ] Verify API key auth works with direct header and JWT flows
   - [ ] Verify trailing slash handling on API routes

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
```

### List PRs

```bash
# Open PRs
gh pr list

# All PRs (including closed/merged)
gh pr list --state all

# PRs by author
gh pr list --author @me
```

### Merge a PR

```bash
gh pr merge {number}

# With specific strategy
gh pr merge {number} --merge    # merge commit
gh pr merge {number} --squash   # squash and merge
gh pr merge {number} --rebase   # rebase and merge
```

### View PR comments

```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments
```

---

## Repository Information

### View repo details

```bash
gh repo view
gh repo view {owner}/{repo}
```

### View workflow runs

```bash
# List recent workflow runs
gh run list

# View a specific run
gh run view {run_id}

# Watch a running workflow
gh run watch {run_id}
```

### View workflow run logs

```bash
# Download logs
gh run view {run_id} --log

# View failed steps only
gh run view {run_id} --log-failed
```

---

## Issues

### Create an issue

```bash
gh issue create --title "Bug: something broken" --body "Description here"
```

### List issues

```bash
gh issue list
gh issue list --label "bug"
gh issue list --assignee @me
```

---

## The `gh api` Command

The `gh api` command is the most powerful — it gives direct access to the GitHub REST API with automatic authentication.

### Syntax

```bash
gh api {endpoint} [flags]
```

### Useful flags

| Flag | Purpose | Example |
|------|---------|---------|
| `--jq` | Filter JSON output with jq expressions | `--jq '.name'` |
| `-X` | HTTP method | `-X POST` |
| `-f` | Add a string field | `-f title="My PR"` |
| `--paginate` | Auto-paginate results | Useful for large lists |

### Examples

```bash
# Get repo info
gh api repos/{owner}/{repo}

# List branches
gh api repos/{owner}/{repo}/branches --jq '.[].name'

# Get latest release
gh api repos/{owner}/{repo}/releases/latest --jq '.tag_name'

# Compare branches (used extensively in this project)
gh api repos/{owner}/{repo}/compare/main...develop

# Get PR review comments
gh api repos/{owner}/{repo}/pulls/{number}/comments
```

### jq Expressions Cheat Sheet

```bash
# Single field
--jq '.name'

# Multiple fields
--jq '{name: .name, stars: .stargazers_count}'

# Array iteration
--jq '.[] | .name'

# Filter arrays
--jq '.files[] | select(.status == "modified")'

# String interpolation
--jq '.files[] | "\(.filename): +\(.additions)/-\(.deletions)"'

# First 7 chars of a string
--jq '.sha[:7]'
```

---

## Common Workflows

### Check if your branch is up to date before creating a PR

```bash
# 1. Compare your branch with main
gh api repos/{owner}/{repo}/compare/main...develop \
  --jq '{ahead_by: .ahead_by, behind_by: .behind_by, status: .status}'

# 2. If behind, merge main locally
git fetch origin main && git merge origin/main

# 3. Push and create PR
git push origin develop
gh pr create --title "Feature: my changes" --base main
```

### Review what a PR changes

```bash
# 1. Get PR details
gh pr view 1 --json title,body,files

# 2. See the diff
gh pr diff 1

# 3. Check CI status
gh pr checks 1
```

### Investigate CI failures

```bash
# 1. List recent runs
gh run list --branch main

# 2. View the failed run
gh run view {run_id} --log-failed

# 3. Re-run failed jobs
gh run rerun {run_id} --failed
```


# Miscellaneous #

```bash
 git fetch origin main && git merge origin/main --no-commit 2>&1  
 
```