use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

const USAGE_ENDPOINT: &str = "https://chatgpt.com/backend-api/wham/usage";

#[derive(Debug, Deserialize)]
struct CodexAuth {
    tokens: Tokens,
}

#[derive(Debug, Deserialize)]
struct Tokens {
    access_token: String,
    account_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct UsageResponse {
    plan_type: Option<String>,
    rate_limit: Option<RateLimit>,
}

#[derive(Debug, Default, Deserialize)]
struct RateLimit {
    primary_window: Option<UsageWindow>,
    secondary_window: Option<UsageWindow>,
}

#[derive(Debug, Deserialize)]
struct UsageWindow {
    used_percent: Value,
}

#[derive(Debug, Serialize)]
struct UsageSnapshot {
    plan: String,
    session_used: i64,
    weekly_used: i64,
}

fn read_auth(path: &Path) -> Result<CodexAuth, String> {
    let contents = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "Could not read {}. Sign in with the Codex CLI first: {error}",
            path.display()
        )
    })?;

    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn parse_percentage(value: &Value) -> i64 {
    value
        .as_f64()
        .map(|percentage| percentage.round().clamp(0.0, 100.0) as i64)
        .unwrap_or(0)
}

fn format_plan(plan_type: Option<String>) -> String {
    let plan = plan_type.unwrap_or_else(|| "unknown".to_owned());
    format!("ChatGPT {}", plan.to_uppercase())
}

#[tauri::command]
async fn fetch_openai_usage() -> Result<UsageSnapshot, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not locate the home directory.".to_owned())?;
    let auth = read_auth(&home.join(".codex").join("auth.json"))?;

    let client = reqwest::Client::builder()
        .user_agent("codex-card")
        .build()
        .map_err(|error| format!("Could not initialize the HTTP client: {error}"))?;

    let mut request = client
        .get(USAGE_ENDPOINT)
        .bearer_auth(&auth.tokens.access_token);

    if let Some(account_id) = auth.tokens.account_id {
        request = request.header("ChatGPT-Account-Id", account_id);
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("Could not request Codex usage: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Codex usage request returned HTTP {}.",
            response.status()
        ));
    }

    let usage: UsageResponse = response
        .json()
        .await
        .map_err(|error| format!("Could not parse the Codex usage response: {error}"))?;
    let rate_limit = usage.rate_limit.unwrap_or_default();

    Ok(UsageSnapshot {
        plan: format_plan(usage.plan_type),
        session_used: rate_limit
            .primary_window
            .map(|window| parse_percentage(&window.used_percent))
            .unwrap_or(0),
        weekly_used: rate_limit
            .secondary_window
            .map(|window| parse_percentage(&window.used_percent))
            .unwrap_or(0),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fetch_openai_usage])
        .run(tauri::generate_context!())
        .expect("error while running Codex Card");
}

#[cfg(test)]
mod tests {
    use super::{format_plan, parse_percentage, read_auth};
    use serde_json::json;
    use std::io::Write;

    #[test]
    fn percentage_is_rounded_and_clamped() {
        assert_eq!(parse_percentage(&json!(42.6)), 43);
        assert_eq!(parse_percentage(&json!(120)), 100);
        assert_eq!(parse_percentage(&json!(-5)), 0);
        assert_eq!(parse_percentage(&json!("invalid")), 0);
    }

    #[test]
    fn plan_has_a_readable_fallback() {
        assert_eq!(format_plan(Some("plus".to_owned())), "ChatGPT PLUS");
        assert_eq!(format_plan(None), "ChatGPT UNKNOWN");
    }

    #[test]
    fn auth_file_is_parsed_without_exposing_extra_fields() {
        let mut file = tempfile::NamedTempFile::new().expect("temporary auth file");
        write!(
            file,
            r#"{{"tokens":{{"access_token":"secret","account_id":"account"}},"last_refresh":"ignored"}}"#
        )
        .expect("write auth file");

        let auth = read_auth(file.path()).expect("parse auth file");
        assert_eq!(auth.tokens.access_token, "secret");
        assert_eq!(auth.tokens.account_id.as_deref(), Some("account"));
    }
}
