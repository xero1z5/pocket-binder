use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct CardGridProps {
    pub collection: Signal<CardCollection>,
    pub search_query: Signal<String>,
    pub selected_account_filter: Signal<String>,
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub selected_card_id: Signal<Option<String>>, 
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
    // Memoized filtering: only recalculates when collection, search_query, or account filter change
    let visible_entries = use_memo(move || {
        let query = props.search_query.read().to_lowercase();
        let selected_acc = props.selected_account_filter.read().clone();

        let mut entries: Vec<Inventory> = props.collection.read().inventory.iter().filter(|e| {
            let matches_search = query.is_empty() || e.card.name.to_lowercase().contains(&query);
            let matches_account = selected_acc == "All" || e.owners.contains_key(&selected_acc);
            matches_search && matches_account
        }).cloned().collect();

        entries.sort_by(|a, b| {
            get_rarity_priority(&a.card.rarity).cmp(&get_rarity_priority(&b.card.rarity))
        });

        entries
    });

    rsx! {
        div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-1 sm:gap-2 md:gap-4 pb-24 px-1",
            for entry in visible_entries() {
                {
                    // Clone the ID up front so both closures can own their own copy
                    let card_id_for_click = entry.card.id.clone();
                    let card_id_for_hover = entry.card.id.clone();

                    let image_url = if let Some(Some(api_map)) = &*props.image_db.read() {
                        api_map.get(&entry.card.id).map(|c| c.full_image_url.clone())
                    } else { None };

                    let card_name = entry.card.name.clone();
                    let rarity_code = entry.card.rarity.clone();

                    rsx! {
                        div { 
                            class: "bg-slate-800/40 border border-indigo-500/10 rounded-xl p-2 flex flex-col items-center shadow-lg transition-all duration-300 hover:-translate-y-1.5 hover:shadow-[0_10px_20px_rgba(99,102,241,0.15)] cursor-pointer active:scale-95 group backdrop-blur-sm",
                            onclick: move |_| props.selected_card_id.set(Some(card_id_for_click.clone())),
                            // Prefetch the larger detail image on hover so it's cached when clicked
                            onmouseenter: move |_| {
                                let cid = card_id_for_hover.clone();
                                if let Some(Some(api_map)) = &*props.image_db.read() {
                                    if let Some(api_card) = api_map.get(&cid) {
                                        let detail_url = format!("https://wsrv.nl/?url={}&w=400&output=webp", api_card.full_image_url.replace("https://", ""));
                                        spawn(async move {
                                            let _ = reqwest::get(&detail_url).await;
                                        });
                                    }
                                }
                            },

                            if let Some(url) = image_url {
                                {
                                    rsx! {
                                        img { 
                                            src: "{url}", 
                                            alt: "{card_name}", 
                                            loading: "lazy", decoding: "async",
                                            width: "400", height: "560",
                                            class: "w-full rounded-lg mb-2 shadow-md border border-indigo-500/10 aspect-[63/88] object-cover group-hover:border-indigo-400/50 transition-colors"
                                        } 
                                    }
                                }
                            } else {
                                div { class: "w-full aspect-[63/88] bg-slate-700/50 rounded-lg mb-2 border border-indigo-500/10 animate-pulse" }
                            }
                            
                            h2 { class: "font-semibold text-[10px] md:text-xs text-center truncate w-full text-slate-200 group-hover:text-indigo-400 transition-colors tracking-tight", "{card_name}" }
                            
                            div { class: "mt-1",
                                RarityDisplay { rarity_code }
                            }
                        }
                    }
                }
            }
        }
        
        if props.collection.read().inventory.is_empty() {
            div { class: "flex flex-col items-center justify-center mt-20 text-slate-500",
                svg { class: "w-12 h-12 mb-3 opacity-50", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" }
                }
                span { class: "text-sm font-medium", "No cards found." }
            }
        }
    }
}
