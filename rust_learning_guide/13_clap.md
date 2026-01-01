# Clap - Command Line Parsing

Clap is Rust's most popular command-line argument parsing library, providing a powerful and ergonomic way to build CLI applications.

## What is Clap?

Clap provides:
- **Argument parsing** - Handle flags, options, and positional arguments
- **Subcommands** - Build complex CLI tools like git or cargo
- **Derive macros** - Define CLI structure with Rust structs
- **Help generation** - Automatic `--help` and usage messages
- **Completion scripts** - Shell completion for bash, zsh, fish

## Why Clap?

Clap is the standard for Rust CLIs because:
- Excellent derive macro support
- Comprehensive feature set
- Great documentation
- Active maintenance

## Installation

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

## Basic Usage

### Simple CLI with Derive

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "myapp")]
#[command(about = "A simple CLI application", long_about = None)]
#[command(version)]
struct Cli {
    /// The name to greet
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let cli = Cli::parse();

    for _ in 0..cli.count {
        println!("Hello, {}!", cli.name);
    }
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20clap%3A%3AParser%3B%0A%0A%23%5Bderive(Parser)%5D%0A%23%5Bcommand(name%20%3D%20%22myapp%22)%5D%0A%23%5Bcommand(about%20%3D%20%22A%20simple%20CLI%22)%5D%0Astruct%20Cli%20%7B%0A%20%20%20%20%2F%2F%2F%20The%20name%20to%20greet%0A%20%20%20%20name%3A%20String%2C%0A%0A%20%20%20%20%2F%2F%2F%20Number%20of%20times%0A%20%20%20%20%23%5Barg(short%2C%20long%2C%20default_value_t%20%3D%201)%5D%0A%20%20%20%20count%3A%20u8%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Simulate%20parsing%20%22myapp%20World%20-c%203%22%0A%20%20%20%20let%20args%20%3D%20vec!%5B%22myapp%22%2C%20%22World%22%2C%20%22-c%22%2C%20%223%22%5D%3B%0A%20%20%20%20%0A%20%20%20%20match%20Cli%3A%3Atry_parse_from(args)%20%7B%0A%20%20%20%20%20%20%20%20Ok(cli)%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20for%20_%20in%200..cli.count%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Hello%2C%20%7B%7D!%22%2C%20cli.name)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20eprintln!(%22Error%3A%20%7B%7D%22%2C%20e)%2C%0A%20%20%20%20%7D%0A%7D)

### Generated Help

Running `myapp --help`:
```
A simple CLI application

Usage: myapp [OPTIONS] <NAME>

Arguments:
  <NAME>  The name to greet

Options:
  -c, --count <COUNT>  Number of times to greet [default: 1]
  -h, --help           Print help
  -V, --version        Print version
```

## Real Example from MetaMCP

From `src/bin/metamcp-cli.rs`:

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "metamcp-cli")]
#[command(about = "MetaMCP CLI for API key management", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage API keys
    Keys {
        #[command(subcommand)]
        action: KeyActions,
    },
}

#[derive(Subcommand)]
enum KeyActions {
    /// List all API keys
    List {
        /// Include inactive keys
        #[arg(long)]
        include_inactive: bool,
    },

    /// Create a new API key
    Create {
        /// Name for the API key
        #[arg(short, long)]
        name: String,
    },

    /// Show details of an API key
    Show {
        /// The API key ID
        key_id: String,
    },

    /// Deactivate an API key
    Inactivate {
        /// The API key ID
        key_id: String,
    },

    /// Reactivate an API key
    Activate {
        /// The API key ID
        key_id: String,
    },

    /// Delete an API key permanently
    Delete {
        /// The API key ID
        key_id: String,

        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },

