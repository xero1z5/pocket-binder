use dioxus::{html::{a::target, meta::content, script::r#async}, prelude::*};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

//=================== APP ====================

// this represents the entire database file state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardCollection {
    pub accounts: Vec<Account>,
    pub inventory: Vec<Inventory>,
}

// account definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub name: String,
    pub id: String,
    pub main: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String, // card name
    pub rarity: String, // ? enum -> shiny fa, immersive, 2* (RR, FA, Trainers)
    pub card_type: String, // pokemon, trainer
    pub pack: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inventory {
    pub card: Card,
    pub owners: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfficialCard {
    pub id: String,
    pub name: String,
    pub image: String,
    pub pack: String,
    pub rarity: String,

    #[serde(rename="type", default)]
    pub card_type: String,
}

//=================== LOGIC =======================

impl CardCollection {
     
    pub fn add_card(&mut self, card: Card, account_name: &str, quantity: i32){

        if let Some(entry)=self.inventory.iter_mut().find(|e| e.card==card){
            let current_count = entry.owners.entry(account_name.to_string()).or_insert(0);
            *current_count+=quantity;
        }
        else {
            let mut new_owners = HashMap::new();
            new_owners.insert(account_name.to_string(),quantity);

            self.inventory.push(Inventory{
                card,
                owners: new_owners,
            });
        }
    }

    pub fn remove_card(&mut self, card: &Card, account_name: &str, quantity: i32) -> Result<(),String> {
        let entry_index=self.inventory.iter().position(|e| &e.card==card);

        if let Some(index)=entry_index{
            let entry = &mut self.inventory[index];

            if let Some(current_count) = entry.owners.get_mut(account_name){
                if *current_count >= quantity{
                    *current_count-=quantity;

                    if *current_count==0{
                        entry.owners.remove(account_name);
                    }
                }
                else{
                    return Err(format!("{} does not have enough of this card.", account_name));
                }
            }
            else{
                return Err(format!("{} does not own this card.", account_name));
            }

            if self.inventory[index].owners.is_empty() {
                self.inventory.remove(index);
            }
            Ok(())
        }
        else {
            Err("Card not found in database".to_string())
        }
    }

    pub fn trade_card(&mut self, card: &Card, from_account: &str, to_account: &str, quantity: i32) -> Result<(), String> {
        self.remove_card(card,from_account, quantity)?;
        self.add_card(card.clone(), to_account, quantity);

        Ok(())
    }
}

//======================= DX ===================

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // =========================================================================
    // 1. STATE SIGNALS (All restored and aligned!)
    // =========================================================================
    
    // Core Data
    let mut collection = use_signal(|| CardCollection { accounts: Vec::new(), inventory: Vec::new() });
    let mut sync_status = use_signal(|| String::new());
    
    // UI Toggles & Search
    let mut search_query = use_signal(|| String::new());
    let mut selected_account_filter = use_signal(|| String::from("All"));
    let mut show_add_modal = use_signal(|| false);
    let mut add_search_query = use_signal(|| String::new());
    let mut add_target_account = use_signal(|| String::new());

    // Supabase Auth Signals
    let mut show_login_modal = use_signal(|| true);
    let mut user_email = use_signal(|| LocalStorage::get::<String>("user_email").unwrap_or_default());
    let mut user_password = use_signal(|| String::new());
    let mut auth_token = use_signal(|| LocalStorage::get::<String>("supabase_token").unwrap_or_default());

    // Automatically bypass modal if token exists on startup
    use_effect(move || {
        if !auth_token.read().is_empty() {
            show_login_modal.set(false);
            
            // Auto-fetch collection on load
            let token = auth_token.read().clone();
            spawn(async move {
                sync_status.set("🔄 Syncing collection...".to_string());
                if let Ok(data) = load_from_supabase(&token).await {
                    collection.set(data);
                    sync_status.set("✅ Cloud Synced!".to_string());
                } else {
                    sync_status.set("❌ Session expired. Please log in again.".to_string());
                    show_login_modal.set(true);
                }
            });
        }
    });

    // =========================================================================
    // 2. FETCH OFFICIAL API DATABASE
    // =========================================================================
    let image_db = use_resource(move || async move {
        let url = "https://raw.githubusercontent.com/chase-manning/pokemon-tcg-pocket-cards/refs/heads/main/v4.json";

        let response = reqwest::get(url).await.ok()?;
        let official_cards = response.json::<Vec<OfficialCard>>().await.ok()?;

        let mut api_db: HashMap<String, OfficialCard> = HashMap::new();
        for card in official_cards {
            api_db.insert(card.id.clone(), card);
        }
        
        Some(api_db)
    });

    // =========================================================================
    // 3. UI LAYOUT
    // =========================================================================
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-gray-900 text-white min-h-screen p-6 font-sans",
            
            // --- HEADER ---
            header { class: "border-b border-gray-800 pb-4 mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4",
                h1 { class: "text-2xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-red-500 to-orange-400", 
                    "🧬 POCKET TCG BINDER" 
                }
                
                // Login Button / User Profile
                div { class: "flex items-center gap-4",
                    if auth_token.read().is_empty() {
                        button {
                            class: "bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-600 font-bold py-2 px-4 rounded-lg transition-colors flex items-center gap-2 shadow-lg",
                            onclick: move |_| show_login_modal.set(true),
                            "🔑 Cloud Login"
                        }
                    } else {
                        div { class: "flex items-center gap-3 bg-gray-800/50 py-1.5 px-4 rounded-full border border-gray-700",
                            span { class: "text-sm text-green-400 font-mono font-bold", "👤 {user_email}" }
                            button {
                                class: "text-xs text-gray-500 hover:text-red-400 hover:underline transition-colors ml-2",
                                onclick: move |_| {
                                    auth_token.set(String::new());
                                    user_email.set(String::new());
                                    LocalStorage::delete("supabase_token");
                                    LocalStorage::delete("user_email");
                                },
                                "Logout"
                            }
                        }
                    }
                }
            }

            // --- FILTER BAR ---
            div { class: "flex flex-col md:flex-row gap-4 mb-6",
                // Search Input
                input {
                    class: "flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500",
                    placeholder: "Search by card name...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value())
                }

                // Account Filter Dropdown
                select {
                    class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500 cursor-pointer",
                    onchange: move |evt| selected_account_filter.set(evt.value()),
                    option { value: "All", "All Accounts" }
                    for account in collection.read().accounts.iter() {
                        option { value: "{account.name}", "{account.name}" }
                    }
                }

                // New Card Button
                button {
                    class: "bg-orange-600 hover:bg-orange-500 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95",
                    onclick: move |_| show_add_modal.set(true),
                    "➕ Add Card"
                }

                // Save to Supabase Button
                button {
                    class: "bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95 flex items-center gap-2",
                    onclick: move |_| {
                        sync_status.set("⏳ Saving to Cloud...".to_string());
                        
                        let current_collection = collection.read().clone();
                        let token_to_use = auth_token.read().clone();
                        
                        spawn(async move {
                            match save_to_supabase(current_collection, token_to_use).await {
                                Ok(_) => sync_status.set("✅ Cloud Synced!".to_string()),
                                Err(e) => sync_status.set(format!("❌ {}", e)),
                            }
                        });
                    },
                    "☁️ Sync to Cloud"
                }

                // status indicator
                if !sync_status.read().is_empty() {
                    span { class: "text-sm font-mono flex items-center", "{sync_status}" }
                }
            }

            // --- THE VISUAL GRID ---
            div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4",
                for entry in collection.read().inventory.iter().filter(|e| {
                    let matches_search = search_query.read().is_empty() || 
                                         e.card.name.to_lowercase().contains(&search_query.read().to_lowercase());
                    
                    let matches_account = &*selected_account_filter.read() == "All" || 
                                          e.owners.contains_key(&*selected_account_filter.read());
                    
                    matches_search && matches_account
                }) {
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl p-3 flex flex-col items-center shadow-lg transition-transform hover:scale-105",

                        // Dynamic Image Rendering
                        {
                            let image_url = if let Some(Some(api_map)) = &*image_db.read() {
                                api_map.get(&entry.card.id).map(|c| c.image.clone())
                            } else {
                                None
                            };

                            if let Some(url) = image_url {
                                rsx! {
                                    img {
                                        src: "{url}",
                                        alt: "{entry.card.name}",
                                        class: "w-full rounded-lg mb-3 shadow-md border border-gray-600 aspect-[63/88] object-cover",
                                        loading: "lazy"
                                    }
                                }
                            } else {
                                rsx!{
                                    div { class: "w-full aspect-[63/88] bg-gray-700 rounded-lg mb-3 flex items-center justify-center border border-gray-600 animate-pulse",
                                        span { class: "text-4xl opacity-50", "🃏" }
                                    }
                                }
                            }
                        }
                        
                        // Card Info
                        h2 { class: "font-bold text-sm text-center truncate w-full", "{entry.card.name}" }
                        p { class: "text-xs text-orange-400 mb-2 uppercase tracking-wide", "{entry.card.rarity}" }
                        
                        // Who owns it?
                        div { class: "w-full flex flex-wrap gap-1 justify-center mt-auto",
                            for (owner_name, count) in entry.owners.iter() {
                                if *count > 0 {
                                    span { class: "text-[10px] bg-blue-900/50 text-blue-200 border border-blue-700/50 px-2 py-0.5 rounded-full font-mono",
                                        "{owner_name}: {count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if collection.read().inventory.is_empty() {
                div { class: "text-center text-gray-500 mt-12",
                    "No cards found matching your criteria."
                }
            }
        }
    
        // =========================================================================
        // 4. THE ADD CARD MODAL
        // =========================================================================
        if *show_add_modal.read() {
            div { class: "fixed inset-0 bg-black/90 flex flex-col items-center p-4 md:p-10 z-50",
                
                // Modal Header & Controls
                div { class: "w-full max-w-6xl bg-gray-900 border border-gray-700 rounded-t-xl p-4 flex flex-col md:flex-row gap-4 justify-between items-center shadow-2xl",
                    h2 { class: "text-2xl font-bold text-orange-400", "Add from Database" }
                    
                    div { class: "flex w-full md:w-auto gap-2",
                        input {
                            class: "flex-1 md:w-64 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500",
                            placeholder: "Search official API...",
                            value: "{add_search_query}",
                            oninput: move |evt| add_search_query.set(evt.value())
                        }
                        
                        select {
                            class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 cursor-pointer",
                            onchange: move |evt| add_target_account.set(evt.value()),
                            for account in collection.read().accounts.iter() {
                                option { value: "{account.name}", "{account.name}" }
                            }
                        }
                        
                        button {
                            class: "bg-red-900/80 hover:bg-red-800 text-red-200 px-4 py-2 rounded-lg font-bold",
                            onclick: move |_| show_add_modal.set(false),
                            "Close"
                        }
                    }
                }

                // Modal Results Grid
                div { class: "w-full max-w-6xl flex-1 bg-gray-800/50 border-x border-b border-gray-700 rounded-b-xl p-4 overflow-y-auto",
                    div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-3",
                        
                        {
                            // 1. Safely extract and filter the data into an owned Vector FIRST
                            let mut display_cards = Vec::new();
                            
                            if let Some(Some(db)) = &*image_db.read() {
                                let search_term = add_search_query.read().to_lowercase();
                                let mut count = 0;
                                
                                for card in db.values() {
                                    if search_term.is_empty() || card.name.to_lowercase().contains(&search_term) {
                                        display_cards.push(card.clone());
                                        count += 1;

                                        if count >= 32 {
                                            break;
                                        }
                                    }
                                }
                            }

                            // 2. Now that the `image_db.read()` lock is dropped, we can safely render the UI
                            if display_cards.is_empty() {
                                rsx! { p { class: "col-span-full text-center text-gray-500 mt-10 animate-pulse", "Loading API Database or No Results Found..." } }
                            } else {
                                rsx! {
                                    for official_card in display_cards {
                                        div { 
                                            class: "relative group cursor-pointer transition-transform hover:scale-105 hover:z-10",
                                            
                                            // THE ADD CARD ACTION
                                            onclick: move |_| {
                                                let new_card = Card {
                                                    id: official_card.id.clone(),
                                                    name: official_card.name.clone(),
                                                    rarity: official_card.rarity.clone(),
                                                    card_type: official_card.card_type.clone(),
                                                    pack: official_card.pack.clone(),
                                                };
                                                collection.write().add_card(
                                                    new_card, 
                                                    &*add_target_account.read(), 
                                                    1
                                                );
                                            },
                                            
                                            img { 
                                                src: "{official_card.image}", 
                                                loading: "lazy",
                                                class: "w-full rounded border border-gray-600 shadow-md" 
                                            }
                                            
                                            // Hover Overlay
                                            div { class: "absolute inset-0 bg-blue-500/50 opacity-0 group-hover:opacity-100 rounded flex items-center justify-center backdrop-blur-sm transition-opacity",
                                                span { class: "bg-gray-900 text-white text-xs font-bold px-2 py-1 rounded-full", "+ Add" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // =========================================================================
        // 5. THE LOGIN MODAL
        // =========================================================================
        if *show_login_modal.read() {
            div { class: "fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                div { class: "bg-gray-900 border border-gray-700 rounded-xl p-6 w-full max-w-md shadow-2xl flex flex-col gap-4",
                    
                    h2 { class: "text-2xl font-black text-white", "Welcome to Pocket Binder" }
                    
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-gray-400 font-bold uppercase", "Email Address" }
                        input {
                            r#type: "email",
                            class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:border-blue-500",
                            placeholder: "you@example.com",
                            value: "{user_email}",
                            oninput: move |evt| user_email.set(evt.value())
                        }
                    }

                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-gray-400 font-bold uppercase", "Password" }
                        input {
                            r#type: "password",
                            class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:border-blue-500",
                            placeholder: "••••••••",
                            value: "{user_password}",
                            oninput: move |evt| user_password.set(evt.value())
                        }
                    }

                    if !sync_status.read().is_empty(){
                        div { class: "text-center text-sm font-mono mt-2 p-2 rounded bg-gray-800/80 text-orange-400 border border-gray-700",
                            "{sync_status}"
                        }
                    }

                    div { class: "flex gap-2 mt-4",
                        // SIGN IN BUTTON
                        button {
                            class: "bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-4 rounded-lg flex-1 transition-transform active:scale-95",
                            onclick: move |_| {
                                let email = user_email.read().clone();
                                let pass = user_password.read().clone();

                                if !email.is_empty() && !pass.is_empty() {
                                    sync_status.set("🔄 Authenticating...".to_string());
                                    
                                    spawn(async move {
                                        // false = standard login
                                        match supabase_auth(&email, &pass, false).await {
                                            Ok(token) => {
                                                // Save to vault
                                                let _ = LocalStorage::set("user_email", &email);
                                                let _ = LocalStorage::set("supabase_token", &token);
                                                auth_token.set(token.clone());
                                                show_login_modal.set(false);
                                                
                                                // Immediately load their collection
                                                sync_status.set("🔄 Downloading Binder...".to_string());
                                                if let Ok(data) = load_from_supabase(&token).await {
                                                    collection.set(data);
                                                    sync_status.set("✅ Loaded!".to_string());
                                                }
                                            },
                                            Err(e) => sync_status.set(format!("❌ {}", e)),
                                        }
                                    });
                                }
                            },
                            "Sign In"
                        }
                        
                        // CREATE ACCOUNT BUTTON
                        button {
                            class: "bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-4 rounded-lg flex-1 transition-transform active:scale-95",
                            onclick: move |_| {
                                let email = user_email.read().clone();
                                let pass = user_password.read().clone();

                                if !email.is_empty() && !pass.is_empty() {
                                    sync_status.set("🔄 Creating Account...".to_string());
                                    spawn(async move {
                                        // true = sign up
                                        match supabase_auth(&email, &pass, true).await {
                                            Ok(token) => {
                                                let _ = LocalStorage::set("user_email", &email);
                                                let _ = LocalStorage::set("supabase_token", &token);
                                                auth_token.set(token);
                                                show_login_modal.set(false);
                                                sync_status.set("✅ Account Created!".to_string());
                                            },
                                            Err(e) => sync_status.set(format!("❌ {}", e)),
                                        }
                                    });
                                }
                            },
                            "Sign Up"
                        }
                    }
                }
            }
        }    
    }
}

// =========================================================================
// 5. SUPABASE SYNC LOGIC
// =========================================================================

// Safe to hardcode for a frontend web app: Anonymous Keys only have permissions that RLS allows!
const SUPABASE_URL: &str = "https://zlqxrapobcheqfapchao.supabase.co"; // <-- Paste yours here
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InpscXhyYXBvYmNoZXFmYXBjaGFvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODA5NDIwNjgsImV4cCI6MjA5NjUxODA2OH0.P9NRbm1-7orI1dP0TIcRzOkDjSJa1IGYtOdhQBXNmXU";     // <-- Paste yours here

// 1. Authenticate (Login or Sign Up)
async fn supabase_auth(email: &str, password: &str, is_signup: bool) -> Result<String, String> {
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

// 2. Load the User's Binder
async fn load_from_supabase(token: &str) -> Result<CardCollection, String> {
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

// 3. Save the Binder (Upsert)
async fn save_to_supabase(collection: CardCollection, token: String) -> Result<(), String> {
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
