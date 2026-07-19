use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::models::optimized_image_url;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct CardGridProps {
    pub collection: Signal<CardCollection>,
    pub search_query: Signal<String>,
    pub selected_account_filter: Signal<String>,
    pub selected_rarities: Signal<Vec<String>>,
    pub selected_types: Signal<Vec<String>>,
    pub selected_packs: Signal<Vec<String>>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub selected_card_id: Signal<Option<String>>,
    pub active_view: Signal<String>,
    pub current_view_cards: Signal<Vec<String>>,
    pub mass_select_mode: Signal<bool>,
    pub selected_mass_cards: Signal<Vec<String>>,
}

fn get_rarity_priority(rarity_code: &str) -> usize {
    match rarity_code {
        "UR" => 1,
        "SSR" => 2,
        "S" => 3,
        "IM" => 4,
        "SAR" => 5,
        "SR" => 6,
        "AR" => 7,
        "RR" => 8,
        "R" => 9,
        "U" => 10,
        "C" => 11,
        _ => 12,
    }
}

#[component]
pub fn CardGrid(mut props: CardGridProps) -> Element {
    let query = props.search_query.read().to_lowercase();
    let selected_acc = props.selected_account_filter.read().clone();
    let active_rarities = props.selected_rarities.read().clone();
    let active_types = props.selected_types.read().clone();
    let active_packs = props.selected_packs.read().clone();
    let active_view_val = props.active_view.read().clone();

    // Borrow image_db (no clone of the whole map) so we always have the latest state
    let image_db = props.image_db.read();

    let col = props.collection.read();

    let base_cards: Vec<Card> = if active_view_val.starts_with("pack:") {
        let pack_code = active_view_val.trim_start_matches("pack:");
        if let Some(api_map) = image_db.as_ref() {
            api_map.values()
                .filter(|c| c.set == pack_code)
                .map(|c| Card {
                    id: c.generated_id.clone(),
                    name: c.name.clone(),
                    rarity: c.rarity.clone(),
                    card_type: c.card_type.clone(),
                    pack: c.set.clone(),
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        match active_view_val.as_str() {
            "wishlist" => col.wishlist.clone(),
            "tradable" => {
                col.inventory.iter()
                    .filter(|e| col.is_tradable(&e.card.id))
                    .filter(|e| selected_acc == "All" || e.owners.contains_key(&selected_acc))
                    .map(|e| e.card.clone())
                    .collect()
            },
            _ => { // "collection"
                col.inventory.iter()
                    .filter(|e| selected_acc == "All" || e.owners.contains_key(&selected_acc))
                    .map(|e| e.card.clone())
                    .collect()
            }
        }
    };

    let mut visible_cards: Vec<Card> = base_cards.into_iter().filter(|card| {
        // Search query applies to every view (collection, wishlist, tradable, pack)
        let matches_search = query.is_empty() || card.name.to_lowercase().contains(&query);
        let matches_rarity = active_rarities.is_empty() || active_rarities.contains(&card.rarity);
        
        let matches_pack = if active_packs.is_empty() {
            true
        } else {
            let card_set = if let Some(api_map) = image_db.as_ref() {
                api_map.get(&card.id).map(|c| c.set.clone()).unwrap_or_default()
            } else {
                String::new()
            };
            active_packs.contains(&card_set) || active_packs.contains(&card.pack)
        };

        let matches_type = if active_types.is_empty() {
            true
        } else {
            // Look up type from the image_db if inventory card doesn't have it
            let card_type = if card.card_type.is_empty() {
        if let Some(api_map) = image_db.as_ref() {
                    api_map.get(&card.id).map(|c| c.card_type.clone()).unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                card.card_type.clone()
            };
            active_types.iter().any(|t| card_type == *t)
        };
        matches_search && matches_rarity && matches_type && matches_pack
    }).collect();

    visible_cards.sort_by(|a, b| {
        get_rarity_priority(&a.rarity).cmp(&get_rarity_priority(&b.rarity))
    });

    // Sync visible cards sequence for swiping in the detail modal
    let visible_ids: Vec<String> = visible_cards.iter().map(|c| c.id.clone()).collect();
    if visible_ids != *props.current_view_cards.read() {
        let mut cvc = props.current_view_cards.clone();
        spawn(async move {
            cvc.set(visible_ids);
        });
    }

    rsx! {
        div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-1 sm:gap-2 md:gap-4 px-1",
            for (i, card) in visible_cards.into_iter().enumerate() {
                {
                    // Clone the ID up front so both closures can own their own copy
                    let card_id_for_click = card.id.clone();

                    let image_url = if let Some(api_map) = image_db.as_ref() {
                        api_map.get(&card.id).map(|c| optimized_image_url(&c.full_image_url, 400))
                    } else { None };

                    let card_name = card.name.clone();
                    let rarity_code = card.rarity.clone();

                    let is_wishlisted = col.is_wishlisted(&card_id_for_click);
                    let is_tradable = col.is_tradable(&card_id_for_click);
                    let is_owned = col.inventory.iter().any(|e| e.card.id == card_id_for_click);
                    
                    let is_high_rarity = get_rarity_priority(&rarity_code) <= 5; // SAR, IM, S, SSR, UR
                    
                    let op = if active_view_val.starts_with("pack:") && !is_owned {
                        "opacity-50 grayscale-[0.8] hover:grayscale-[0.5] hover:opacity-80"
                    } else {
                        "opacity-100 grayscale-0"
                    };

                    rsx! {
                        div {
                            key: "{card_id_for_click}",
                            class: "relative",
                            style: "overflow: visible; padding: 20px 0;",
                            div {
                                class: "js-tilt w-full h-full cursor-pointer relative group hover:-translate-y-2 hover:scale-[1.03] hover:z-20 transition-all duration-300 {op}",
                                style: "transform-style: preserve-3d; transform-origin: center center;",
                                onclick: move |_| {
                                if *props.mass_select_mode.read() {
                                    let mut curr = props.selected_mass_cards.read().clone();
                                    if let Some(pos) = curr.iter().position(|id| id == &card_id_for_click) {
                                        curr.remove(pos);
                                    } else {
                                        curr.push(card_id_for_click.clone());
                                    }
                                    props.selected_mass_cards.set(curr);
                                } else {
                                    props.selected_card_id.set(Some(card_id_for_click.clone()));
                                }
                            },
                            {
                                let is_selected_for_mass = *props.mass_select_mode.read() && props.selected_mass_cards.read().contains(&card_id_for_click);
                                let bg_border_class = if is_selected_for_mass {
                                    "bg-indigo-900/40 border border-indigo-400 ring-2 ring-indigo-400/50 shadow-[0_8px_24px_rgba(99,102,241,0.4)]"
                                } else {
                                    "bg-slate-900/20 border border-white/10 shadow-[0_4px_16px_rgba(0,0,0,0.1)] group-hover:border-teal-400/80 group-hover:shadow-[0_0_0_1px_rgba(45,212,191,0.75),0_8px_24px_rgba(45,212,191,0.25)]"
                                };

                                rsx! {
                                    div { class: "card-3d-element rounded-xl p-2.5 flex flex-col items-center h-full relative transition-all duration-500 overflow-visible {bg_border_class}",
                                          style: "transform: translateZ(0);",
                                        if is_high_rarity {
                                            div { class: "holo-shimmer" }
                                        }
                                        div { class: "absolute top-3 right-3 flex flex-col gap-1.5 z-20",
                                            if *props.mass_select_mode.read() {
                                                {
                                                    let is_selected = props.selected_mass_cards.read().contains(&card_id_for_click);
                                                    rsx! {
                                                        div {
                                                            class: "w-6 h-6 rounded-full border-2 flex items-center justify-center shadow-md transition-all",
                                                            class: if is_selected { "bg-indigo-500 border-indigo-300" } else { "bg-slate-900/80 border-white/30 backdrop-blur-sm" },
                                                            if is_selected {
                                                                svg { class: "w-4 h-4 text-white", fill: "none", view_box: "0 0 24 24", stroke_width: "3", stroke: "currentColor",
                                                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                if is_wishlisted {
                                                    div { class: "w-5 h-5 rounded-full bg-pink-500/90 border border-pink-400 flex items-center justify-center shadow-md",
                                                        svg { class: "w-3 h-3 text-white fill-current", view_box: "0 0 24 24",
                                                            path { d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                                        }
                                                    }
                                                }
                                                if is_tradable {
                                                    div { class: "w-5 h-5 rounded-full bg-emerald-500/90 border border-emerald-400 flex items-center justify-center shadow-md",
                                                        svg { class: "w-3 h-3 text-white", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if let Some(url) = image_url {
                                            {
                                                rsx! {
                                                    img { 
                                                        src: "{url}", 
                                                        alt: "{card_name}", 
                                                        loading: "lazy", decoding: "async",
                                                        width: "400", height: "560",
                                                        class: "w-full rounded-xl mb-3 shadow-md border border-white/5 aspect-[63/88] object-cover relative z-10",
                                                        style: "transform: translateZ(30px);"
                                                    } 
                                                }
                                            }
                                        } else {
                                            div { class: "w-full aspect-[63/88] bg-slate-800/50 rounded-lg mb-3 border border-white/5 animate-pulse relative z-10" }
                                        }
                                        h2 { class: "font-semibold text-[10px] md:text-xs text-center truncate w-full text-slate-200 group-hover:text-white transition-colors tracking-tight relative z-10", "{card_name}" }
                                        div { class: "mt-auto pt-2 relative z-10", style: "transform: translateZ(15px);",
                                            RarityDisplay { rarity_code }
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
        
        if props.collection.read().inventory.is_empty() && active_view_val == "collection" {
            div { class: "flex flex-col items-center justify-center mt-20 text-slate-500",
                svg { class: "w-12 h-12 mb-3 opacity-50", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" }
                }
                span { class: "text-sm font-medium", "No cards found." }
            }
        } else if props.collection.read().wishlist.is_empty() && active_view_val == "wishlist" {
            div { class: "flex flex-col items-center justify-center mt-20 text-slate-500",
                svg { class: "w-12 h-12 mb-3 opacity-50 text-pink-400", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                }
                span { class: "text-sm font-medium", "Your wishlist is empty." }
            }
        }
    }
}
