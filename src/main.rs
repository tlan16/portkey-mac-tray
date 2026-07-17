use chrono::Local;
use std::time::{Duration, Instant};
use tray_icon::TrayIconBuilder;
use winit::event_loop::{ControlFlow, EventLoopBuilder};

// This is a macOS-specific trait that lets us hide the app from the Dock
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up the event loop
    let mut builder = EventLoopBuilder::new();

    // Tell macOS this is an "Accessory" app (runs in menu bar, hides from Dock and CMD+Tab)
    #[cfg(target_os = "macos")]
    builder.with_activation_policy(ActivationPolicy::Accessory);

    let event_loop = builder.build()?;

    // 2. Create the menu bar item
    // On macOS, tray icons don't strictly need an image; they can just be text!
    let mut tray_icon = TrayIconBuilder::new().with_title("Starting...").build()?;

    // 3. Run the event loop
    event_loop.run(move |_event, elwt| {
        // Tell the OS to wake this thread up exactly 1 second from now.
        // This is crucial! It ensures your app uses 0.0% CPU while idling.
        let now = Instant::now();
        elwt.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_secs(1)));

        // Get the current time and update the menu bar title
        let current_time = Local::now().format("%H:%M:%S").to_string();
        tray_icon.set_title(Some(&current_time));
    })?;

    Ok(())
}
