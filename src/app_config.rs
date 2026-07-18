use std::sync::LazyLock;
use obfstr::obfstr;

// ---------------------------------------------------------
// CONFIGURATION
// ---------------------------------------------------------
pub struct AppConfig {
    pub portkey_api_key: String,
}

// LazyLock ensures the obfuscated string is decrypted and allocated
// EXACTLY ONCE the first time `APP_CONFIG` is accessed.
pub static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    AppConfig {
        // obfstr! macro encrypts the string at compile time.
        // It is decrypted here at runtime into a standard String.
        portkey_api_key: obfstr!("REDACTED_API_KEY").to_string(),
    }
});
