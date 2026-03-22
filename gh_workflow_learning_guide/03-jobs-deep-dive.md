# Jobs Deep Dive

## Job 1: Format (`fmt`)

```yaml
fmt:
  name: Format
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Check formatting
      run: cargo fmt --all -- --check
```

### What it does

1. Checks out the code
2. Installs the stable Rust toolchain with the `rustfmt` component
3. Runs `cargo fmt --all -- --check` which:
   - `--all` — checks all workspace members
   - `--check` — fails if any file is not formatted (doesn't modify files)

### Why no PostgreSQL?

Format checking only parses Rust syntax — it doesn't compile anything, so SQLx compile-time checks aren't triggered.

### Fix locally

```bash
cargo fmt --all
```

---

## Job 2: Clippy (`clippy`)

```yaml
clippy:
  name: Clippy
  runs-on: ubuntu-latest
  services:
    postgres: ...
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - uses: Swatinem/rust-cache@v2
    - name: Install SQLx CLI
      run: cargo install sqlx-cli --no-default-features --features rustls,postgres
    - name: Run migrations
      run: sqlx migrate run
    - name: Run Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
```

### What it does

1. Starts a PostgreSQL service container (needed for SQLx)
2. Installs Rust with `clippy`
3. Caches Cargo dependencies (via `Swatinem/rust-cache@v2`) to speed up builds
4. Installs `sqlx-cli` and runs database migrations
5. Runs Clippy with:
   - `--all-targets` — checks lib, bins, tests, examples, benches
   - `--all-features` — enables all Cargo features
   - `-D warnings` — treats warnings as errors (strict mode)

### Why `-D warnings`?

Without this flag, Clippy warnings won't fail CI. The `-D` (deny) flag ensures all warnings must be fixed before merging.

### Fix locally

```bash
cargo clippy --all-targets --all-features -- -D warnings
# Auto-fix what Clippy can:
cargo clippy --fix --allow-dirty
```

---

## Job 3: Build (`build`)

```yaml
build:
  name: Build
  runs-on: ubuntu-latest
  services:
    postgres: ...
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Install SQLx CLI
      run: cargo install sqlx-cli --no-default-features --features rustls,postgres
    - name: Run migrations
      run: sqlx migrate run
    - name: Build
      run: cargo build --release --all-targets
```

### What it does

1. Same PostgreSQL + migration setup as Clippy
2. Builds in `--release` mode with all targets

### Why release mode?

- Debug builds may succeed where release builds fail (e.g., integer overflow checks differ)
- Validates that the production binary compiles cleanly
- `--all-targets` ensures examples, tests, and benchmarks also compile

---

## Job 4: Test (`test`)

```yaml
test:
  name: Test
  runs-on: ubuntu-latest
  services:
    postgres: ...
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Install SQLx CLI
      run: cargo install sqlx-cli --no-default-features --features rustls,postgres
    - name: Run migrations
      run: sqlx migrate run
    - name: Run tests
      run: cargo test --all-features
    - name: Run doc tests
      run: cargo test --doc
```

### What it does

1. Same PostgreSQL + migration setup
2. Runs all tests with `cargo test --all-features`
3. Separately runs doc tests with `cargo test --doc`

### Why separate doc tests?

Doc tests (code examples in `///` comments) are compiled and run differently from regular tests. Running them separately ensures documentation examples stay valid.

---

## Common Patterns Across Jobs

### `actions/checkout@v4`
Every job starts by cloning the repository. Each job runs in a **fresh VM** — they don't share filesystems.

### `dtolnay/rust-toolchain@stable`
The de facto standard action for installing Rust. Maintained by David Tolnay (a prominent Rust community member).

### `Swatinem/rust-cache@v2`
Caches `~/.cargo` and `target/` directories between runs. Dramatically speeds up builds (from ~10min to ~3min). Not used in `fmt` because formatting doesn't need compiled dependencies.
