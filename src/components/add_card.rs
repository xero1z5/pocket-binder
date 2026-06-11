use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;

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
            span { class: "text-[10px] font-bold text-teal-400", "{label}" }
        };
    }

    rsx! {
        div { class: "flex items-center justify-center gap-0.5", title: "{label}",
            for _ in 0..count {
                img { 
                    src: "{image_asset.clone().unwrap()}",
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
            class: "group w-11 h-11 md:w-14 md:h-14 flex items-center justify-center bg-slate-800/80 border border-slate-700/80 rounded-xl md:rounded-2xl hover:bg-slate-700 hover:border-teal-500/50 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| props.show_add_modal.set(true),
            svg { class: "w-5 h-5 md:w-6 md:h-6 text-slate-400 group-hover:text-teal-400 transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
            }
        }
    }
}

// --- THE MODAL ---
#[derive(PartialEq, Clone, Props)]
pub struct AddCardModalProps {
    pub show_add_modal: Signal<bool>,
    pub add_search_query: Signal<String>,
    pub collection: Signal<CardCollection>,
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn AddCardModal(mut props: AddCardModalProps) -> Element {
    let mut selected_card_to_add = use_signal(|| None::<OfficialCard>);

    rsx! {
        if *props.show_add_modal.read() {
            div { class: "fixed inset-0 bg-slate-950/90 flex flex-col z-50 animate-fade-in-down",
                div { class: "bg-slate-900 border-b border-slate-800 p-4 pt-6 flex flex-col gap-4 shadow-xl z-10",
                    div { class: "flex justify-between items-center",
                        h2 { class: "text-xl font-bold text-white tracking-tight", "Add Cards" }
                        button { 
                            class: "text-slate-500 hover:text-white p-2 transition-colors", 
                            onclick: move |_| { props.show_add_modal.set(false); selected_card_to_add.set(None); }, 
                            svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                        }
                    }
                    input { 
                        class: "bg-slate-950 border border-slate-800 rounded-xl px-4 py-3 text-white placeholder-slate-500 focus:border-teal-500 outline-none w-full", 
                        placeholder: "Search card name...", 
                        value: "{props.add_search_query}", 
                        oninput: move |evt| { props.add_search_query.set(evt.value()); selected_card_to_add.set(None); } 
                    }
                }

                div { class: "flex-1 overflow-y-auto p-4 relative",
                    if let Some(card) = selected_card_to_add.read().clone() {
                        {
                            let packs_text = if card.packs.is_empty() { "Promo".to_string() } else { card.packs.join(", ") };
                            
                            rsx! {
                                div { class: "flex flex-col items-center justify-center h-full max-w-sm mx-auto gap-6",
                                    img { src: "{card.full_image_url}", class: "w-48 rounded-2xl border border-slate-700 shadow-2xl" }
                                    
                                    div { class: "flex flex-col items-center gap-1 mt-2",
                                        RarityDisplay { rarity_code: card.rarity.clone() }
                                        span { class: "text-slate-400 text-sm font-medium", "{packs_text}" }
                                    }

                                    h3 { class: "text-xl font-bold text-white", "Add {card.name}?" }
                                    
                                    div { class: "w-full flex flex-col gap-2",
                                        for acc in props.collection.read().accounts.iter() {
                                            {
                                                let c = card.clone(); 
                                                let target_acc = acc.name.clone();
                                                rsx! {
                                                    button {
                                                        class: "w-full py-3.5 px-4 bg-slate-800 hover:bg-teal-900/30 border border-slate-700 rounded-xl text-white font-medium flex justify-between items-center transition-all",
                                                        onclick: move |_| {
                                                            let card_to_add = Card { 
                                                                id: c.generated_id.clone(), 
                                                                name: c.name.clone(), 
                                                                rarity: c.rarity.clone(), // NOW SAVING THE RAW CODE
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
                                                        span { "Add to {acc.name}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    button { class: "text-slate-500 hover:text-white mt-4", onclick: move |_| selected_card_to_add.set(None), "← Back" }
                                }
                            }
                        }
                    } else {
                        div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-3",
                            if let Some(Some(api_map)) = &*props.image_db.read() {
                                {
                                    let query = props.add_search_query.read().to_lowercase();
                                    
                                    rsx! {
                                        for (_, api_card) in api_map.iter().filter(|(_, c)| {
                                            !query.is_empty() && c.name.to_lowercase().contains(&query)
                                        }).take(30) {
                                            {
                                                let c = api_card.clone();
                                                let optimized_url = format!("https://wsrv.nl/?url={}&w=200&output=webp", api_card.full_image_url.replace("https://", ""));
                                                let packs_display = if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") };

                                                rsx! {
                                                    div { 
                                                        class: "bg-slate-800 border border-slate-700 rounded-xl p-2 cursor-pointer hover:border-teal-500 transition-all flex flex-col",
                                                        onclick: move |_| selected_card_to_add.set(Some(c.clone())),
                                                        img { src: "{optimized_url}", class: "w-full rounded-lg mb-2 shadow-sm border border-slate-700 aspect-[63/88] object-cover" }
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
        }
    }
}
