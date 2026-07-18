use chrono::{Datelike, Utc};
use reqwest::Client;
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
pub struct WorkspaceCost {
    pub slug: String,
    pub spend_usd: f64,
}

#[derive(Debug, Clone)]
pub struct PortkeyCostResult {
    pub total_usd: f64,
    pub workspaces: Vec<WorkspaceCost>,
}

/// Fetches the Portkey cost since the start of the current month.
pub async fn get_portkey_cost(
    api_key: &str,
    workspace: Option<&str>,
    virtual_key: Option<&str>,
) -> Result<PortkeyCostResult, Box<dyn Error>> {
    let client = Client::new();

    // Calculate dates matching the Fish `date -u` commands
    let now_utc = Utc::now();
    let start_of_month = format!("{:04}-{:02}-01T00:00:00Z", now_utc.year(), now_utc.month());
    let now = now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Determine the list of workspaces to check
    let workspaces = if let Some(ws) = workspace {
        vec![ws.to_string()]
    } else {
        let res = client
            .get("https://api.portkey.ai/v1/admin/workspaces")
            .header("x-portkey-api-key", api_key)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err("Failed to fetch workspaces".into());
        }

        let parsed_res: WorkspacesResponse = res.json().await?;
        parsed_res.data.into_iter().map(|w| w.slug).collect()
    };

    let mut total_cents = 0.0;
    let mut workspace_costs = Vec::new();

    for ws in workspaces {
        // Build the query parameters dynamically
        let mut query = vec![
            ("time_of_generation_min", start_of_month.as_str()),
            ("time_of_generation_max", now.as_str()),
            ("workspace_slug", ws.as_str()),
        ];

        if let Some(vk) = virtual_key {
            query.push(("virtual_key", vk));
        }

        let res = client
            .get("https://api.portkey.ai/v1/analytics/graphs/cost")
            .header("x-portkey-api-key", api_key)
            .query(&query)
            .send()
            .await?;

        if !res.status().is_success() {
            eprintln!("Warning: failed to fetch cost for workspace '{}'", ws);
            continue;
        }

        let cost_res: CostResponse = res.json().await?;

        // Extract `.summary.total // 0`
        let ws_cents = cost_res.summary.total.unwrap_or(0.0);
        let ws_usd = ws_cents / 100.0;

        workspace_costs.push(WorkspaceCost {
            slug: ws,
            spend_usd: ws_usd,
        });

        total_cents += ws_cents;
    }

    let total_usd = total_cents / 100.0;

    Ok(PortkeyCostResult {
        total_usd,
        workspaces: workspace_costs,
    })
}
