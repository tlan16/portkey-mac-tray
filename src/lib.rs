// Library module for portkey-mac-tray
// This allows tests to run without including main.rs GUI initialization

pub mod lib_portkey;
pub mod app_config;

pub use app_config::{APP_CONFIG, get_api_key, set_api_key, delete_api_key, has_api_key};
