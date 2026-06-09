use crate::models::CardCollection;

// =========================================================================
// SUPABASE SYNC LOGIC
// =========================================================================

// Safe to hardcode for a frontend web app: Anonymous Keys only have permissions that RLS allows!
const SUPABASE_URL: &str = "https://zlqxrapobcheqfapchao.supabase.co"; // <-- Paste yours here
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpscXhyYXBvYmNoZXFmYXBjaGFvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODA5NDIwNjgsImV4cCI6MjA5NjUxODA2OH0.P9NRbm1-7orI1dP0TIcRzOkDjSJa1IGYtOdhQBXNmXU";     // <-- Paste yours here

// Authenticate (Login or Sign Up)
pub async fn supabase_auth(email: &str, password: &str, is_signup: bool) -> Result<String, String> {
    let endpoint = if is_signup { "signup" } else { "token?grant_type=password" };
    let url = format!("{}/auth/v1/{}", SUPABASE_URL, endpoint);
    
    let client = reqwest::Client::new();
    let res = client.post(&url)
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
    let client = reqwest::Client::new();
    
    let res = client.get(&url)
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
        return Ok(CardCollection { accounts: Vec::new(), inventory: Vec::new() });
    }
    
    Err(format!("Load failed with status: {}", status))
}

// Save the Binder (Upsert)
pub async fn save_to_supabase(collection: CardCollection, token: String) -> Result<(), String> {
    // "on_conflict=user_id" tells PostgreSQL to update the row if it exists, or insert if it doesn't.
    let url = format!("{}/rest/v1/binders?on_conflict=user_id", SUPABASE_URL);
    let client = reqwest::Client::new();
    
    let res = client.post(&url)
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", token))
        .header("Prefer", "resolution=merge-duplicates")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "collection_data": collection
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    if status.is_success() || status == reqwest::StatusCode::CREATED {
        Ok(())
    } else {
        Err(format!("Save failed with status: {}", status))
    }
}
