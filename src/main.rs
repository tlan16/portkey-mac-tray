use my_mac_tray::app_config::{self, APP_CONFIG};
use my_mac_tray::lib_portkey;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tray_icon::menu::{Menu, MenuItem, MenuEvent as TrayMenuEvent, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::WindowId;

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

// ---------------------------------------------------------
// 0. Verbose logging (global flag + helper macro)
// ---------------------------------------------------------
static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Prints only when `--verbose` is enabled. Behaves like `println!`.
macro_rules! vlog {
    ($($arg:tt)*) => {
        if VERBOSE.load(Ordering::Relaxed) {
            println!($($arg)*);
        }
    };
}

// ---------------------------------------------------------
// 1. Define custom events (Tokio -> Winit communication)
// ---------------------------------------------------------
#[derive(Debug)]
pub enum AppEvent {
    // We can add events here like API responses
    PortkeyDataReceived(String),
    MenuEvent(TrayMenuEvent),
    ApiKeyChanged,
}

// ---------------------------------------------------------
// APP STATE & EVENT LOOP
// ---------------------------------------------------------
struct MyApp {
    tray_icon: Option<TrayIcon>,
    has_api_key: bool,
}

// Notice we changed `ApplicationHandler` to `ApplicationHandler<AppEvent>`
impl ApplicationHandler<AppEvent> for MyApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.tray_icon.is_none() {
            let title = if self.has_api_key {
                match &*APP_CONFIG {
                    Ok(_cfg) => {
                        vlog!("Loaded config: API key present");
                        "Starting...".to_string()
                    }
                    Err(e) => {
                        eprintln!("Config error: {}", e);
                        format!("PK: Error: {e}")
                    }
                }
            } else {
                "Set Portkey API Key".to_string()
            };

            // Build initial tray menu
            let tray_menu = self.build_menu();

            let icon = TrayIconBuilder::new()
                .with_title(&title)
                .with_menu(Box::new(tray_menu))
                .build()
                .expect("Failed to build tray icon");

            self.tray_icon = Some(icon);
        }
    }

    // 2. This new function receives messages from our Tokio tasks!
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::PortkeyDataReceived(data) => {
                vlog!("UI Thread received async data: {}", data);
                if let Some(tray_icon) = &self.tray_icon {
                    tray_icon.set_title(Some(data));
                }
            }
            AppEvent::MenuEvent(event) => {
                vlog!("Menu event received: {:?}", event);
                match event.id().0.as_str() {
                    "quit" => event_loop.exit(),
                    "set-api-key" | "change-api-key" => {
                        // Prompt for API key and update state if successful
                        if prompt_for_api_key().is_some() {
                            self.has_api_key = app_config::has_api_key();
                            if let Some(tray_icon) = &self.tray_icon {
                                let tray_menu = self.build_menu();
                                tray_icon.set_menu(Some(Box::new(tray_menu)));
                                tray_icon.set_title(Some("Starting..."));
                            }
                            // Note: Restarting the background task would require more architecture changes
                            // For now, user needs to restart the app after setting the key
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::ApiKeyChanged => {
                // Reload config and restart background task
                self.has_api_key = app_config::has_api_key();
                if let Some(tray_icon) = &self.tray_icon {
                    let tray_menu = self.build_menu();
                    tray_icon.set_menu(Some(Box::new(tray_menu)));
                    tray_icon.set_title(Some("Starting..."));
                }
                // Note: Full background task restart would require Arc<Mutex> architecture
                // Current implementation requires app restart after setting API key
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_secs(1)));
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: WindowId,
        _event: WindowEvent,
    ) {}
}

impl MyApp {
    fn build_menu(&self) -> tray_icon::menu::Menu {
        let api_key_item = if self.has_api_key {
            MenuItem::with_id("change-api-key", "Change API Key", true, None)
        } else {
            MenuItem::with_id("set-api-key", "Set API Key", true, None)
        };
        
        let quit_item = MenuItem::with_id("quit", "Quit", true, None);
        
        Menu::with_items(&[
            &api_key_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .expect("Failed to build tray menu")
    }
}

fn prompt_for_api_key() -> Option<()> {
    // Since tray apps don't have focus, we use AppleScript to prompt
    // This opens Terminal.app with a secure input dialog
    let script = r#"
tell application "Terminal"
    activate
    set apiKey to display dialog "Enter Portkey API Key:" default answer "" with hidden answer
    set apiKey to text returned of apiKey
    return apiKey
end tell
"#;
    
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let key = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            
            if key.is_empty() {
                eprintln!("API key prompt cancelled or empty");
                return None;
            }
            
            match app_config::set_api_key(&key) {
                Ok(()) => {
                    vlog!("API key stored successfully");
                    Some(())
                }
                Err(e) => {
                    eprintln!("Failed to store API key: {}", e);
                    None
                }
            }
        }
        Ok(_) => {
            eprintln!("Failed to get API key from dialog");
            None
        }
        Err(e) => {
            eprintln!("Failed to run osascript: {}", e);
            None
        }
    }
}

// NO #[tokio::main] here! We keep it a standard sync main function.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Parse CLI args. `--verbose` (or `-v`) is optional and defaults to off.
    let verbose = std::env::args().any(|a| a == "--verbose" || a == "-v");
    VERBOSE.store(verbose, Ordering::Relaxed);
    vlog!("Verbose logging enabled");

    // 3. Build the event loop WITH our custom AppEvent type
    let mut builder = EventLoop::<AppEvent>::with_user_event();

    #[cfg(target_os = "macos")]
    builder.with_activation_policy(ActivationPolicy::Accessory);

    let event_loop = builder.build()?;

    // Create a proxy. This can be safely cloned and sent to other threads/async tasks!
    let proxy: EventLoopProxy<AppEvent> = event_loop.create_proxy();
    let menu_proxy = proxy.clone();

    // 4. Create the Tokio runtime manually
    let rt = tokio::runtime::Runtime::new()?;

    // 5. Only spawn background task if API key exists
    if app_config::has_api_key() {
        rt.spawn(async move {
            // Check if config loaded successfully; if not, show error once and stop.
            let api_key = match &*APP_CONFIG {
                Ok(cfg) => cfg.portkey_api_key.clone(),
                Err(e) => {
                    let msg = format!("PK: Config Error: {e}");
                    eprintln!("{msg}");
                    let _ = proxy.send_event(AppEvent::PortkeyDataReceived(msg));
                    return;
                }
            };

            // Holds the last successfully-fetched display text.
            let mut last_display: Option<String> = None;

            loop {
                match lib_portkey::get_portkey_cost(&api_key, None, None).await {
                    Ok(data) => {
                        // Format the total into a string like "Portkey: $36.38"
                        let display_text = format!("Portkey: ${:.2}", data.total_usd);
                        vlog!("Fetched Portkey cost: {}", display_text);

                        // Remember it for the next failure.
                        last_display = Some(display_text.clone());

                        // Send to Winit UI thread
                        let _ = proxy.send_event(AppEvent::PortkeyDataReceived(display_text));
                    }
                    Err(e) => {
                        // Errors always print, regardless of verbosity.
                        eprintln!("Failed to fetch Portkey data: {}", e);

                        // Reuse the previous value with an "e" suffix to flag it as stale.
                        let fallback = match &last_display {
                            Some(prev) => format!("{prev}e"),
                            None => "PK: Error".to_string(),
                        };

                        let _ = proxy.send_event(AppEvent::PortkeyDataReceived(fallback));
                    }
                }

                // Sleep for 1 minute before checking again
                tokio::time::sleep(tokio::time::Duration::from_mins(1)).await;
            }
        });
    }

    // 6. Forward tray menu events to the winit event loop.
    //     Must be called on the main thread (macOS requirement).
    tray_icon::menu::MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(AppEvent::MenuEvent(event));
    }));

    let mut app = MyApp { 
        tray_icon: None,
        has_api_key: app_config::has_api_key(),
    };

    // 7. Run the winit UI loop on the main thread (blocks forever)
    event_loop.run_app(&mut app)?;

    Ok(())
}

// Note for future improvement:
// To support hot-reloading of API keys without restart, the background task
// would need to check has_api_key() periodically or respond to ApiKeyChanged events.
// Current architecture requires app restart after setting API key.
