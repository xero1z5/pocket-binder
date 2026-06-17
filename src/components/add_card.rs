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
            span { class: "text-[10px] font-bold text-indigo-400", "{label}" }
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
            class: "group w-11 h-11 md:w-14 md:h-14 flex items-center justify-center bg-slate-800/60 border border-indigo-500/20 rounded-xl md:rounded-2xl hover:bg-slate-700/80 hover:border-indigo-400/40 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| props.show_add_modal.set(true),
            svg { class: "w-5 h-5 md:w-6 md:h-6 text-slate-400 group-hover:text-indigo-400 transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
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
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn AddCardModal(mut props: AddCardModalProps) -> Element {
    let mut selected_card_to_add = use_signal(|| None::<OfficialCard>);

    // Local raw input — filtering happens directly from this (no debounce needed,
    // the in-memory HashMap scan is fast enough)
    let mut raw_add_input = use_signal(|| String::new());

    // Memoized filtered results — reads raw_add_input directly for instant results
    let filtered_api_cards = use_memo(move || {
        let query = raw_add_input.read().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        if let Some(Some(api_map)) = &*props.image_db.read() {
            api_map.values()
                .filter(|c| c.name.to_lowercase().contains(&query))
                .take(30)
                .cloned()
                .collect::<Vec<OfficialCard>>()
        } else {
            Vec::new()
        }
    });

    rsx! {
        if *props.show_add_modal.read() {
            div { class: "fixed inset-0 bg-slate-950/90 flex flex-col z-50 animate-fade-in-down backdrop-blur-sm",
                div { class: "bg-slate-900/80 border-b border-indigo-500/20 p-4 pt-6 flex flex-col gap-4 shadow-xl z-10 backdrop-blur-xl",
                    div { class: "flex justify-between items-center",
                        h2 { class: "text-xl font-bold tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-purple-300", "Add Cards" }
                        button { 
                            class: "text-slate-500 hover:text-white p-2 transition-colors", 
                            onclick: move |_| { props.show_add_modal.set(false); selected_card_to_add.set(None); }, 
                            svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                        }
                    }
                    input { 
                        class: "bg-slate-900/80 border border-indigo-500/20 rounded-xl px-4 py-3 text-white placeholder-slate-500 focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none w-full transition-colors shadow-inner", 
                        placeholder: "Search card name...", 
                        value: "{raw_add_input}", 
                        oninput: move |evt| {
                            raw_add_input.set(evt.value());
                            selected_card_to_add.set(None);
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
                                            class: "w-44 md:w-56 rounded-2xl border border-indigo-500/20 shadow-2xl transition-all" 
                                        }
                                        
                                        // Card info below image
                                        div { class: "flex flex-col items-center gap-1.5 mt-1",
                                            h3 { class: "text-lg font-bold text-white text-center", "{card.name}" }
                                            RarityDisplay { rarity_code: card.rarity.clone() }
                                            span { class: "text-slate-500 text-xs font-medium tracking-wide", "{card.set} • {packs_text}" }
                                        }
                                    }

                                    // RIGHT: Account Selection
                                    div { class: "flex flex-col gap-3 w-full md:w-64 md:pt-2",
                                        span { class: "text-[10px] text-slate-500 uppercase font-black tracking-widest mb-1", "Add to Account" }
                                        
                                        for acc in props.collection.read().accounts.iter() {
                                            {
                                                let c = card.clone(); 
                                                let target_acc = acc.name.clone();
                                                let is_main = acc.main;
                                                rsx! {
                                                    button {
                                                        class: "w-full py-3.5 px-4 bg-slate-800/60 hover:bg-indigo-500/15 border border-indigo-500/15 hover:border-indigo-500/40 rounded-xl text-white font-medium flex items-center gap-3 transition-all active:scale-[0.97] group backdrop-blur-sm",
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
                                                        },
                                                        // Account icon
                                                        div { class: "w-9 h-9 rounded-lg bg-slate-700/60 group-hover:bg-indigo-500/20 flex items-center justify-center transition-colors flex-shrink-0",
                                                            span { class: "text-sm", if is_main { "⭐" } else { "👤" } }
                                                        }
                                                        div { class: "flex flex-col items-start",
                                                            span { class: "text-sm font-semibold group-hover:text-indigo-400 transition-colors", "{acc.name}" }
                                                            if is_main {
                                                                span { class: "text-[10px] text-slate-500", "Main Account" }
                                                            }
                                                        }
                                                        // Arrow icon on the right
                                                        svg { class: "w-4 h-4 text-slate-600 group-hover:text-indigo-400 ml-auto transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        button { 
                                            class: "text-slate-500 hover:text-white text-sm mt-2 py-2 transition-colors", 
                                            onclick: move |_| selected_card_to_add.set(None), 
                                            "← Back to results" 
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-3",
                            for api_card in filtered_api_cards() {
                                {
                                    let c = api_card.clone();
                                    let packs_display = if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") };

                                    rsx! {
                                        div { 
                                            class: "bg-slate-800/60 border border-indigo-500/15 rounded-xl p-2 cursor-pointer hover:border-indigo-500/50 hover:bg-slate-800/80 transition-all flex flex-col backdrop-blur-sm",
                                            onclick: move |_| selected_card_to_add.set(Some(c.clone())),
                                            img { 
                                                src: "{optimized_image_url(&api_card.full_image_url, 400)}", 
                                                loading: "lazy", decoding: "async",
                                                width: "400", height: "560",
                                                class: "w-full rounded-lg mb-2 shadow-sm border border-indigo-500/10 aspect-[63/88] object-cover" 
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
