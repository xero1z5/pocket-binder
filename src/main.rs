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
    add_card::AddCardModal,
    account::AccountModal,
    trade::TradeModal,
    hamburger::HamburgerMenu,
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

    // sync engine flags
    let mut data_loaded = use_signal(|| false);
    let mut save_version = use_signal(|| 0u64);      // bumped on every collection change
    let mut last_saved_version = use_signal(|| 0u64); // tracks what we last saved
    
    // UI Toggles & Search
    let mut search_query = use_signal(|| String::new());
    let mut selected_account_filter = use_signal(|| String::from("All"));
    let mut selected_rarities = use_signal(|| Vec::<String>::new());
    let mut selected_types = use_signal(|| Vec::<String>::new());
    let mut show_add_modal = use_signal(|| false);
    let mut show_filter_menu = use_signal(|| false);
    let mut selected_card_id = use_signal(|| None::<String>);

    let mut show_trade_modal = use_signal(|| false);

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
    // EFFECTS & DATA FETCHING
    // =========================================================================

    // Initial Database Load
    use_effect(move || {
        let token = auth_token.read().clone();
        if !token.is_empty() {
            show_login_modal.set(false);
            spawn(async move {
                sync_status.set("🔄 Syncing collection...".to_string());
                if let Ok(data) = load_from_supabase(&token).await {
                    collection.set(data);
                    sync_status.set("✅ Cloud Synced!".to_string());
                } else {
                    sync_status.set("❌ Session expired. Please log in again.".to_string());
                    show_login_modal.set(true);
                }
                // Wait 100ms before unlocking so the initial load doesn't trigger a re-save
                gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                data_loaded.set(true); 
            });
        }
    });

    // Change Tracker: bump save_version whenever the collection changes
    use_effect(move || {
        // By calling .read(), Dioxus will run this block EVERY time you add/remove a card
        let _ = collection.read(); 
        
        // If the initial load is finished, flag that we have unsaved UI changes
        if *data_loaded.peek() {
            *save_version.write() += 1;
        }
    });

    // Trailing-Edge Debounced Saver: waits 800ms of inactivity before saving
    use_effect(move || {
        let version_at_trigger = *save_version.read();

        // Don't save if nothing has changed since last save
        if version_at_trigger == 0 || version_at_trigger == *last_saved_version.peek() {
            return;
        }

        spawn(async move {
            // Wait 800ms to batch rapid-fire clicks into a single save
            gloo_timers::future::sleep(std::time::Duration::from_millis(800)).await;

            // Check if any NEW changes happened during the wait
            let current_version = *save_version.peek();
            if current_version != version_at_trigger {
                // A newer change came in — that change's own effect will handle saving
                return;
            }

            // No new changes during the wait — save now!
            sync_status.set("⏳ Syncing...".to_string());
            
            let current_data = collection.peek().clone();
            let token = auth_token.peek().clone();

            if save_to_supabase(current_data, token).await.is_ok() {
                sync_status.set("✅ Cloud Synced!".to_string());
                last_saved_version.set(current_version);
            } else {
                sync_status.set("❌ Sync Failed! Retrying...".to_string());
                // Bump version to retrigger the saver
                *save_version.write() += 1;
            }
        });
    });

    // Fetch Official API Database with LocalStorage caching (stale-while-revalidate)
    let image_db = use_resource(move || async move {
        let cache_key = "cached_card_db";
        
        // 1. Try to load from LocalStorage cache instantly
        let cached: Option<Vec<OfficialCard>> = LocalStorage::get(cache_key).ok();
        
        // 2. Build the HashMap from cache if available
        let base_image_url = "https://raw.githubusercontent.com/flibustier/pokemon-tcg-exchange/main/public/images/cards-by-set";
        
        let build_db = |cards: Vec<OfficialCard>| -> HashMap<String, OfficialCard> {
            let mut api_db: HashMap<String, OfficialCard> = HashMap::new();
            for mut card in cards {
                card.full_image_url = format!("{}/{}/{}.webp", base_image_url, card.set, card.number);
                card.generated_id = format!("{}-{}", card.set, card.number);
                // Derive card type from image filename prefix
                card.card_type = if card.image.starts_with("cTR") {
                    "Trainer".to_string()
                } else {
                    "Pokémon".to_string()
                };
                api_db.insert(card.generated_id.clone(), card);
            }
            api_db
        };
        
        if let Some(cards) = cached {
            // Return cached data immediately, then refresh in background
            let db = build_db(cards);
            
            // Background refresh: fetch latest and update cache silently
            spawn(async move {
                let url = "https://cdn.jsdelivr.net/npm/pokemon-tcg-pocket-database/dist/cards.json";
                if let Ok(response) = reqwest::get(url).await {
                    if let Ok(fresh_cards) = response.json::<Vec<OfficialCard>>().await {
                        let _ = LocalStorage::set(cache_key, &fresh_cards);
                    }
                }
            });
            
            Some(db)
        } else {
            // First ever load: must fetch from network
            let url = "https://cdn.jsdelivr.net/npm/pokemon-tcg-pocket-database/dist/cards.json";
            let response = reqwest::get(url).await.ok()?;
            let official_cards = response.json::<Vec<OfficialCard>>().await.ok()?;
            
            // Cache for next time
            let _ = LocalStorage::set(cache_key, &official_cards);
            
            Some(build_db(official_cards))
        }
    });

    // =========================================================================
    // 3. UI LAYOUT
    // =========================================================================
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" }

        // pre warm the network connections
        document::Link { rel: "preconnect", href: "https://wsrv.nl" }
        document::Link { rel: "preconnect", href: "https://raw.githubusercontent.com" }
        document::Link { rel: "preconnect", href: "https://cdn.jsdelivr.net" }

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-slate-950 bg-cyber-grid text-white min-h-screen font-sans relative overflow-x-hidden",
            
            // --- TOP SECTIONS ---
            Header {}
            
            // --- MODULAR ACTION BAR ---
            div { class: "flex flex-col w-full mb-8 relative px-2 md:px-0",
                div { class: "flex flex-col md:flex-row justify-between items-end gap-4 w-full",
                    
                    // LEFT: Search Input
                    SearchInput { search_query }

                    // RIGHT: Filter button + Hamburger + Sync Status
                    div { class: "flex flex-col items-end gap-2 w-full md:w-auto",
                        
                        // Sync Status
                        div { class: "h-4 flex items-center pr-1",
                            if !sync_status.read().is_empty() {
                                span { class: "text-[10px] text-gray-400 font-mono tracking-widest", "{sync_status}" }
                            }
                        }

                        // Action Buttons: Filter + Hamburger only
                        div { class: "flex items-center gap-2 md:gap-3",
                            FilterButton { show_filter_menu }
                            HamburgerMenu {
                                show_account_modal,
                                show_add_modal,
                                show_trade_modal,
                                auth_token,
                                user_email,
                                show_login_modal,
                                collection,
                                sync_status,
                            }
                        }
                    }
                }
                
                // The Dropdown Filters Tray
                FilterTray { show_filter_menu, selected_account_filter, selected_rarities, selected_types, collection }
            }

            // --- THE VISUAL GRID ---
            CardGrid { collection, search_query, selected_account_filter, selected_rarities, selected_types, image_db, selected_card_id }
        }

        // --- OVERLAYS & MODALS ---
        AddCardModal { show_add_modal, collection, image_db, toast_message }
        AccountModal { show_account_modal, new_acc_name, new_acc_id, new_acc_is_main, collection, toast_message }
        LoginModal { show_login_modal, user_email, user_password, auth_token, sync_status, collection }

        CardDetailModal { selected_card_id, collection, image_db, toast_message }
        TradeModal { show_trade_modal, collection, image_db, toast_message }
        Toast { toast_message }
    }

        
}
