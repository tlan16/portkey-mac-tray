use std::sync::LazyLock;
use obfstr::obfstr;

// ---------------------------------------------------------
// CONFIGURATION
// ---------------------------------------------------------
pub struct AppConfig {
    pub portkey_api_key: String,
}

/// Resolve the Portkey API key with the following precedence:
/// 1. `PORTKEY_API_KEY` environment variable
/// 2. `.env` file in the project root (loaded via dotenvy)
/// 3. Compile-time obfuscated fallback
fn resolve_portkey_api_key() -> String {
    // Load .env from CWD (or nearest parent). Does nothing if no .env exists.
    dotenvy::dotenv().ok();

    // Check env var (covers both pre-existing env vars and .env-loaded values).
    // dotenvy never overwrites already-set env vars, so real env vars win automatically.
    std::env::var("PORTKEY_API_KEY").unwrap_or_else(|_| {
        obfstr!("REDACTED_API_KEY").to_string()
    })
}

// LazyLock ensures the config is initialised
// EXACTLY ONCE the first time `APP_CONFIG` is accessed.
pub static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    AppConfig {
        portkey_api_key: resolve_portkey_api_key(),
    }
});
