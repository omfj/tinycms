use clap::{Parser, Subcommand};
use colored::Colorize;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

use crate::config::Config;
use crate::schema::{
    AuthConfig, DatabaseConfig, FieldDef, ProviderConfig, StorageConfig, TinyCmsConfig, TypeDef,
};

#[derive(Parser)]
#[command(name = "tinycms", about = "Self-hosted headless CMS")]
struct Cli {
    /// Project directory (defaults to current directory)
    #[arg(short, long, global = true)]
    dir: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the CMS server
    Serve,
    /// Start the CMS server with live config reloading
    Dev,
    /// Display the parsed config from tinycms.config.ts
    Config {
        /// Show secret values instead of redacting them
        #[arg(long)]
        show_secrets: bool,
    },
    /// Check that all required dependencies are available
    Checkhealth,
}

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if let Some(dir) = &cli.dir {
        std::env::set_current_dir(dir)
            .map_err(|e| anyhow::anyhow!("cannot change to directory {dir:?}: {e}"))?;
    }

    let cfg = Config::from_env()?;

    match cli.command {
        Command::Serve | Command::Dev => {
            crate::serve(cfg).await?;
        }
        Command::Config { show_secrets } => {
            show_config(&cfg.config_path, show_secrets).await?;
        }
        Command::Checkhealth => {
            checkhealth(&cfg.config_path).await;
        }
    }

    Ok(())
}

async fn show_config(config_path: &str, show_secrets: bool) -> anyhow::Result<()> {
    let schema = crate::schema::load(config_path).await?;
    render_config(&schema, show_secrets);
    Ok(())
}

fn render_config(config: &TinyCmsConfig, show_secrets: bool) {
    println!("{}", "Config".bold());
    println!("  {}", "database".bold());
    render_database(&config.database, show_secrets, 4);
    println!();
    println!("  {}", "auth".bold());
    render_auth(config.auth.as_ref(), show_secrets, 4);
    println!();
    println!("  {}", "storage".bold());
    render_storage(config.storage.as_ref(), show_secrets, 4);
    println!();
    println!("  {}", "types".bold());
    render_types(&config.types, 4);
}

fn render_database(database: &DatabaseConfig, show_secrets: bool, indent: usize) {
    render_line(indent, "url", Some(&database.url), true, show_secrets);
}

fn render_auth(auth: Option<&AuthConfig>, show_secrets: bool, indent: usize) {
    match auth {
        Some(auth) if auth.providers.is_empty() => {
            println!("{space}  none", space = " ".repeat(indent));
        }
        Some(auth) => {
            println!("{space}  providers", space = " ".repeat(indent));
            for provider in &auth.providers {
                match provider {
                    ProviderConfig::Credentials => {
                        println!("{space}    - credentials", space = " ".repeat(indent));
                    }
                    ProviderConfig::GitHub {
                        client_id,
                        client_secret,
                    } => {
                        println!("{space}    - github", space = " ".repeat(indent));
                        render_optional_pair(
                            indent + 8,
                            "clientId",
                            client_id.as_deref(),
                            false,
                            show_secrets,
                        );
                        render_optional_pair(
                            indent + 8,
                            "clientSecret",
                            client_secret.as_deref(),
                            true,
                            show_secrets,
                        );
                    }
                    ProviderConfig::Google {
                        client_id,
                        client_secret,
                    } => {
                        println!("{space}    - google", space = " ".repeat(indent));
                        render_optional_pair(
                            indent + 8,
                            "clientId",
                            client_id.as_deref(),
                            false,
                            show_secrets,
                        );
                        render_optional_pair(
                            indent + 8,
                            "clientSecret",
                            client_secret.as_deref(),
                            true,
                            show_secrets,
                        );
                    }
                }
            }
        }
        None => println!("{space}  none", space = " ".repeat(indent)),
    }
}

fn render_storage(storage: Option<&StorageConfig>, show_secrets: bool, indent: usize) {
    match storage {
        Some(storage) => {
            render_line(indent, "bucket", Some(&storage.bucket), false, show_secrets);
            render_optional_line(
                indent,
                "region",
                storage.region.as_deref(),
                false,
                show_secrets,
            );
            render_optional_line(
                indent,
                "accessKeyId",
                storage.access_key_id.as_deref(),
                true,
                show_secrets,
            );
            render_optional_line(
                indent,
                "secretAccessKey",
                storage.secret_access_key.as_deref(),
                true,
                show_secrets,
            );
            render_optional_line(
                indent,
                "endpoint",
                storage.endpoint.as_deref(),
                false,
                show_secrets,
            );
        }
        None => println!("{space}  none", space = " ".repeat(indent)),
    }
}

fn render_types(types: &[TypeDef], indent: usize) {
    if types.is_empty() {
        println!("{space}  none", space = " ".repeat(indent));
        return;
    }

    for ty in types {
        println!(
            "{space}  - {name}",
            space = " ".repeat(indent),
            name = ty.name
        );
        if ty.fields.is_empty() {
            println!("{space}    fields: none", space = " ".repeat(indent));
            continue;
        }

        println!("{space}    fields", space = " ".repeat(indent));
        for field in &ty.fields {
            let base = field.base();
            println!(
                "{space}      - {name}",
                space = " ".repeat(indent),
                name = base.name
            );
            render_line(indent + 8, "type", Some(field.field_type()), false, true);
            render_line(
                indent + 8,
                "required",
                Some(if base.required { "true" } else { "false" }),
                false,
                true,
            );
            let to = if let FieldDef::Reference(f) = field {
                f.to.as_deref()
            } else {
                None
            };
            let source = if let FieldDef::Slug(f) = field {
                f.source.as_deref()
            } else {
                None
            };
            render_optional_list(indent + 8, "to", to);
            render_optional_line(indent + 8, "source", source, false, true);
        }
    }
}

