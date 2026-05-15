mod config;
mod field;

use anyhow::Context;

pub use config::{AuthConfig, DatabaseConfig, ProviderConfig, StorageConfig, TinyCmsConfig};
pub use field::{FieldDef, FieldError, TypeDef};

/// JS runtimes tried in order. First one found in PATH wins.
///
/// If runtime is specified in the config it will use that one.
static RUNTIMES: &[Runtime] = &[
    Runtime {
        bin: "node",
        args: &["--input-type=module"],
    },
    Runtime {
        bin: "deno",
        args: &["run", "--allow-read", "--allow-env", "-"],
    },
    Runtime {
        bin: "bun",
        args: &["run", "-"],
    },
];

struct Runtime {
    bin: &'static str,
    args: &'static [&'static str],
}

pub async fn load(config_path: &str) -> anyhow::Result<TinyCmsConfig> {
    let abs = std::fs::canonicalize(config_path)
        .with_context(|| format!("config file not found: {config_path}"))?;

    let script = format!(
        "import c from {path:?}; process.stdout.write(JSON.stringify(c.default ?? c));\n",
        path = abs.display()
    );

    let preferred = sniff_runtime_pref(config_path);
    let runtime = find_runtime(preferred.as_deref())
        .await
        .context("no JS runtime found — install Node, Deno, or Bun")?;

    let mut child = tokio::process::Command::new(runtime.bin)
        .args(runtime.args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn JS runtime")?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(script.as_bytes()).await?;
    }

    let output = child.wait_with_output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("config evaluation failed ({}):\n{stderr}", runtime.bin);
    }

    serde_json::from_slice(&output.stdout)
        .context("config did not produce valid JSON — ensure your config file has a default export")
}

// This looks for the runtime, before we parse it with a JS runtime.
// This is because this decides which runtime to use to parse the config.
// It does not make sense to pick on to later change it.
pub(crate) fn sniff_runtime_pref(config_path: &str) -> Option<String> {
    let text = std::fs::read_to_string(config_path).ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix("runtime")?.trim_start();
        let rest = rest.strip_prefix(':')?.trim_start();
        let rest = rest.trim_start_matches(['"', '\'']);
        let end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '-')
            .unwrap_or(rest.len());
        let name = &rest[..end];
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

async fn find_runtime(preferred: Option<&str>) -> Option<&'static Runtime> {
    if let Some(name) = preferred {
        if let Some(rt) = RUNTIMES.iter().find(|r| r.bin == name) {
            if which(rt.bin).await {
                tracing::debug!("using JS runtime: {} (from config)", rt.bin);
                return Some(rt);
            }
            tracing::warn!("preferred runtime '{name}' not found in PATH, falling back");
        } else {
            tracing::warn!("unknown runtime '{name}' in config, falling back to auto-detect");
        }
    }
    for runtime in RUNTIMES {
        if which(runtime.bin).await {
            tracing::debug!("using JS runtime: {}", runtime.bin);
            return Some(runtime);
        }
    }
    None
}

async fn which(bin: &str) -> bool {
    tokio::process::Command::new("which")
        .arg(bin)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
