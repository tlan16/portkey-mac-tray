# GitHub Login Test - Troubleshooting

## Test Failure: `test_login_to_github`

### Problem
The test fails with the error: "Could not auto detect a chrome executable"

### Root Cause
The `spider` crate with `chrome_headed` feature requires a Chrome or Chromium browser to be installed on the system, but none was found.

### Solution

#### Option 1: Install Google Chrome (Recommended)

**Using Homebrew:**
```bash
brew install --cask google-chrome
```

**Manual Installation:**
1. Visit https://www.google.com/chrome/
2. Download and install Chrome for macOS
3. Verify installation at `/Applications/Google Chrome.app`

#### Option 2: Use a Custom Chrome Installation

If Chrome is installed in a non-standard location, set the `CHROME` environment variable:

```bash
export CHROME=/path/to/chrome/executable
```

#### Option 3: Install Chromium

```bash
brew install chromium
export CHROME=/opt/homebrew/bin/chromium
```

### Verification

After installing Chrome, run the test:

```bash
cargo test test_login_to_github --lib -- --nocapture
```

### Technical Details

**Required Dependencies:**
- `spider` crate with `chrome_headed` feature enabled
- Chrome or Chromium browser (for browser automation)

**Browser Detection:**
The spider crate looks for Chrome in standard locations:
- `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome` (macOS)
- Environment variable `CHROME` if set

**Additional Notes:**
- The test opens a visible browser window (non-headless mode)
- Ensure you have a display environment available (not suitable for CI without Xvfb)