    /// Rotate an API key (generate new, deactivate old)
    Rotate {
        /// The API key ID to rotate
        key_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::from_env()?;
    let db = Database::new(&config.database_url).await?;
    let auth_service = AuthService::new(
        config.jwt_secret.clone(),
        &config.encryption_key,
        db.clone(),
    );

    match cli.command {
        Commands::Keys { action } => {
            handle_key_commands(action, &db, &auth_service).await?;
        }
    }

    Ok(())
}

async fn handle_key_commands(
    action: KeyActions,
    db: &Database,
    auth_service: &AuthService,
) -> Result<()> {
    match action {
        KeyActions::List { include_inactive } => {
            let keys = db.api_keys().list_all(include_inactive).await?;
            println!("API Keys:");
            for key in keys {
                println!(
                    "  {} - {} (active: {})",
                    key.id, key.name, key.is_active
                );
            }
        }

        KeyActions::Create { name } => {
            let (raw_key, stored_key) = auth_service
                .generate_api_key(name.clone())
                .await?;

            println!("API Key created successfully!");
            println!("ID: {}", stored_key.id);
            println!("Name: {}", name);
            println!("Key: {}", raw_key);
            println!("\nIMPORTANT: Save this key now. You won't be able to see it again!");
        }

        KeyActions::Show { key_id } => {
            let id = Uuid::parse_str(&key_id)?;
            if let Some(key) = db.api_keys().find_by_id(id).await? {
                println!("ID: {}", key.id);
                println!("Name: {}", key.name);
                println!("Active: {}", key.is_active);
                println!("Created: {}", key.created_at);
            } else {
                println!("API key not found");
            }
        }

        KeyActions::Delete { key_id, confirm } => {
            if !confirm {
                println!("Use --confirm to delete permanently");
                return Ok(());
            }
            let id = Uuid::parse_str(&key_id)?;
            db.api_keys().delete(id).await?;
            println!("API key deleted");
        }

        // ... other actions
    }

    Ok(())
}
```

## Argument Types

### Positional Arguments

```rust
#[derive(Parser)]
struct Cli {
    /// Input file
    input: String,

    /// Output file (optional)
    output: Option<String>,

    /// Multiple files
    files: Vec<String>,
}
```

### Flags (Boolean Options)

```rust
#[derive(Parser)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Force overwrite
    #[arg(short, long)]
    force: bool,
}
```

### Options with Values

```rust
#[derive(Parser)]
struct Cli {
    /// Output format
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Port number
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Config file (optional)
    #[arg(short, long)]
    config: Option<String>,

    /// Multiple values
    #[arg(short, long)]
    tags: Vec<String>,
}
```

### Value Constraints

```rust
use clap::ValueEnum;

#[derive(Clone, ValueEnum)]
enum Format {
    Json,
    Yaml,
    Toml,
}

#[derive(Parser)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = Format::Json)]
    format: Format,

    /// Port (1024-65535)
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1024..65536))]
    port: u16,
}
```

[Run value enum example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20clap%3A%3A%7BParser%2C%20ValueEnum%7D%3B%0A%0A%23%5Bderive(Clone%2C%20ValueEnum%2C%20Debug)%5D%0Aenum%20Format%20%7B%0A%20%20%20%20Json%2C%0A%20%20%20%20Yaml%2C%0A%20%20%20%20Toml%2C%0A%7D%0A%0A%23%5Bderive(Parser%2C%20Debug)%5D%0Astruct%20Cli%20%7B%0A%20%20%20%20%23%5Barg(short%2C%20long%2C%20value_enum%2C%20default_value_t%20%3D%20Format%3A%3AJson)%5D%0A%20%20%20%20format%3A%20Format%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Default%0A%20%20%20%20let%20cli%20%3D%20Cli%3A%3Atry_parse_from(%5B%22app%22%5D).unwrap()%3B%0A%20%20%20%20println!(%22Default%3A%20%7B%3A%3F%7D%22%2C%20cli.format)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Yaml%0A%20%20%20%20let%20cli%20%3D%20Cli%3A%3Atry_parse_from(%5B%22app%22%2C%20%22-f%22%2C%20%22yaml%22%5D).unwrap()%3B%0A%20%20%20%20println!(%22With%20yaml%3A%20%7B%3A%3F%7D%22%2C%20cli.format)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Invalid%0A%20%20%20%20let%20result%20%3D%20Cli%3A%3Atry_parse_from(%5B%22app%22%2C%20%22-f%22%2C%20%22xml%22%5D)%3B%0A%20%20%20%20println!(%22Invalid%3A%20%7B%3A%3F%7D%22%2C%20result.is_err())%3B%0A%7D)

## Subcommands

### Nested Subcommands

```rust
#[derive(Parser)]
#[command(name = "cargo-like")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run tests
    Test {
        /// Test name filter
        filter: Option<String>,
    },

    /// Package management
    Package {
        #[command(subcommand)]
        action: PackageActions,
    },
}

