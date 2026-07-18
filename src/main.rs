use chrono::Local;
use std::time::{Duration, Instant};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

// macOS-specific imports
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

// 1. Define a Struct to hold our application state
// We store the tray_icon here so it stays alive for the life of the app.
struct MyApp {
    tray_icon: Option<TrayIcon>,
}

// 2. Implement the ApplicationHandler trait
impl ApplicationHandler for MyApp {
    // This is called when the OS is ready for us to initialize our UI
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.tray_icon.is_none() {
            let icon = TrayIconBuilder::new()
                .with_title("Starting...")
                .build()
                .expect("Failed to build tray icon");

            self.tray_icon = Some(icon);
        }
    }

    // This is called constantly while the app is sleeping/waking
    // It replaces the old closure we had before.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Tell the event_loop (formerly `elwt`) to wake us up in 1 second
        let now = Instant::now();
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_secs(1)));

        // Update the menu bar title
        if let Some(tray_icon) = &self.tray_icon {
            let current_time = Local::now().format("%H:%M:%S").to_string();
            tray_icon.set_title(Some(current_time));
        }
    }

    // Required by the trait, but we can ignore it since we don't have Windows yet
    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: WindowId,
        _event: WindowEvent,
    ) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the Event Loop Builder
    let mut builder = EventLoop::builder();

    // Hide from the Dock and CMD+Tab
    #[cfg(target_os = "macos")]
    builder.with_activation_policy(ActivationPolicy::Accessory);

    let event_loop = builder.build()?;

    // Create our app state
    let mut app = MyApp { tray_icon: None };

    // 3. Run the app using the new `run_app` method!
    event_loop.run_app(&mut app)?;

    Ok(())
}
