mod lib_portkey;
mod app_config;

use chrono::Local;
use std::time::{Duration, Instant};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::WindowId;

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
use crate::app_config::APP_CONFIG;

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
            println!("Loaded config: API key = {}", APP_CONFIG.portkey_api_key);

            let icon = TrayIconBuilder::new()
                .with_title("Starting...")
                .build()
                .expect("Failed to build tray icon");

            self.tray_icon = Some(icon);
        }
    }

    // 2. This new function receives messages from our Tokio tasks!
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::PortkeyDataReceived(data) => {
                println!("UI Thread received async data: {}", data);
                if let Some(tray_icon) = &self.tray_icon {
                    tray_icon.set_title(Some(data));
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_secs(1)));

        // We can keep the ticking clock, or remove it if you only want API data
        /*
        if let Some(tray_icon) = &self.tray_icon {
            let current_time = Local::now().format("%H:%M:%S").to_string();
            tray_icon.set_title(Some(current_time));
        }
        */
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
        // Since it's a loop, you can fetch it every X minutes!
        loop {
            // Call our updated lib function
            match lib_portkey::get_portkey_cost(&APP_CONFIG.portkey_api_key, None, None).await {
                Ok(data) => {
                    // Format the total into a string like "PK: $36.38"
                    let display_text = format!("Portkey: ${:.2}", data.total_usd);

                    // Send to Winit UI thread
                    let _ = proxy.send_event(AppEvent::PortkeyDataReceived(display_text));
                }
                Err(e) => {
                    eprintln!("Failed to fetch Portkey data: {}", e);
                    let _ = proxy.send_event(AppEvent::PortkeyDataReceived("PK: Error".to_string()));
                }
            }

            // Sleep for 5 minutes before checking again
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    });

    let mut app = MyApp { tray_icon: None };

    // 6. Run the winit UI loop on the main thread (blocks forever)
    event_loop.run_app(&mut app)?;

    Ok(())
}
