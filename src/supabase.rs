use crate::models::CardCollection;
use std::sync::OnceLock;

// =========================================================================
// SUPABASE SYNC LOGIC
// =========================================================================

// Safe to hardcode for a frontend web app: Anonymous Keys only have permissions that RLS allows!
const SUPABASE_URL: &str = "https://zlqxrapobcheqfapchao.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpscXhyYXBvYmNoZXFmYXBjaGFvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODA5NDIwNjgsImV4cCI6MjA5NjUxODA2OH0.P9NRbm1-7orI1dP0TIcRzOkDjSJa1IGYtOdhQBXNmXU";

// This keeps the TLS connection "warm" and prevents CORS Preflight cold starts!
fn get_client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| reqwest::Client::new()).clone()
}

// Authenticate (Login or Sign Up)
pub async fn supabase_auth(email: &str, password: &str, is_signup: bool) -> Result<String, String> {
    let endpoint = if is_signup { "signup" } else { "token?grant_type=password" };
    let url = format!("{}/auth/v1/{}", SUPABASE_URL, endpoint);
    
    let res = get_client().post(&url)
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    if status.is_success() {
        if let Some(token) = json["access_token"].as_str() {
            return Ok(token.to_string());
        }
    }
    
    // Fallback error message if auth fails
    Err(json["error_description"].as_str().or(json["msg"].as_str()).unwrap_or("Auth failed").to_string())
}

// Load the User's Binder
pub async fn load_from_supabase(token: &str) -> Result<CardCollection, String> {
    let url = format!("{}/rest/v1/binders?select=collection_data", SUPABASE_URL);
    
    let res = get_client().get(&url)
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    if status.is_success() {
        let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        
        // Supabase returns an array of matching rows.
        if let Some(rows) = json.as_array() {
            if !rows.is_empty() {
                let collection: CardCollection = serde_json::from_value(rows[0]["collection_data"].clone())
                    .map_err(|e| format!("JSON Parse Error: {}", e))?;
                return Ok(collection);
            }
        }
        // If they have no rows, they are a new user! Return an empty binder.
        return Ok(CardCollection { accounts: Vec::new(), inventory: Vec::new(), wishlist: Vec::new(), tradable: Vec::new() });
    }
    
    Err(format!("Load failed with status: {}", status))
}

// Save the Binder (Upsert) - NOW WITH SILENT MICRO-RETRIES
pub async fn save_to_supabase(collection: CardCollection, token: String) -> Result<(), String> {
    let url = format!("{}/rest/v1/binders?on_conflict=user_id", SUPABASE_URL);
    let client = get_client();
    let payload = serde_json::json!({ "collection_data": collection });

    let mut attempts = 0;
    let max_attempts = 3; // We will try 3 times before actually throwing an error to the UI

    loop {
        attempts += 1;
        
        let res = client.post(&url)
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", format!("Bearer {}", token))
            .header("Prefer", "resolution=merge-duplicates")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) => {
                let status = response.status();
                if status.is_success() || status == reqwest::StatusCode::CREATED {
                    return Ok(()); // Success! Break the loop and return.
                } else if attempts >= max_attempts {
                    return Err(format!("Save failed with status: {}", status)); // Out of retries
                }
            },
            Err(e) => {
                if attempts >= max_attempts {
                    return Err(e.to_string()); // Out of retries, connection totally dead
                }
            }
        }

        // If we made it here, the request failed but we still have retries left.
        // Wait 150ms to let Supabase "wake up" and try again!
        gloo_timers::future::sleep(std::time::Duration::from_millis(150)).await;
    }
}
