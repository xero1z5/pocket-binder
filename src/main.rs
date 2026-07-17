mod models;
mod supabase;
mod components;

use dioxus::prelude::*;
use std::collections::HashMap;
use gloo_storage::{LocalStorage, Storage};

use models::*;
use supabase::*;

use crate::components::{
    filter::{SearchInput, FilterButton, FilterTray},
    grid::CardGrid,
    add_card::AddCardModal,
    account::AccountModal,
    login::LoginModal,
    card_detail::CardDetailModal,
    trade::TradeModal,
    toast::Toast,
    navigation::FloatingDock,
    mass_action::MassActionBar,
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
    let mut active_view = use_signal(|| "collection".to_string()); // "collection", "wishlist", "tradable", or "pack:A1"
    let mut selected_account_filter = use_signal(|| String::from("All"));
    let mut selected_rarities = use_signal(|| Vec::<String>::new());
    let mut selected_types = use_signal(|| Vec::<String>::new());
    let mut selected_packs = use_signal(|| Vec::<String>::new());
    let mut search_query = use_signal(|| String::new());
    let mut show_add_modal = use_signal(|| false);
    let mut show_filter_menu = use_signal(|| false);
    let mut show_hamburger_menu = use_signal(|| false);
    let mut selected_card_id = use_signal(|| None::<String>);
    let mut current_view_cards = use_signal(|| Vec::<String>::new());
    let mut mass_select_mode = use_signal(|| false);
    let mut selected_mass_cards = use_signal(|| Vec::<String>::new());

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

    let any_modal_open = *show_add_modal.read() || 
        *show_filter_menu.read() || 
        selected_card_id.read().is_some() || 
        *show_trade_modal.read() || 
        *show_login_modal.read() || 
        *show_account_modal.read();

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

    // Auto-Migrate orphaned cards when collection or image_db changes
    use_effect(move || {
        if let Some(db) = &*image_db.read() {
            let mut col = collection.read().clone();
            if col.migrate_orphaned_cards(db) {
                // If migration actually changed something, save it
                collection.set(col.clone());
                let _ = LocalStorage::set("binder_state", &col);
                
                let token = auth_token.read().clone();
                if !token.is_empty() {
                    spawn(async move {
                        let _ = save_to_supabase(col, token).await;
                    });
                }
            }
        }
    });

    use_effect(move || {
        spawn(async move {
            let cache_key = "cached_card_db_v9";
            let pack_cache_key = "cached_pack_db_v9";
            
            // 1. Try to load from LocalStorage cache instantly
            let cached: Option<Vec<OfficialCard>> = LocalStorage::get(cache_key).ok();
            
            let build_db = |cards: Vec<OfficialCard>| -> HashMap<String, OfficialCard> {
                let mut api_db: HashMap<String, OfficialCard> = HashMap::new();
                for mut card in cards {
                    if card.generated_id.is_empty() {
                        card.generated_id = format!("{}-{:03}", card.set, card.number);
                    }
                    if card.full_image_url.is_empty() {
                        card.full_image_url = format!("https://assets.tcgdex.net/en/tcgp/{}/{:03}/high.webp", card.set, card.number);
                    }
                    api_db.insert(card.generated_id.clone(), card);
                }
                api_db
            };
            
            if let Some(cards) = cached {
                image_db.set(Some(build_db(cards)));
            }
            
            if let Some(packs) = LocalStorage::get::<Vec<PackSet>>(pack_cache_key).ok() {
                pack_db.set(Some(packs));
            }

            // ===================================================================
            // 2. Background refresh — Flibustier as PRIMARY source
            // ===================================================================
            let cdn_base = "https://unpkg.com/pokemon-tcg-pocket-database@latest/dist";
            
            // Sets that exist on TCGdex CDN (for high-quality images)
            let tcgdex_sets: Vec<&str> = vec![
                "P-A", "A1", "A1a", "A2", "A2a", "A2b", "A3", "A3a", "A3b",
                "A4", "A4a", "B1", "B1a", "B2", "B2a",
            ];
            
            // Normalize flibustier set IDs (PROMO-A → P-A) to a common format
            let normalize_set = |s: &str| -> String {
                match s {
                    "PROMO-A" => "P-A".to_string(),
                    "PROMO-B" => "P-B".to_string(),
                    other => other.to_string(),
                }
            };
            
            // --- Fetch cards from flibustier (primary, complete list) ---
            let cards_url = format!("{}/cards.json", cdn_base);
            let sets_url = format!("{}/sets.json", cdn_base);
            
            let cards_future = reqwest::get(&cards_url);
            let sets_future = reqwest::get(&sets_url);
            
            // Fetch both in parallel
            let (cards_result, sets_result) = (cards_future.await, sets_future.await);
            
            // Process cards
            if let Ok(cards_resp) = cards_result {
                if let Ok(flib_cards) = cards_resp.json::<Vec<FlibustierCard>>().await {
                    let all_cards: Vec<OfficialCard> = flib_cards.into_iter().map(|fc| {
                        let set_id = normalize_set(&fc.set);
                        let generated_id = format!("{}-{:03}", set_id, fc.number);
                        
                        // Use LimitlessTCG CDN for all images as requested (fastest updates for new sets)
                        let image_url = format!("https://limitlesstcg.nyc3.cdn.digitaloceanspaces.com/pocket/{}/{}_{:03}_EN.webp", set_id, set_id, fc.number);
                        
                        // Detect card type from the image name pattern
                        let card_type = if fc.image.starts_with("cTR") {
                            "Trainer".to_string()
                        } else {
                            "Pokémon".to_string()
                        };
                        
                        OfficialCard {
                            set: set_id,
                            number: fc.number,
                            name: fc.name,
                            rarity: fc.rarity,
                            packs: fc.packs,
                            image: fc.image,
                            card_type,
                            full_image_url: image_url,
                            generated_id,
                        }
                    }).collect();
                    
                    let _ = LocalStorage::set(cache_key, &all_cards);
                    image_db.set(Some(build_db(all_cards)));
                }
            }
            
            // Process sets
            if let Ok(sets_resp) = sets_result {
                if let Ok(sets_map) = sets_resp.json::<HashMap<String, Vec<FlibustierSet>>>().await {
                    let mut all_packs: Vec<PackSet> = Vec::new();
                    for (_series, sets) in &sets_map {
                        for fset in sets {
                            let code = normalize_set(&fset.code);
                            all_packs.push(PackSet {
                                code,
                                release_date: fset.release_date.clone(),
                                name: fset.name.clone(),
                                packs: fset.packs.clone(),
                            });
                        }
                    }
                    // Sort by release date (newest first)
                    all_packs.sort_by(|a, b| b.release_date.cmp(&a.release_date));
                    
                    let _ = LocalStorage::set(pack_cache_key, &all_packs);
                    pack_db.set(Some(all_packs));
                }
            }
        });
    });

    // =========================================================================
    // 3. UI LAYOUT
    // =========================================================================
    let vanilla_tilt_js = r#"
        // Use MutationObserver to initialize vanilla-tilt on new cards automatically
        const observer = new MutationObserver((mutations) => {
            let newCards = [];
            mutations.forEach((mutation) => {
                mutation.addedNodes.forEach((node) => {
                    if (node.nodeType === 1) { 
                        if (node.classList && node.classList.contains('js-tilt')) {
                            newCards.push(node);
                        }
                        if (node.querySelectorAll) {
                            const children = node.querySelectorAll('.js-tilt');
                            children.forEach(c => newCards.push(c));
                        }
                    }
                });
            });
            const isHoverDevice = window.matchMedia("(hover: hover)").matches;
            if (newCards.length > 0 && window.VanillaTilt && isHoverDevice) {
                window.VanillaTilt.init(newCards, {
                    max: 12,
                    speed: 400,
                    glare: true,
                    "max-glare": 0.35,
                    scale: 1.05
                });
            }
        });
        // Initialize observer when document is ready
        document.addEventListener("DOMContentLoaded", () => {
            observer.observe(document.body, { childList: true, subtree: true });
            // init any existing on load
            const isHoverDevice = window.matchMedia("(hover: hover)").matches;
            if (window.VanillaTilt && isHoverDevice) {
                window.VanillaTilt.init(document.querySelectorAll(".js-tilt"), {
                    max: 12, speed: 400, glare: true, "max-glare": 0.35, scale: 1.05
                });
            }
        });
    "#;

    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" }

        // pre warm the network connections
        document::Link { rel: "preconnect", href: "https://api.tcgdex.net" }
        document::Link { rel: "preconnect", href: "https://assets.tcgdex.net" }

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        if any_modal_open {
            style { "body {{ overflow: hidden !important; }}" }
        }

        // Forces dx serve reload for animations shrink fix
        div { class: "bg-[#0c0f1a] text-slate-200 min-h-screen font-sans relative overflow-x-hidden flex selection:bg-indigo-500/30",
            
            // Script injection for 3D Tilt
            script { src: "https://cdnjs.cloudflare.com/ajax/libs/vanilla-tilt/1.8.1/vanilla-tilt.min.js" }
            script { dangerous_inner_html: "{vanilla_tilt_js}" }

            
            // --- Ambient Animated Backgrounds ---
            div { class: "bg-blob bg-blob-1" }
            div { class: "bg-blob bg-blob-2" }
            div { class: "bg-blob bg-blob-3" }
            
            // --- Floating Particles ---
            div { class: "particles-container",
                for i in 0..50 {
                    div {
                        class: "particle",
                        style: format!("left: {}vw; animation-delay: -{}s; animation-duration: {}s; width: {}px; height: {}px;", 
                            (i as f32 * 7.33) % 100.0, 
                            (i as f32 * 1.7) % 15.0, 
                            8.0 + (i as f32 % 10.0),
                            2.0 + (i as f32 % 3.0),
                            2.0 + (i as f32 % 3.0)
                        )
                    }
                }
            }

            // --- Floating Action Bar (Top) ---
            div { class: "fixed top-4 left-1/2 -translate-x-1/2 w-[95%] max-w-4xl z-40 glass-panel rounded-2xl px-4 py-2 flex justify-between items-center shadow-lg border border-white/10 animate-fade-in-down backdrop-blur-xl",
                
                // LEFT: Search Input
                div { class: "w-full max-w-md",
                    SearchInput { search_query }
                }

                // RIGHT: Filter & Status
                div { class: "flex items-center gap-3",
                    if !sync_status.read().is_empty() {
                        {
                            let is_syncing = sync_status.read().contains("Syncing") || sync_status.read().contains("Downloading");
                            let is_error = sync_status.read().contains("Failed") || sync_status.read().contains("expired") || sync_status.read().contains("❌");
                            let color_class = if is_error { "text-rose-400" } else if is_syncing { "text-sky-400" } else { "text-emerald-400" };
                            
                            rsx! {
                                div {
                                    class: "flex items-center justify-center w-11 h-11 md:w-12 md:h-12 transition-all {color_class}",
                                    title: "{sync_status}",
                                    if is_syncing {
                                        svg { class: "w-5 h-5 md:w-6 md:h-6 animate-spin", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.016 4.356v4.992" }
                                        }
                                    } else {
                                        svg { class: "w-5 h-5 md:w-6 md:h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M2.25 15a4.5 4.5 0 004.5 4.5H18a3.75 3.75 0 001.332-7.257 3 3 0 00-3.758-3.848 5.25 5.25 0 00-10.233 2.33A4.502 4.502 0 002.25 15z" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    button {
                        class: "group w-11 h-11 md:w-12 md:h-12 flex items-center justify-center border rounded-xl md:rounded-2xl transition-all backdrop-blur-md cursor-pointer",
                        class: if *mass_select_mode.read() { "bg-emerald-500 border-emerald-400 text-white shadow-[0_0_15px_rgba(16,185,129,0.5)]" } else { "bg-white/5 border-white/10 hover:bg-white/10 hover:border-white/20 text-slate-400 hover:text-white" },
                        title: if *mass_select_mode.read() { "Exit multi-select mode" } else { "Multi-select: pick many cards at once" },
                        onclick: move |_| {
                            let curr = *mass_select_mode.read();
                            mass_select_mode.set(!curr);
                            if curr {
                                selected_mass_cards.set(Vec::new());
                            }
                        },
                        if *mass_select_mode.read() {
                            svg { class: "w-5 h-5 md:w-6 md:h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2.5", stroke: "currentColor",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" }
                            }
                        } else {
                            svg { class: "w-5 h-5 md:w-5 md:h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }
                            }
                        }
                    }
                    FilterButton { show_filter_menu }
                }
            }

            // --- Main Content Area ---
            div { 
                class: "w-full min-h-screen pt-20 pb-28 px-1 sm:px-2 md:px-4 relative z-10",
                
                // --- THE VISUAL GRID ---
                CardGrid { collection, search_query, selected_account_filter, selected_rarities, selected_types, selected_packs, image_db, selected_card_id, active_view, current_view_cards, mass_select_mode, selected_mass_cards }
            }

            // --- Floating Navigation Dock (Bottom) ---
            FloatingDock {
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

            // --- Filter Drawer ---
            FilterTray { show_filter_menu, selected_account_filter, selected_rarities, selected_types, selected_packs, collection, pack_db }
        }

        // --- OVERLAYS & MODALS ---
        AddCardModal { 
            show_add_modal, collection, image_db, toast_message,
            mass_select_mode: mass_select_mode.clone(),
            selected_mass_cards: selected_mass_cards.clone(),
        }
        AccountModal { show_account_modal, new_acc_name, new_acc_id, new_acc_is_main, collection, toast_message, auth_token, user_email }
        LoginModal { show_login_modal, user_email, user_password, auth_token, sync_status, collection }

        CardDetailModal { selected_card_id, collection, image_db, toast_message, current_view_cards }
        TradeModal { show_trade_modal, collection, image_db, toast_message }
        MassActionBar { selected_mass_cards, mass_select_mode, collection, image_db, toast_message }
        Toast { toast_message }
    }

        
}
