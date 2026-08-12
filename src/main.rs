use my_mac_tray::app_config::APP_CONFIG;
use my_mac_tray::lib_portkey;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
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
}

// ---------------------------------------------------------
// APP STATE & EVENT LOOP
// ---------------------------------------------------------
struct MyApp {
    tray_icon: Option<TrayIcon>,
}

// Notice we changed `ApplicationHandler` to `ApplicationHandler<AppEvent>`
impl ApplicationHandler<AppEvent> for MyApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.tray_icon.is_none() {
            let title = match &*APP_CONFIG {
                Ok(cfg) => {
                    vlog!("Loaded config: API key = {}", cfg.portkey_api_key);
                    "Starting...".to_string()
                }
                Err(e) => {
                    eprintln!("Config error: {}", e);
                    format!("PK: Error: {e}")
                }
            };

            let icon = TrayIconBuilder::new()
                .with_title(&title)
                .build()
                .expect("Failed to build tray icon");

            self.tray_icon = Some(icon);
        }
    }

    // 2. This new function receives messages from our Tokio tasks!
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::PortkeyDataReceived(data) => {
                vlog!("UI Thread received async data: {}", data);
                if let Some(tray_icon) = &self.tray_icon {
                    tray_icon.set_title(Some(data));
                }
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

    // 4. Create the Tokio runtime manually
    let rt = tokio::runtime::Runtime::new()?;

    // 5. Spawn an async task into the background
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

    let mut app = MyApp { tray_icon: None };

    // 6. Run the winit UI loop on the main thread (blocks forever)
    event_loop.run_app(&mut app)?;

    Ok(())
}
