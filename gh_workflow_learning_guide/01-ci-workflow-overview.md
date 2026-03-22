# CI Workflow Overview

## What is GitHub Actions?

GitHub Actions is a CI/CD platform built into GitHub. You define workflows as YAML files in `.github/workflows/`, and GitHub runs them automatically based on triggers (push, PR, schedule, etc.).

## Our CI Pipeline

The file `.github/workflows/ci.yml` defines a CI pipeline with **4 parallel jobs**:

```
Push/PR to main
        │
        ├── fmt     (Format check)
        ├── clippy  (Linter)
        ├── build   (Release build)
        └── test    (Unit + doc tests)
```

All 4 jobs run **independently and in parallel** — if one fails, the others still complete. This gives fast feedback on exactly what's broken.

## Job Summary

| Job | Purpose | Needs PostgreSQL? | Time |
|-----|---------|-------------------|------|
| `fmt` | Checks code formatting with `rustfmt` | No | ~30s |
| `clippy` | Runs Rust linter for warnings/errors | Yes (SQLx compile-time checks) | ~3-5m |
| `build` | Compiles a release build | Yes (SQLx compile-time checks) | ~5-8m |
| `test` | Runs all tests + doc tests | Yes (tests need a database) | ~3-5m |

## Why These 4 Jobs?

- **fmt**: Enforces consistent code style across all contributors
- **clippy**: Catches common mistakes, anti-patterns, and potential bugs at compile time
- **build**: Ensures the project compiles in release mode (catches optimization-specific issues)
- **test**: Validates correctness — unit tests, integration tests, and documentation examples
