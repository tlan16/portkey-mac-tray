// Library module for my-mac-tray
// This allows tests to run without including main.rs GUI initialization

pub mod lib_portkey;
pub mod app_config;
pub mod lib_github;

pub use app_config::APP_CONFIG;
