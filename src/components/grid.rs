use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;

#[derive(PartialEq, Clone, Props)]
pub struct CardGridProps {
    pub collection: Signal<CardCollection>,
    pub search_query: Signal<String>,
    pub selected_account_filter: Signal<String>,
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub selected_card_id: Signal<Option<String>>, 
}

#[component]
pub fn CardGrid(mut props: CardGridProps) -> Element {
    
    let visible_entries: Vec<Inventory> = props.collection.read().inventory.iter().filter(|e| {
        let matches_search = props.search_query.read().is_empty() || 
                             e.card.name.to_lowercase().contains(&props.search_query.read().to_lowercase());
        let matches_account = &*props.selected_account_filter.read() == "All" || 
                              e.owners.contains_key(&*props.selected_account_filter.read());
        matches_search && matches_account
    }).cloned().collect();

    rsx! {
        div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3 md:gap-4 pb-24",
            for entry in visible_entries {
                div { 
                    class: "bg-gray-800 border border-gray-700 rounded-xl p-2 md:p-3 flex flex-col items-center shadow-lg transition-transform hover:scale-105 cursor-pointer active:scale-95 group",
                    
                    // The safe onclick handler using the cloned ID
                    onclick: move |_| props.selected_card_id.set(Some(entry.card.id.clone())),
                    
                    {
                        let image_url = if let Some(Some(api_map)) = &*props.image_db.read() {
                            api_map.get(&entry.card.id).map(|c| c.image.clone())
                        } else {
                            None
                        };

                        if let Some(url) = image_url {
                            let optimized_url = format!("https://wsrv.nl/?url={}&w=200&output=webp", url.replace("https://", ""));
                            rsx! { 
                                img { 
                                    src: "{optimized_url}", 
                                    alt: "{entry.card.name}", 
                                    loading: "lazy",
                                    decoding: "async",
                                    class: "w-full rounded-lg mb-2 md:mb-3 shadow-md border border-gray-600 aspect-[63/88] object-cover bg-gray-900 content-[auto] group-hover:border-gray-400 transition-colors"
                                } 
                            }
                        } else {
                            rsx! { div { class: "w-full aspect-[63/88] bg-gray-700 rounded-lg mb-2 md:mb-3 flex items-center justify-center border border-gray-600 animate-pulse", span { class: "text-2xl md:text-4xl opacity-50", "🃏" } } }
                        }
                    }
                    
                    h2 { class: "font-bold text-xs md:text-sm text-center truncate w-full group-hover:text-orange-400 transition-colors", "{entry.card.name}" }
                    p { class: "text-[10px] md:text-xs text-orange-400 mb-1 uppercase tracking-wide", "{entry.card.rarity}" }
                }
            }
        }
        
        if props.collection.read().inventory.is_empty() {
            div { class: "text-center text-gray-500 mt-12", "No cards found matching your criteria." }
        }
    }
}
