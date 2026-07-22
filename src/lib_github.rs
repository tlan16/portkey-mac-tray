use spider::chromiumoxide::browser::HeadlessMode;
use spider::chromiumoxide::cdp::browser_protocol::target::CreateTargetParamsBuilder;
use spider::chromiumoxide::handler::viewport::Viewport;
use spider::chromiumoxide::{Browser, BrowserConfig};
use spider::tokio_stream::StreamExt;

/// Attempts to login to GitHub using browser automation.
/// 
/// # Requirements
/// - Chrome or Chromium browser must be installed on the system
/// - On macOS, Chrome should be at /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
/// 
/// # Errors
/// Returns an error if:
/// - No Chrome/Chromium browser is found
/// - Browser fails to launch
/// - Navigation to GitHub login page fails
#[allow(dead_code)]
pub async fn login_to_github() -> Result<(), Box<dyn std::error::Error>> {
    let screen_width = 1920u32;
    let screen_height = 1080u32;

    println!("Starting browser launch...");
    
    let browser_config = BrowserConfig::builder()
        .window_size(screen_width, screen_height)
        .disable_default_args()
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--excludeSwitches=enable-automation")
        .arg("--disable-infobars")
        .arg("--lang=en-US,en")
        .arg("--disable-setuid-sandbox")
        .arg("--disable-extensions")
        .arg("--no-default-browser-check")
        .arg("--no-first-run")
        .arg("--disable-features=InfiniteSessionRestore,BioSensor")
        .arg("--no-session-restore-bubble")
        .arg("--disable-session-crashed-bubble")
        .arg("--no-profile-loading")
        .arg("--disable-sync")
        .arg("--disable-device-discovery-notifications")
        .arg("--disable-managed-configuration-service")
        .arg("--disable-component-extensions-with-background-pages")
        .arg("--disable-component-update")
        .arg("--disable-plugins")
        .arg("--disable-default-apps")
        .arg("--no-default-browser-check")
        .arg("--no-managed-user-acknowledgment-check")
        .viewport(Viewport {
            width: screen_width,
            height: screen_height,
            device_scale_factor: None,
            emulating_mobile: false,
            is_landscape: false,
            has_touch: false,
        })
        .headless_mode(HeadlessMode::False)
        .build()
        .map_err(|e| {
            eprintln!("Failed to build browser config: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Install Google Chrome from: https://www.google.com/chrome/");
            eprintln!("   Or use Homebrew: brew install --cask google-chrome");
            eprintln!("2. If Chrome is installed in a custom location, set the CHROME environment variable:");
            eprintln!("   export CHROME=/path/to/chrome/executable");
            e
        })?;
    
    let (browser, mut handler) = match Browser::launch(browser_config).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to launch browser: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Install Google Chrome from: https://www.google.com/chrome/");
            eprintln!("   Or use Homebrew: brew install --cask google-chrome");
            eprintln!("2. If Chrome is installed in a custom location, set the CHROME environment variable:");
            eprintln!("   export CHROME=/path/to/chrome/executable");
            return Err(Box::new(e));
        }
    };
    
    println!("Browser launched successfully");

    let _handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let initial_page_url = "https://github.com/login";
    let create_target_params_builder = CreateTargetParamsBuilder::default()
        .url(initial_page_url)
        .background(false)
        .focus(true);
    let create_target_params = create_target_params_builder.build()?;
    let page = browser.new_page(create_target_params).await?;

    page.wait_for_network_almost_idle().await?;
    println!("Initial page loaded: {}", initial_page_url);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_login_to_github() {
        // This test opens a browser window and navigates to GitHub login
        // Requires Chrome or Chromium to be installed
        match super::login_to_github().await {
            Ok(_) => println!("Test completed successfully"),
            Err(e) => {
                eprintln!("Test failed: {}", e);
                panic!("Test failed: {}", e);
            }
        }
    }
}
