use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::models::optimized_image_url;

// --- 1. LOAD ASSETS AT COMPILE TIME ---
const DIAMOND_IMG: Asset = asset!("/assets/diamond.webp");
const STAR_IMG: Asset = asset!("/assets/star.webp");
const CROWN_IMG: Asset = asset!("/assets/crown.webp");
const SHINY_IMG: Asset = asset!("/assets/shiny-star.webp");

// --- 2. RARITY LOGIC & COMPONENT ---
pub fn get_rarity_data(code: &str) -> (&'static str, Option<Asset>, u8) {
    match code {
        "C" => ("Common", Some(DIAMOND_IMG), 1),
        "U" => ("Uncommon", Some(DIAMOND_IMG), 2),
        "R" => ("Rare", Some(DIAMOND_IMG), 3),
        "RR" => ("Double Rare", Some(DIAMOND_IMG), 4),
        "AR" => ("Art Rare", Some(STAR_IMG), 1),
        "SR" => ("Super Rare", Some(STAR_IMG), 2),
        "SAR" => ("Special Art Rare", Some(STAR_IMG), 2),
        "IM" => ("Immersive Rare", Some(STAR_IMG), 3),
        "UR" => ("Crown Rare", Some(CROWN_IMG), 1),
        "S" => ("Shiny", Some(SHINY_IMG), 1),
        "SSR" => ("Shiny Super Rare", Some(SHINY_IMG), 2),
        "PROMO" => ("Promo", None, 0),
        _ => ("Unknown", None, 0),
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct RarityDisplayProps {
    pub rarity_code: String,
}

#[component]
pub fn RarityDisplay(props: RarityDisplayProps) -> Element {
    let (label, image_asset, count) = get_rarity_data(&props.rarity_code);

    if count == 0 || image_asset.is_none() {
        return rsx! {
            span { class: "text-[10px] font-bold text-white", "{label}" }
        };
    }

    rsx! {
        div { class: "flex items-center justify-center gap-0.5", title: "{label}",
            for _ in 0..count {
                img { 
                    src: image_asset.clone().unwrap(),
                    class: "h-3.5 w-auto object-contain drop-shadow-md",
                    alt: "{label}"
                }
            }
        }
    }
}

// --- THE BUTTON ---
#[derive(PartialEq, Clone, Props)]
pub struct AddCardButtonProps {
    pub show_add_modal: Signal<bool>,
}

#[component]
pub fn AddCardButton(mut props: AddCardButtonProps) -> Element {
    rsx! {
        button {
            class: "group w-11 h-11 md:w-14 md:h-14 flex items-center justify-center bg-slate-800/60 border border-white/30/20 rounded-xl md:rounded-2xl hover:bg-slate-700/80 hover:border-white/40 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| props.show_add_modal.set(true),
            svg { class: "w-5 h-5 md:w-6 md:h-6 text-slate-400 group-hover:text-white transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
            }
        }
    }
}

// --- THE MODAL ---
#[derive(PartialEq, Clone, Props)]
pub struct AddCardModalProps {
    pub show_add_modal: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
    pub mass_select_mode: Signal<bool>,
    pub selected_mass_cards: Signal<Vec<String>>,
}

#[component]
pub fn AddCardModal(mut props: AddCardModalProps) -> Element {
    let mut selected_card_to_add = use_signal(|| None::<OfficialCard>);
    let mut show_account_list = use_signal(|| false);
    let mut search_input = use_signal(|| String::new());

    // Local filter state — isolated from the main collection filters
    let mut local_rarities = use_signal(|| Vec::<String>::new());
    let mut local_types   = use_signal(|| Vec::<String>::new());
    let mut show_local_filters = use_signal(|| false);

    // Memoized filtered results — live search as-you-type
    let filtered_api_cards = use_memo(move || {
        let query = search_input.read().trim().to_lowercase();

        if let Some(api_map) = &*props.image_db.read() {
            let selected_r = local_rarities.read().clone();
            let selected_t = local_types.read().clone();

            // Only show cards when a search/filter is active (something typed or filters chosen).
            // Mass-select also relies on this, so it never renders the full list.
            let has_filters = !query.is_empty() || !selected_r.is_empty() || !selected_t.is_empty();
            if !has_filters {
                return Vec::new();
            }

            let mut results: Vec<OfficialCard> = api_map.values()
                .filter(|c| {
                    let matches_q = query.is_empty() || c.name.to_lowercase().contains(&query);
                    let matches_r = selected_r.is_empty() || selected_r.contains(&c.rarity);
                    let matches_t = selected_t.is_empty() || selected_t.contains(&c.card_type);
                    matches_q && matches_r && matches_t
                })
                .cloned()
                .collect();
            results.sort_by(|a, b| a.set.cmp(&b.set).then(a.number.cmp(&b.number)));
            results
        } else {
            Vec::new()
        }
    });


    rsx! {
        if *props.show_add_modal.read() {
            // Semi-transparent overlay so the background collection is still visible
            div { class: "fixed inset-0 bg-slate-950/70 flex flex-col z-50 animate-fade-in-down backdrop-blur-sm",
                div { class: "glass-panel border-b border-white/10 p-3 flex flex-col gap-3 shadow-2xl z-10 backdrop-blur-2xl",
                    div { class: "flex justify-end",
                        button { 
                            class: "text-rose-400 hover:text-rose-300 p-2 rounded-lg hover:bg-rose-500/10 transition-all", 
                            title: "Close",
                            onclick: move |_| { props.show_add_modal.set(false); selected_card_to_add.set(None); show_account_list.set(false); }, 
                            svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        div { class: "relative group flex-1",
                            div { class: "absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none text-slate-500 group-focus-within:text-sky-400 transition-colors",
                                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-5 h-5",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                                }
                            }
                            input { 
                                class: "w-full bg-slate-900/80 border border-white/20 rounded-xl pl-10 pr-3 py-2 text-white placeholder-slate-500 focus:border-sky-400/50 focus:ring-1 focus:ring-sky-400/30 outline-none transition-colors shadow-inner text-sm", 
                                placeholder: if *props.mass_select_mode.read() { "Filter cards to bulk-select..." } else { "Search by name..." },
                                value: "{search_input}", 
                                oninput: move |evt| {
                                    search_input.set(evt.value());
                                    selected_card_to_add.set(None);
                                    show_account_list.set(false);
                                },
                            }
                        }

                        // Multi-select toggle
                        button {
                            class: "w-12 h-12 flex items-center justify-center border rounded-xl transition-all backdrop-blur-md cursor-pointer flex-shrink-0",
                            class: if *props.mass_select_mode.read() { "bg-emerald-500 border-emerald-400 text-white shadow-[0_0_15px_rgba(16,185,129,0.5)]" } else { "bg-slate-900/80 border-white/20 hover:bg-white/10 hover:border-white/30 text-slate-400 hover:text-white" },
                            title: if *props.mass_select_mode.read() { "Exit multi-select mode" } else { "Multi-select: pick many cards at once" },
                            onclick: move |_| {
                                let curr = *props.mass_select_mode.read();
                                props.mass_select_mode.set(!curr);
                                if curr { props.selected_mass_cards.set(Vec::new()); }
                            },
                            if *props.mass_select_mode.read() {
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2.5", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" }
                                }
                            } else {
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }
                                }
                            }
                        }

                        // Local filter button — does NOT affect the main collection view
                        button {
                            class: "w-12 h-12 flex items-center justify-center border rounded-xl transition-all backdrop-blur-md cursor-pointer flex-shrink-0",
                            class: if *show_local_filters.read() || !local_rarities.read().is_empty() || !local_types.read().is_empty() { "bg-pink-500/20 border-pink-500/50 text-pink-400 shadow-[0_0_12px_rgba(236,72,153,0.3)]" } else { "bg-slate-900/80 border-white/20 hover:bg-white/10 hover:border-white/30 text-slate-400 hover:text-pink-400" },
                            title: "Filter by rarity / type (local only)",
                            onclick: move |_| { let v = *show_local_filters.read(); show_local_filters.set(!v); },
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-5 h-5",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" }
                            }
                        }
                    }

                    // Local filter chips (does NOT touch global collection filters)
                    if *show_local_filters.read() {
                        div { class: "flex flex-col gap-2 pt-1 pb-1 animate-fade-in-down",
                            div { class: "flex flex-wrap gap-1.5 items-center",
                                span { class: "text-[10px] text-slate-500 uppercase font-bold tracking-widest mr-1", "Rarity" }
                                for rarity in ["C", "U", "R", "RR", "AR", "SR", "SAR", "IM", "UR", "S", "SSR", "PROMO"] {
                                    {
                                        let r = rarity.to_string();
                                        let is_active = local_rarities.read().contains(&r);
                                        rsx! {
                                            button {
                                                class: "px-2.5 py-1 rounded-lg border transition-all flex items-center",
                                                class: if is_active { "bg-indigo-500/30 border-indigo-400/60" } else { "bg-slate-800/60 border-white/10 hover:border-white/30" },
                                                onclick: move |_| {
                                                    let mut v = local_rarities.read().clone();
                                                    if v.contains(&r) { v.retain(|x| x != &r); } else { v.push(r.clone()); }
                                                    local_rarities.set(v);
                                                },
                                                RarityDisplay { rarity_code: r.clone() }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "flex flex-wrap gap-1.5 items-center",
                                span { class: "text-[10px] text-slate-500 uppercase font-bold tracking-widest mr-1", "Type" }
                                for card_type in ["Pokémon", "Trainer"] {
                                    {
                                        let t = card_type.to_string();
                                        let is_active = local_types.read().contains(&t);
                                        rsx! {
                                            button {
                                                class: "px-2.5 py-1 rounded-lg text-[11px] font-bold border transition-all",
                                                class: if is_active { "bg-emerald-500/30 border-emerald-400/60 text-emerald-300" } else { "bg-slate-800/60 border-white/10 text-slate-400 hover:border-white/30 hover:text-white" },
                                                onclick: move |_| {
                                                    let mut v = local_types.read().clone();
                                                    if v.contains(&t) { v.retain(|x| x != &t); } else { v.push(t.clone()); }
                                                    local_types.set(v);
                                                },
                                                "{card_type}"
                                            }
                                        }
                                    }
                                }
                                if !local_rarities.read().is_empty() || !local_types.read().is_empty() {
                                    button {
                                        class: "px-2.5 py-1 rounded-lg text-[11px] font-bold border bg-rose-500/10 border-rose-500/30 text-rose-400 hover:bg-rose-500/20 transition-all",
                                        onclick: move |_| { local_rarities.set(Vec::new()); local_types.set(Vec::new()); },
                                        "✕ Clear filters"
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "flex-1 overflow-y-auto p-4 relative",
                    if let Some(card) = selected_card_to_add.read().clone() {
                        {
                            let packs_text = if card.packs.is_empty() { "Promo".to_string() } else { card.packs.join(", ") };
                            
                            rsx! {
                                // Side-by-side layout: card on left, actions on right
                                div { class: "flex flex-col md:flex-row items-center md:items-start justify-center gap-6 md:gap-10 h-full max-w-2xl mx-auto py-4",
                                    
                                    // LEFT: Card Image + Metadata
                                    div { class: "flex flex-col items-center gap-3 flex-shrink-0",
                                        img { 
                                            src: "{optimized_image_url(&card.full_image_url, 600)}", 
                                            class: "w-32 md:w-56 rounded-2xl border border-white/30/20 shadow-2xl transition-all" 
                                        }
                                        
                                        // Card info below image
                                        div { class: "flex flex-col items-center gap-1.5 mt-1",
                                            h3 { class: "text-lg font-bold text-white text-center", "{card.name}" }
                                            RarityDisplay { rarity_code: card.rarity.clone() }
                                            span { class: "text-slate-500 text-xs font-medium tracking-wide", "{card.set} • {packs_text}" }
                                        }
                                    }

                                    // RIGHT: Actions
                                    div { class: "flex flex-col gap-3 w-full md:w-64 md:pt-2",
                                        {
                                            let is_wishlisted = props.collection.read().is_wishlisted(&card.generated_id);
                                            let c = card.clone();
                                            rsx! {
                                                button {
                                                    class: "w-full py-2.5 px-3 md:py-3.5 md:px-4 rounded-xl font-medium flex items-center justify-center gap-2 transition-all active:scale-[0.97] group backdrop-blur-sm mb-1",
                                                    class: if is_wishlisted { "bg-pink-500/20 text-pink-400 border border-pink-500/30 hover:bg-pink-500/30" } else { "bg-slate-800/60 text-slate-300 border border-slate-600/30 hover:bg-slate-700/80 hover:text-white" },
                                                    onclick: move |_| {
                                                        let card_to_add = Card { 
                                                            id: c.generated_id.clone(), 
                                                            name: c.name.clone(), 
                                                            rarity: c.rarity.clone(),
                                                            card_type: c.card_type.clone(), 
                                                            pack: if c.packs.is_empty() { "Promo".to_string() } else { c.packs.join(", ") }
                                                        };
                                                        props.collection.write().toggle_wishlist(card_to_add);
                                                    },
                                                    svg { class: "w-5 h-5 flex-shrink-0 transition-all", class: if is_wishlisted { "fill-pink-400 text-pink-400" } else { "fill-none text-slate-400 group-hover:text-pink-400" }, view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                                    }
                                                    span { class: "text-sm", if is_wishlisted { "Wishlisted" } else { "Add to Wishlist" } }
                                                }
                                            }
                                        }

                                        if !*show_account_list.read() {
                                            button {
                                                class: "w-full py-2.5 px-3 md:py-3.5 md:px-4 bg-white/30 hover:bg-white text-white rounded-xl font-medium flex items-center justify-center gap-2 transition-all active:scale-[0.97] shadow-lg shadow-white/30/20",
                                                onclick: move |_| show_account_list.set(true),
                                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                                                }
                                                span { class: "text-sm", "Add Card" }
                                            }
                                        }

                                        if *show_account_list.read() {
                                            div { class: "flex flex-col animate-fade-in-down w-full",
                                                div { class: "flex items-center justify-between mb-2 mt-1",
                                                    span { class: "text-[10px] text-slate-500 uppercase font-black tracking-widest", "Select Account" }
                                                    button {
                                                        class: "text-[10px] text-rose-400 hover:text-rose-300 hover:bg-rose-500/10 transition-all px-2 py-1 rounded-lg",
                                                        onclick: move |_| show_account_list.set(false),
                                                        "Cancel"
                                                    }
                                                }
                                                div { class: "flex flex-col gap-2",
                                                    for acc in props.collection.read().accounts.iter() {
                                                        {
                                                            let c = card.clone(); 
                                                            let target_acc = acc.name.clone();
                                                            let is_main = acc.main;
                                                            rsx! {
                                                                button {
                                                                    class: "w-full py-2.5 px-3 md:py-3 md:px-4 bg-slate-950/80 hover:bg-slate-900/90 border border-white/20 hover:border-white/40 rounded-xl text-white font-medium flex items-center gap-3 transition-all active:scale-[0.97] group backdrop-blur-2xl",
                                                                    onclick: move |_| {
                                                                        let card_to_add = Card { 
                                                                            id: c.generated_id.clone(), 
                                                                            name: c.name.clone(), 
                                                                            rarity: c.rarity.clone(),
                                                                            card_type: c.card_type.clone(), 
                                                                            pack: if c.packs.is_empty() { "Promo".to_string() } else { c.packs.join(", ") }
                                                                        };
                                                                        props.collection.write().add_card(card_to_add, &target_acc, 1);
                                                                        
                                                                        props.toast_message.set(Some(format!("Added to {}", target_acc)));
                                                                        let mut t = props.toast_message.clone(); 
                                                                        spawn(async move { 
                                                                            gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await; 
                                                                            t.set(None); 
                                                                        });
                                                                        selected_card_to_add.set(None);
                                                                        show_account_list.set(false);
                                                                    },
                                                                    // Account icon
                                                                    div { class: "w-8 h-8 rounded-lg bg-slate-700/60 group-hover:bg-white/30/20 flex items-center justify-center transition-colors flex-shrink-0",
                                                                        span { class: "text-xs", if is_main { "⭐" } else { "👤" } }
                                                                    }
                                                                    div { class: "flex flex-col items-start",
                                                                        span { class: "text-sm font-semibold group-hover:text-white transition-colors", "{acc.name}" }
                                                                    }
                                                                    // Arrow icon on the right
                                                                    svg { class: "w-4 h-4 text-slate-600 group-hover:text-white ml-auto transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        button { 
                                            class: "text-slate-500 hover:text-rose-400 text-sm mt-4 py-1.5 transition-all border border-slate-700/50 hover:border-rose-500/30 rounded-xl", 
                                            onclick: move |_| {
                                                selected_card_to_add.set(None);
                                                show_account_list.set(false);
                                            }, 
                                            "← Back to results" 
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 gap-3",
                            for api_card in filtered_api_cards() {
                                {
                                    let c = api_card.clone();
                                    let packs_display = if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") };

                                    let is_selected = props.selected_mass_cards.read().contains(&c.generated_id);
                                    rsx! {
                                        div { 
                                            class: "bg-slate-800/60 border rounded-xl p-2 cursor-pointer transition-all flex flex-col backdrop-blur-sm relative",
                                            class: if is_selected { "border-indigo-400 ring-2 ring-indigo-400/50 bg-indigo-900/40" } else { "border-white/30/15 hover:border-white/30/50 hover:bg-slate-800/80" },
                                            onclick: move |_| {
                                                if *props.mass_select_mode.read() {
                                                    let mut current = props.selected_mass_cards.read().clone();
                                                    if current.contains(&c.generated_id) {
                                                        current.retain(|id| id != &c.generated_id);
                                                    } else {
                                                        current.push(c.generated_id.clone());
                                                    }
                                                    props.selected_mass_cards.set(current);
                                                } else {
                                                    selected_card_to_add.set(Some(c.clone()));
                                                }
                                            },
                                            // Selection indicator
                                            if is_selected {
                                                div { class: "absolute top-3 right-3 w-6 h-6 rounded-full border-2 flex items-center justify-center shadow-md z-20 bg-indigo-500 border-indigo-300",
                                                    svg { class: "w-4 h-4 text-white", fill: "none", view_box: "0 0 24 24", stroke_width: "3", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" } }
                                                }
                                            }
                                            img { 
                                                src: "{optimized_image_url(&api_card.full_image_url, 400)}", 
                                                loading: "lazy", decoding: "async",
                                                width: "400", height: "560",
                                                class: "w-full rounded-lg mb-2 shadow-sm border border-white/30/10 aspect-[63/88] object-cover" 
                                            }
                                            h2 { class: "text-[11px] font-bold text-center text-slate-200 truncate", "{api_card.name}" }
                                            
                                            div { class: "mt-1 mb-1",
                                                RarityDisplay { rarity_code: api_card.rarity.clone() }
                                            }
                                            
                                            span { class: "text-[9px] text-center text-slate-500", "{api_card.set} • {packs_display}" }
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
}
