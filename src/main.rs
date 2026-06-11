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

    // sync engine flags
    let mut data_loaded = use_signal(|| false);
    let mut is_saving = use_signal(|| false);
    let mut pending_save = use_signal(|| false);
    
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

    // Change Tracker
    use_effect(move || {
        // By calling .read(), Dioxus will run this block EVERY time you add/remove a card
        let _ = collection.read(); 
        
        // If the initial load is finished, flag that we have unsaved UI changes
        if *data_loaded.peek() {
            pending_save.set(true); 
        }
    });

    // Sync Engine Queue
    use_effect(move || {
        // We read these flags to wake up the engine whenever they change
        let has_pending = *pending_save.read();
        let currently_saving = *is_saving.read();

        // If we have changes AND the network is free, start saving!
        if has_pending && !currently_saving {
            
            // Lock the network so we don't accidentally run two overlapping saves
            pending_save.set(false);
            is_saving.set(true);

            spawn(async move {
                // Wait a tiny 600ms to batch rapid-fire clicks into a single save
                // time to batch rapid-fire clicks to a single save
                gloo_timers::future::sleep(std::time::Duration::from_millis(250)).await;
                
                sync_status.set("⏳ Syncing...".to_string());
                
                // Grab a snapshot of exactly how the data looks RIGHT NOW
                let current_data = collection.peek().clone();
                let token = auth_token.peek().clone();

                // Send it to Supabase
                if save_to_supabase(current_data, token).await.is_ok() {
                    sync_status.set("✅ Cloud Synced!".to_string());
                } else {
                    sync_status.set("❌ Sync Failed! Retrying...".to_string());
                    pending_save.set(true); // If it fails, put it back in the queue
                }
                
                // Unlock the network so the next batch of changes can save
                is_saving.set(false);
            });
        }
    });

    // Fetch Official API Database (Updated to Flibustier via jsDelivr)
    let image_db = use_resource(move || async move {
        // 1. Fetch the JSON from the NPM package CDN
        let url = "https://cdn.jsdelivr.net/npm/pokemon-tcg-pocket-database/dist/cards.json";
        let response = reqwest::get(url).await.ok()?;
        let official_cards = response.json::<Vec<OfficialCard>>().await.ok()?;
        
        let mut api_db: HashMap<String, OfficialCard> = HashMap::new();
        
        // 2. Base URL for the actual WebP images hosted in the exchange repo
        let base_image_url = "https://raw.githubusercontent.com/flibustier/pokemon-tcg-exchange/main/public/images/cards-by-set";

        for mut card in official_cards {
            // Build the perfect image URL: .../cards-by-set/A1/1.webp
            card.full_image_url = format!("{}/{}/{}.webp", base_image_url, card.set, card.number);
            
            // Create a universal ID (e.g., "A1-1")
            card.generated_id = format!("{}-{}", card.set, card.number);
            
            api_db.insert(card.generated_id.clone(), card);
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
        document::Link { rel: "preconnect", href: "https://cdn.jsdelivr.net" }

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

                        // The Square Action Buttons
                        div { class: "flex items-center gap-2 md:gap-3",
        
                            // THE NEW "HEAL DATABASE" BUTTON
                            button {
                                class: "group w-11 h-11 md:w-14 md:h-14 flex items-center justify-center bg-purple-600/20 border border-purple-500/50 rounded-xl md:rounded-2xl hover:bg-purple-500 hover:border-purple-400 transition-all shadow-lg",
                                title: "Fix Broken Legacy Cards",
                                onclick: move |_| {
                                    if let Some(Some(api_map)) = &*image_db.read() {
                                        sync_status.set("🛠️ Fixing Cards...".to_string());
                                        let mut col = collection.write();
                                        let mut needs_save = false;

                                        for entry in col.inventory.iter_mut() {
                                            let mut found_api_card = api_map.get(&entry.card.id);
                                            
                                            // 1. If ID is broken, find the card by its exact name
                                            if found_api_card.is_none() {
                                                if let Some((_, matching_card)) = api_map.iter().find(|(_, c)| c.name.to_lowercase() == entry.card.name.to_lowercase()) {
                                                    entry.card.id = matching_card.generated_id.clone();
                                                    found_api_card = Some(matching_card);
                                                    needs_save = true;
                                                }
                                            }
                                            
                                            // 2. Update the old "☆☆" text to the new "SAR" code
                                            if let Some(api_card) = found_api_card {
                                                let new_pack = if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") };
                                                if entry.card.rarity != api_card.rarity || entry.card.pack != new_pack {
                                                    entry.card.rarity = api_card.rarity.clone();
                                                    entry.card.pack = new_pack;
                                                    needs_save = true;
                                                }
                                            }
                                        }

                                        // 3. Upload the fixed database to Supabase
                                        if needs_save {
                                            let current_collection = col.clone();
                                            let token = auth_token.read().clone();
                                            spawn(async move {
                                                if save_to_supabase(current_collection, token).await.is_ok() {
                                                    sync_status.set("✅ Cards Fixed & Synced!".to_string());
                                                }
                                            });
                                        } else {
                                            sync_status.set("✅ Cards already up to date.".to_string());
                                        }
                                    }
                                },
                                span { class: "text-xl group-hover:scale-110 transition-transform", "✨" }
                            }
                            FilterButton { show_filter_menu }
                            AccountButton { show_account_modal }
                            AddCardButton { show_add_modal }
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
