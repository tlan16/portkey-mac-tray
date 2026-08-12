use std::sync::LazyLock;

// ---------------------------------------------------------
// CONFIGURATION
// ---------------------------------------------------------
pub struct AppConfig {
    pub portkey_api_key: String,
}

/// Resolve the Portkey API key from `PORTKEY_API_KEY` env var
/// (also loaded from `.env` if present).
fn resolve_portkey_api_key() -> Result<String, String> {
    dotenvy::dotenv().ok();

    let key = std::env::var("PORTKEY_API_KEY")
        .map_err(|_| "PORTKEY_API_KEY must be set (via env var or .env file)".to_string())?;

    if key.is_empty() {
        return Err("PORTKEY_API_KEY must not be empty".to_string());
    }

    Ok(key)
}

// LazyLock ensures the config is initialised
// EXACTLY ONCE the first time `APP_CONFIG` is accessed.
pub static APP_CONFIG: LazyLock<Result<AppConfig, String>> = LazyLock::new(|| {
    let portkey_api_key = resolve_portkey_api_key()?;
    Ok(AppConfig { portkey_api_key })
});
