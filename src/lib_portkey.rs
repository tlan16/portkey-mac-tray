use chrono::{Datelike, Utc};
use serde::Deserialize;
use std::error::Error;

// ---------------------------------------------------------
// Internal structs for JSON deserialization
// ---------------------------------------------------------
#[derive(Deserialize)]
struct Workspace {
    slug: String,
}

#[derive(Deserialize)]
struct WorkspacesResponse {
    data: Vec<Workspace>,
}

#[derive(Deserialize)]
struct CostSummary {
    total: Option<f64>,
}

#[derive(Deserialize)]
struct CostResponse {
    summary: CostSummary,
}

// ---------------------------------------------------------
// Public structs to return structured data to the caller
// ---------------------------------------------------------
#[derive(Debug, Clone)]
pub struct PortkeyCostResult {
    pub total_usd: f64,
}

/// Fetches the Portkey cost since the start of the current month.
pub fn get_portkey_cost(
    api_key: &str,
    workspace: Option<&str>,
    virtual_key: Option<&str>,
) -> Result<PortkeyCostResult, Box<dyn Error>> {
    // Calculate dates matching the Fish `date -u` commands
    let now_utc = Utc::now();
    let start_of_month = format!("{:04}-{:02}-01T00:00:00Z", now_utc.year(), now_utc.month());
    let now = now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Determine the list of workspaces to check
    let workspaces = if let Some(ws) = workspace {
        vec![ws.to_string()]
    } else {
        let res = ureq::get("https://api.portkey.ai/v1/admin/workspaces")
            .header("x-portkey-api-key", api_key)
            .call()?;

        if !res.status().is_success() {
            return Err("Failed to fetch workspaces".into());
        }

        let parsed_res: WorkspacesResponse = res.into_body().read_json()?;
        parsed_res.data.into_iter().map(|w| w.slug).collect()
    };

    let mut total_cents = 0.0;

    for ws in workspaces {
        // Build the URL with query parameters
        let mut url = format!(
            "https://api.portkey.ai/v1/analytics/graphs/cost?time_of_generation_min={}&time_of_generation_max={}&workspace_slug={}",
            start_of_month, now, ws
        );

        if let Some(vk) = virtual_key {
            url.push_str(&format!("&virtual_key={}", vk));
        }

        let res = ureq::get(&url)
            .header("x-portkey-api-key", api_key)
            .call();

        match res {
            Ok(res) if res.status().is_success() => {
                let cost_res: CostResponse = res.into_body().read_json()?;
                let ws_cents = cost_res.summary.total.unwrap_or(0.0);
                total_cents += ws_cents;
            }
            _ => {
                eprintln!("Warning: failed to fetch cost for workspace '{}'", ws);
                continue;
            }
        }
    }

    let total_usd = total_cents / 100.0;

    Ok(PortkeyCostResult {
        total_usd,
    })
}
