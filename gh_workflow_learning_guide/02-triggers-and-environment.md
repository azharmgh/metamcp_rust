# Workflow Triggers and Environment

## Triggers (`on:`)

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

The workflow runs in two scenarios:

1. **Push to `main`** — After code is merged, CI validates the merged result
2. **Pull request targeting `main`** — CI runs on the PR branch *before* merging, acting as a gate

### What does NOT trigger CI

- Pushes to `develop` or feature branches (no CI until a PR is opened to `main`)
- Pushes to tags
- Manual triggers (no `workflow_dispatch`)

### Why this design?

Limiting CI to `main` saves runner minutes. Feature branches get validated only when they're ready for review (via PR). The `develop` branch is validated when a PR to `main` is created.

## Environment Variables (`env:`)

```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  DATABASE_URL: postgresql://metamcp:metamcp_ci_password@localhost:5432/metamcp_ci
  JWT_SECRET: ci-test-jwt-secret-not-for-production
  ENCRYPTION_KEY: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

These are set at the **workflow level**, meaning all jobs inherit them.

| Variable | Purpose |
|----------|---------|
| `CARGO_TERM_COLOR` | Forces colored output in Cargo (makes logs readable) |
| `RUST_BACKTRACE` | Shows full backtraces on panics (helps debug test failures) |
| `DATABASE_URL` | Connection string for SQLx compile-time query checking |
| `JWT_SECRET` | Test-only secret for JWT signing (not a real secret) |
| `ENCRYPTION_KEY` | Test-only key for API key encryption |

### Security Note

The `JWT_SECRET` and `ENCRYPTION_KEY` are **not real secrets** — they're dummy values used only in CI. Real secrets should be stored in GitHub Settings > Secrets and referenced as `${{ secrets.NAME }}`.

## Runner

All jobs use `runs-on: ubuntu-latest`, which provides:
- Ubuntu Linux (latest LTS)
- Pre-installed tools (git, curl, docker, etc.)
- 2 vCPUs, 7 GB RAM (standard GitHub-hosted runner)
