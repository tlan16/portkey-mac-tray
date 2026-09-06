use keyring::Entry;
use std::sync::LazyLock;

const SERVICE_NAME: &str = "my-mac-tray";
const KEY_NAME: &str = "portkey-api-key";

// ---------------------------------------------------------
// CONFIGURATION
// ---------------------------------------------------------
pub struct AppConfig {
    pub portkey_api_key: String,
}

/// Retrieve the Portkey API key from macOS Keychain.
pub fn get_api_key() -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;
    
    entry.get_password()
        .map_err(|e| format!("API key not found in keychain: {}", e))
}

/// Store the Portkey API key in macOS Keychain.
pub fn set_api_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("API key must not be empty".to_string());
    }
    
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;
    
    entry.set_password(key)
        .map_err(|e| format!("Failed to store API key: {}", e))
}

/// Delete the Portkey API key from macOS Keychain.
pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access keychain: {}", e))?;
    
    entry.delete_credential()
        .map_err(|e| format!("Failed to delete API key: {}", e))
}

/// Check if an API key exists in the keychain.
pub fn has_api_key() -> bool {
    get_api_key().is_ok()
}

// LazyLock ensures the config is initialised
// EXACTLY ONCE the first time `APP_CONFIG` is accessed.
pub static APP_CONFIG: LazyLock<Result<AppConfig, String>> = LazyLock::new(|| {
    let portkey_api_key = get_api_key()?;
    Ok(AppConfig { portkey_api_key })
});
