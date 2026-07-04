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
    let mut collection = use_signal(|| CardCollection { accounts: Vec::new(), inventory: Vec::new(), wishlist: Vec::new(), tradable: Vec::new() });
    let mut sync_status = use_signal(|| String::new());

    // sync engine flags
    let mut data_loaded = use_signal(|| false);
    let mut save_version = use_signal(|| 0u64);      // bumped on every collection change
    let mut last_saved_version = use_signal(|| 0u64); // tracks what we last saved
    
    // UI Toggles & Search
    let mut active_view = use_signal(|| "collection".to_string());
    let mut selected_account_filter = use_signal(|| String::from("All"));
    let mut selected_rarities = use_signal(|| Vec::<String>::new());
    let mut selected_types = use_signal(|| Vec::<String>::new());
    let mut selected_packs = use_signal(|| Vec::<String>::new());
    let mut search_query = use_signal(|| String::new());
    let mut show_add_modal = use_signal(|| false);
    let mut show_filter_menu = use_signal(|| false);
    let mut show_hamburger_menu = use_signal(|| false);
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

    use_effect(move || {
        let any_modal_open = *show_add_modal.read() || 
            *show_filter_menu.read() || 
            selected_card_id.read().is_some() || 
            *show_trade_modal.read() || 
            *show_login_modal.read() || 
            *show_account_modal.read() ||
            *show_hamburger_menu.read();

        if any_modal_open {
            let _ = document::eval("document.body.style.overflow = 'hidden'");
        } else {
            let _ = document::eval("document.body.style.overflow = ''");
        }
    });

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
    let mut image_db = use_signal(|| None::<HashMap<String, OfficialCard>>);
    let mut pack_db = use_signal(|| None::<Vec<PackSet>>);

    use_effect(move || {
        spawn(async move {
            let cache_key = "cached_card_db";
            let pack_cache_key = "cached_pack_db";
            
            // 1. Try to load from LocalStorage cache instantly
            let cached: Option<Vec<OfficialCard>> = LocalStorage::get(cache_key).ok();
            let cached_packs: Option<HashMap<String, Vec<PackSet>>> = LocalStorage::get(pack_cache_key).ok();
            
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
                image_db.set(Some(build_db(cards)));
            }
            if let Some(pack_map) = cached_packs {
                let mut flattened: Vec<PackSet> = pack_map.into_values().flatten().collect();
                flattened.sort_by(|a, b| b.release_date.cmp(&a.release_date));
                pack_db.set(Some(flattened));
            }

            // Background refresh: fetch latest and update cache + UI
            // Unpkg handles @latest without caching forever, and timestamp bypasses browser cache
            let timestamp = js_sys::Date::now();
            let url = format!("https://unpkg.com/pokemon-tcg-pocket-database@latest/dist/cards.json?t={}", timestamp);
            let sets_url = format!("https://unpkg.com/pokemon-tcg-pocket-database@latest/dist/sets.json?t={}", timestamp);
            
            let fetch_cards = async {
                if let Ok(response) = reqwest::get(&url).await {
                    if let Ok(fresh_cards) = response.json::<Vec<OfficialCard>>().await {
                        let _ = LocalStorage::set(cache_key, &fresh_cards);
                        image_db.set(Some(build_db(fresh_cards)));
                    }
                }
            };
            
            let fetch_packs = async {
                if let Ok(response) = reqwest::get(&sets_url).await {
                    if let Ok(fresh_packs) = response.json::<HashMap<String, Vec<PackSet>>>().await {
                        let _ = LocalStorage::set(pack_cache_key, &fresh_packs);
                        let mut flattened: Vec<PackSet> = fresh_packs.into_values().flatten().collect();
                        flattened.sort_by(|a, b| b.release_date.cmp(&a.release_date));
                        pack_db.set(Some(flattened));
                    }
                }
            };
            
            fetch_cards.await;
            fetch_packs.await;
        });
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
                                is_open: show_hamburger_menu,
                                show_account_modal,
                                show_add_modal,
                                show_trade_modal,
                                auth_token,
                                user_email,
                                show_login_modal,
                                collection,
                                sync_status,
                                active_view,
                                pack_db,
                            }
                        }
                    }
                }
                
                // The Dropdown Filters Tray
                FilterTray { show_filter_menu, selected_account_filter, selected_rarities, selected_types, selected_packs, collection, pack_db }
            }

            // --- THE VISUAL GRID ---
            CardGrid { collection, search_query, selected_account_filter, selected_rarities, selected_types, selected_packs, image_db, selected_card_id, active_view }
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