#[derive(Subcommand)]
enum PackageActions {
    /// Add a dependency
    Add {
        /// Package name
        name: String,
    },

    /// Remove a dependency
    Remove {
        /// Package name
        name: String,
    },
}
```

### Using Subcommands

```rust
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { release } => {
            if release {
                println!("Building in release mode...");
            } else {
                println!("Building in debug mode...");
            }
        }
        Commands::Test { filter } => {
            match filter {
                Some(f) => println!("Running tests matching: {}", f),
                None => println!("Running all tests"),
            }
        }
        Commands::Package { action } => match action {
            PackageActions::Add { name } => {
                println!("Adding package: {}", name);
            }
            PackageActions::Remove { name } => {
                println!("Removing package: {}", name);
            }
        },
    }
}
```

## Advanced Features

### Environment Variables

```rust
#[derive(Parser)]
struct Cli {
    /// Database URL (or set DATABASE_URL)
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// API key (or set API_KEY)
    #[arg(long, env)]  // Uses --api-key or API_KEY
    api_key: Option<String>,
}
```

### Global Options

```rust
#[derive(Parser)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

// --verbose can be used before or after subcommand
// myapp --verbose build
// myapp build --verbose
```

### Validation

```rust
use std::path::PathBuf;

fn validate_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Path does not exist: {}", s))
    }
}

#[derive(Parser)]
struct Cli {
    /// Input file (must exist)
    #[arg(value_parser = validate_path)]
    input: PathBuf,
}
```

### Custom Help

```rust
#[derive(Parser)]
#[command(
    name = "myapp",
    author = "Your Name",
    version = "1.0",
    about = "Short description",
    long_about = "This is a longer description that appears in --help.\n\n\
                  It can span multiple lines.",
    after_help = "Examples:\n  myapp build\n  myapp test --release"
)]
struct Cli {
    // ...
}
```

## Common Patterns

### Config File Override

```rust
#[derive(Parser)]
struct Cli {
    /// Config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Override config values
    #[arg(long)]
    port: Option<u16>,
}

fn main() {
    let cli = Cli::parse();

    // Load config from file
    let mut config = load_config(&cli.config)?;

    // Override with CLI arguments
    if let Some(port) = cli.port {
        config.port = port;
    }
}
```

### Progress Flags

```rust
#[derive(Parser)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let cli = Cli::parse();

    match cli.verbose {
        0 => println!("Normal output"),
        1 => println!("Verbose output"),
        2 => println!("Very verbose output"),
        _ => println!("Debug output"),
    }
}
```

## Best Practices

### DO

1. **Use derive macros** - Clean and maintainable
2. **Add doc comments** - They become help text
3. **Provide defaults** - When sensible
4. **Use subcommands** - For complex CLIs
5. **Validate early** - Use value parsers

### DON'T

1. **Don't have too many flags** - Group into subcommands
2. **Don't require many arguments** - Use config files
3. **Don't forget `--help`** - It's automatic!
4. **Don't use positional for optional** - Use `--flag value`

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Ergonomic** | Derive macros are clean |
| **Powerful** | Handles complex CLIs |
| **Auto help** | Generates usage/help |
| **Validation** | Built-in value validation |
| **Completions** | Shell completion support |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Compile time** | Macros slow compilation |
| **Binary size** | Adds to final binary |
| **Learning curve** | Many features to learn |

## When to Use

**Use Clap when:**
- Building any CLI tool
- Need subcommands
- Want automatic help
- Need argument validation

**Consider alternatives when:**
- Very simple CLI (consider `std::env::args`)
- Minimal binary size needed (consider `lexopt`)

## Further Learning

### Official Resources
- [Clap Documentation](https://docs.rs/clap)
- [Clap GitHub](https://github.com/clap-rs/clap)
- [Clap Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)

### Practice
1. Build a file converter CLI
2. Create a task manager CLI
3. Implement a git-like interface
4. Add shell completions

## Related Crates

- **clap_complete** - Shell completion generation
- **clap_mangen** - Man page generation
- **indicatif** - Progress bars
- **dialoguer** - Interactive prompts
- **console** - Terminal styling
