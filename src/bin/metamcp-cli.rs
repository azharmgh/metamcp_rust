//! MetaMCP CLI - API Key Management Tool

use anyhow::Result;
use clap::{Parser, Subcommand};
use metamcp::{AuthService, Config, Database};
use std::sync::Arc;

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
        /// Show inactive keys
        #[arg(long)]
        include_inactive: bool,
    },

    /// Create a new API key
    Create {
        /// Name for the API key
        #[arg(short, long)]
        name: String,
    },

    /// Show API key details
    Show {
        /// API key ID
        key_id: String,
    },

    /// Inactivate an API key
    Inactivate {
        /// API key ID
        key_id: String,
    },

    /// Activate an inactive API key
    Activate {
        /// API key ID
        key_id: String,
    },

    /// Delete an API key permanently
    Delete {
        /// API key ID
        key_id: String,

        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },

    /// Rotate an API key (create new, inactivate old)
    Rotate {
        /// API key ID to rotate
        key_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for CLI
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    // Load configuration
    let config = Config::from_env()?;

    // Connect to database
    let db = Database::new(&config.database_url).await?;

    // Initialize auth service
    let auth_service = Arc::new(AuthService::new(
        config.jwt_secret,
        &config.encryption_key,
        db.clone(),
    ));

    match cli.command {
        Commands::Keys { action } => handle_key_commands(action, &db, &auth_service).await?,
    }

    Ok(())
}

async fn handle_key_commands(
    action: KeyActions,
    db: &Database,
    auth: &Arc<AuthService>,
) -> Result<()> {
    match action {
        KeyActions::List { include_inactive } => {
            let keys = db.api_keys().list_all(include_inactive).await?;

            if keys.is_empty() {
                println!("\nNo API keys found.\n");
                return Ok(());
            }

            println!(
                "\n{:<36} {:<30} {:<10} {:<20}",
                "ID", "Name", "Status", "Created"
            );
            println!("{}", "-".repeat(100));

            for key in keys {
                let status = if key.is_active { "Active" } else { "Inactive" };
                println!(
                    "{:<36} {:<30} {:<10} {}",
                    key.id,
                    truncate_string(&key.name, 28),
                    status,
                    key.created_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
            println!();
        }

        KeyActions::Create { name } => {
            let (api_key, stored_key) = auth.generate_api_key(name.clone()).await?;

            println!("\n✓ API Key created successfully!");
            println!("\nKey ID: {}", stored_key.id);
            println!("Name: {}", stored_key.name);
            println!("\n╔════════════════════════════════════════════════════════════════╗");
            println!("║  IMPORTANT: Save this API key now. It won't be shown again!    ║");
            println!("╚════════════════════════════════════════════════════════════════╝");
            println!("\nAPI Key: {}\n", api_key);
        }

        KeyActions::Show { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            let key = db
                .api_keys()
                .find_by_id(key_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            println!("\nAPI Key Details:");
            println!("────────────────────────────────────");
            println!("ID:        {}", key.id);
            println!("Name:      {}", key.name);
            println!(
                "Status:    {}",
                if key.is_active { "Active" } else { "Inactive" }
            );
            println!(
                "Created:   {}",
                key.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            if let Some(last_used) = key.last_used_at {
                println!("Last Used: {}", last_used.format("%Y-%m-%d %H:%M:%S UTC"));
            } else {
                println!("Last Used: Never");
            }
            println!();
        }

        KeyActions::Inactivate { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;

            // Verify key exists
            db.api_keys()
                .find_by_id(key_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            db.api_keys().set_inactive(key_uuid).await?;
            println!("\n✓ API key inactivated successfully\n");
        }

        KeyActions::Activate { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;

            // Verify key exists
            db.api_keys()
                .find_by_id(key_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            db.api_keys().set_active(key_uuid).await?;
            println!("\n✓ API key activated successfully\n");
        }

        KeyActions::Delete { key_id, confirm } => {
            if !confirm {
                eprintln!("\n✗ Error: Must use --confirm flag to delete an API key");
                eprintln!("  This action is permanent and cannot be undone.\n");
                std::process::exit(1);
            }

            let key_uuid = uuid::Uuid::parse_str(&key_id)?;

            // Verify key exists
            db.api_keys()
                .find_by_id(key_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            db.api_keys().delete(key_uuid).await?;
            println!("\n✓ API key deleted permanently\n");
        }

        KeyActions::Rotate { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            let old_key = db
                .api_keys()
                .find_by_id(key_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            // Create new key with same name (appended with rotation date)
            let new_name = format!(
                "{} (rotated {})",
                old_key.name,
                chrono::Utc::now().format("%Y-%m-%d")
            );
            let (new_api_key, new_stored_key) = auth.generate_api_key(new_name).await?;

            // Inactivate old key
            db.api_keys().set_inactive(key_uuid).await?;

            println!("\n✓ API key rotated successfully!");
            println!("\nOld Key ID: {} (now inactive)", old_key.id);
            println!("New Key ID: {}", new_stored_key.id);
            println!("\n╔════════════════════════════════════════════════════════════════╗");
            println!("║  IMPORTANT: Save this new API key now!                         ║");
            println!("╚════════════════════════════════════════════════════════════════╝");
            println!("\nNew API Key: {}\n", new_api_key);
        }
    }

    Ok(())
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