fn render_optional_pair(
    indent: usize,
    key: &str,
    value: Option<&str>,
    secret: bool,
    show_secrets: bool,
) {
    match value {
        Some(value) => render_line(indent, key, Some(value), secret, show_secrets),
        None => render_line(indent, key, None, secret, show_secrets),
    }
}

fn render_optional_line(
    indent: usize,
    key: &str,
    value: Option<&str>,
    secret: bool,
    show_secrets: bool,
) {
    render_optional_pair(indent, key, value, secret, show_secrets);
}

fn render_optional_list(indent: usize, key: &str, value: Option<&[String]>) {
    match value {
        Some(items) if !items.is_empty() => {
            println!("{space}{key}:", space = " ".repeat(indent));
            for item in items {
                println!("{space}- {item}", space = " ".repeat(indent + 2));
            }
        }
        Some(_) => println!("{space}{key}: none", space = " ".repeat(indent)),
        None => {}
    }
}

fn render_line(indent: usize, key: &str, value: Option<&str>, secret: bool, show_secrets: bool) {
    let rendered = match value {
        Some(_) if secret && !show_secrets => "[redacted]",
        Some(value) => value,
        None => "none",
    };
    println!("{space}{key}: {rendered}", space = " ".repeat(indent));
}

async fn checkhealth(config_path: &str) {
    let runtime_pref = crate::schema::sniff_runtime_pref(config_path);
    let runtime_header = match &runtime_pref {
        Some(name) => format!("JS runtime (preferred: {name})"),
        None => "JS runtime (first match is used)".to_string(),
    };
    println!("{}", runtime_header.bold());
    check("node", &["--version"]).await;
    check("deno", &["-v"]).await;
    check("bun", &["--version"]).await;

    println!("\n{}", "Tools:".bold());
    check("sqlx", &["--version"]).await;

    let schema = match crate::schema::load(config_path).await {
        Ok(s) => Some(s),
        Err(e) => {
            println!("\n  {} could not load config: {e}", missing());
            None
        }
    };

    println!("\n{}", "Database:".bold());
    match schema.as_ref() {
        Some(s) => check_database(&s.database.url).await,
        None => println!("  {} skipped — config unavailable", skip()),
    }

    println!("\n{}", "Storage:".bold());
    match schema.as_ref().and_then(|s| s.storage.as_ref()) {
        Some(s) => {
            let endpoint = s.endpoint.as_deref().unwrap_or("AWS S3");
            println!("  {} bucket={} endpoint={}", ok(), s.bucket, endpoint);
        }
        None => println!("  {} storage not configured — uploads disabled", skip()),
    }
}

async fn check_database(database_url: &str) {
    let sanitized = sanitize_database_url(database_url);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await;

    match pool {
        Ok(pool) => match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&pool)
            .await
        {
            Ok(1) => println!("  {} connected {}", ok(), sanitized.dimmed()),
            Ok(_) => println!("  {} connected, unexpected ping response", warn()),
            Err(e) => println!("  {} query failed: {}", missing(), e),
        },
        Err(e) => println!(
            "  {} cannot connect {}: {}",
            missing(),
            sanitized.dimmed(),
            e
        ),
    }
}

fn sanitize_database_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return "<invalid DATABASE_URL>".into();
    };
    let authority_start = scheme_end + 3;
    let authority_end = url[authority_start..]
        .find('/')
        .map(|index| authority_start + index)
        .unwrap_or(url.len());
    let authority = &url[authority_start..authority_end];

    if let Some(at_index) = authority.rfind('@') {
        let credentials = &authority[..at_index];
        if let Some(colon_index) = credentials.find(':') {
            let password_start = authority_start + colon_index + 1;
            let password_end = authority_start + at_index;
            return format!("{}****{}", &url[..password_start], &url[password_end..]);
        }
    }

    url.to_string()
}

async fn check(bin: &str, args: &[&str]) {
    match tokio::process::Command::new(bin).args(args).output().await {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let version = parse_version(&raw);
            println!("  {} {} v{}", ok(), bin, version);
        }
        Ok(_) => println!("  {} {} exited with error", warn(), bin),
        Err(_) => println!("  {} {} not found in PATH", missing(), bin),
    }
}

fn parse_version(output: &str) -> &str {
    for word in output.split_whitespace() {
        let s = word.strip_prefix('v').unwrap_or(word);
        if s.starts_with(|c: char| c.is_ascii_digit()) {
            return s;
        }
    }
    output.trim()
}

fn ok() -> String {
    "[ok]".green().bold().to_string()
}
fn warn() -> String {
    "[warn]".yellow().bold().to_string()
}
fn missing() -> String {
    "[missing]".red().bold().to_string()
}
fn skip() -> String {
    "[skip]".dimmed().to_string()
}
