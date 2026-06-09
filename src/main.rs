mod models;
mod supabase;
mod components;

use dioxus::prelude::*;
use std::collections::HashMap;
use gloo_storage::{LocalStorage, Storage};

use models::*;
use supabase::*;

use components::{
    header::Header,
    filter::FilterBar,
    grid::CardGrid,
    toast::Toast,
    add_card::AddCardModal,
    account::AccountModal,
    login::LoginModal,
};

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
    // STATE SIGNALS
    // =========================================================================
    
    // Core Data
    let mut collection = use_signal(|| CardCollection { accounts: Vec::new(), inventory: Vec::new() });
    let mut sync_status = use_signal(|| String::new());
    
    // UI Toggles & Search
    let mut search_query = use_signal(|| String::new());
    let mut selected_account_filter = use_signal(|| String::from("All"));
    let mut show_add_modal = use_signal(|| false);
    let mut show_filter_menu = use_signal(|| false);
    let mut add_search_query = use_signal(|| String::new());
    let mut add_target_account = use_signal(|| String::new());

    // Supabase Auth Signals
    let mut show_login_modal = use_signal(|| true);
    let mut user_email = use_signal(|| LocalStorage::get::<String>("user_email").unwrap_or_default());
    let mut user_password = use_signal(|| String::new());
    let mut auth_token = use_signal(|| LocalStorage::get::<String>("supabase_token").unwrap_or_default());

    // Notifications
    let mut toast_message = use_signal(|| None::<String>);

    // Adding accounts
    let mut show_account_modal = use_signal(|| false);
    let mut new_acc_name = use_signal(|| String::new());
    let mut new_acc_id = use_signal(|| String::new());
    let mut new_acc_is_main = use_signal(|| true);

    // =========================================================================
    // 2. EFFECTS & DATA FETCHING
    // =========================================================================

    // Automatically bypass modal if token exists on startup
    use_effect(move || {
        if !auth_token.read().is_empty() {
            show_login_modal.set(false);
            
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

    // Fetch Official API Database
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

        div { class: "bg-gray-900 text-white min-h-screen p-6 font-sans relative",
            
            // --- TOP SECTIONS ---
            Header { auth_token, user_email, show_login_modal }
            
            FilterBar {
                search_query, selected_account_filter, collection,
                show_add_modal, show_account_modal, sync_status, auth_token,
                show_filter_menu
            }

            // --- THE VISUAL GRID ---
            CardGrid { collection, search_query, selected_account_filter, image_db, toast_message }
        }

        // --- OVERLAYS & MODALS ---
        // (Placed outside the main div so they overlay correctly)
        AddCardModal { show_add_modal, add_search_query, add_target_account, collection, image_db, toast_message }
        AccountModal { show_account_modal, new_acc_name, new_acc_id, new_acc_is_main, collection, toast_message }
        LoginModal { show_login_modal, user_email, user_password, auth_token, sync_status, collection }
        Toast { toast_message }
    }
}
