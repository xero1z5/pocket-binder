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
    grid::CardGrid,
    toast::Toast,
    login::LoginModal,
    card_detail::CardDetailModal,
    filter::{SearchInput, FilterButton, FilterTray},
    add_card::{AddCardButton, AddCardModal},
    account::{AccountButton, AccountModal},
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
    let mut selected_card_id = use_signal(|| None::<String>);
    let mut add_search_query = use_signal(|| String::new());

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
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" }

        // pre warm the network connections
        document::Link { rel: "preconnect", href: "https://wsrv.nl" }
        document::Link { rel: "preconnect", href: "https://raw.githubusercontent.com" }

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-gray-900 text-white min-h-screen p-6 font-sans relative",
            
            // --- TOP SECTIONS ---
            Header { auth_token, user_email, show_login_modal }
            
            // --- MODULAR ACTION BAR ---
            div { class: "flex flex-col w-full mb-8 relative",
                div { class: "flex flex-col md:flex-row justify-between items-end gap-4 w-full",
                    
                    // LEFT: Search Input
                    SearchInput { search_query }

                    // RIGHT: Action Buttons & Sync Status
                    div { class: "flex flex-col items-end gap-2 w-full md:w-auto",
                        
                        // Sync Status
                        div { class: "h-4 flex items-center pr-1",
                            if !sync_status.read().is_empty() {
                                span { class: "text-[10px] text-gray-400 font-mono tracking-widest", "{sync_status}" }
                            }
                        }

                        // The 4 Square Action Buttons
                        div { class: "flex items-center gap-2 md:gap-3",
                            FilterButton { show_filter_menu }
                            AccountButton { show_account_modal }
                            AddCardButton { show_add_modal }
                            
                            // Inline Cloud Sync Button
                            button {
                                class: "group w-11 h-11 md:w-14 md:h-14 flex flex-col items-center justify-center bg-blue-600/20 border border-blue-500/50 rounded-xl md:rounded-2xl hover:bg-blue-500 hover:border-blue-400 transition-all shadow-lg shadow-blue-900/20",
                                onclick: move |_| {
                                    sync_status.set("Syncing...".to_string());
                                    let current_collection = collection.read().clone();
                                    let token_to_use = auth_token.read().clone();
                                    
                                    spawn(async move {
                                        match save_to_supabase(current_collection, token_to_use).await {
                                            Ok(_) => sync_status.set("Last sync: Just now".to_string()),
                                            Err(_) => sync_status.set("Sync Failed!".to_string()),
                                        }
                                    });
                                },
                                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-5 h-5 md:w-6 md:h-6 text-blue-400 group-hover:text-white transition-colors",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 16.5V9.75m0 0l3 3m-3-3l-3 3M6.75 19.5a4.5 4.5 0 01-1.41-8.775 5.25 5.25 0 0110.233-2.33 3 3 0 013.758 3.848A3.752 3.752 0 0118 19.5H6.75z" }
                                }
                            }
                        }
                    }
                }
                
                // The Dropdown Filters Tray
                FilterTray { show_filter_menu, selected_account_filter, collection }
            }

            // --- THE VISUAL GRID ---
            CardGrid { collection, search_query, selected_account_filter, image_db, selected_card_id }
        }

        // --- OVERLAYS & MODALS ---
        AddCardModal { show_add_modal, add_search_query, collection, image_db, toast_message }
        AccountModal { show_account_modal, new_acc_name, new_acc_id, new_acc_is_main, collection, toast_message }
        LoginModal { show_login_modal, user_email, user_password, auth_token, sync_status, collection }

        CardDetailModal { selected_card_id, collection, image_db, toast_message }

        Toast { toast_message }
    }
}
