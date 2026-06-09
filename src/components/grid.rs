use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;

#[derive(PartialEq, Clone, Props)]
pub struct CardGridProps {
    pub collection: Signal<CardCollection>,
    pub search_query: Signal<String>,
    pub selected_account_filter: Signal<String>,
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>, 
}

#[component]
pub fn CardGrid(mut props: CardGridProps) -> Element {
    rsx! {
        // Reduced gap on mobile (gap-3), standard gap on desktop (md:gap-4)
        div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3 md:gap-4 pb-24",
            for entry in props.collection.read().inventory.iter().filter(|e| {
                let matches_search = props.search_query.read().is_empty() || 
                                     e.card.name.to_lowercase().contains(&props.search_query.read().to_lowercase());
                let matches_account = &*props.selected_account_filter.read() == "All" || 
                                      e.owners.contains_key(&*props.selected_account_filter.read());
                matches_search && matches_account
            }) {
                div { class: "bg-gray-800 border border-gray-700 rounded-xl p-2 md:p-3 flex flex-col items-center shadow-lg transition-transform hover:scale-105",
                    {
                        let image_url = if let Some(Some(api_map)) = &*props.image_db.read() {
                            api_map.get(&entry.card.id).map(|c| c.image.clone())
                        } else {
                            None
                        };

                        if let Some(url) = image_url {
                            rsx! { 
                                img { 
                                    src: "{url}", 
                                    alt: "{entry.card.name}", 
                                    // CRITICAL FOR PERFORMANCE: loading lazy, decoding async, and fixed aspect ratio
                                    loading: "lazy",
                                    decoding: "async",
                                    class: "w-full rounded-lg mb-2 md:mb-3 shadow-md border border-gray-600 aspect-[63/88] object-cover bg-gray-900"
                                } 
                            }
                        } else {
                            rsx! { div { class: "w-full aspect-[63/88] bg-gray-700 rounded-lg mb-2 md:mb-3 flex items-center justify-center border border-gray-600 animate-pulse", span { class: "text-2xl md:text-4xl opacity-50", "🃏" } } }
                        }
                    }
                    
                    h2 { class: "font-bold text-xs md:text-sm text-center truncate w-full", "{entry.card.name}" }
                    p { class: "text-[10px] md:text-xs text-orange-400 mb-2 uppercase tracking-wide", "{entry.card.rarity}" }
                    
                    // --- DYNAMIC OWNER BADGES ---
                    div { class: "w-full flex flex-wrap gap-1 justify-center mt-auto",
                        for (owner_name, count) in entry.owners.iter() {
                            if *count > 0 {
                                {
                                    let target_card = entry.card.clone();
                                    let target_owner = owner_name.clone();
                                    let card_name = entry.card.name.clone();
                                    
                                    // Check if this account is a Main (Green) or Temp (Blue) account
                                    let is_main = props.collection.read().accounts.iter()
                                        .find(|a| a.name == *owner_name)
                                        .map(|a| a.main)
                                        .unwrap_or(false);

                                    let badge_bg = if is_main { 
                                        "bg-green-900/50 text-green-200 border-green-700/50 hover:bg-green-800" 
                                    } else { 
                                        "bg-blue-900/50 text-blue-200 border-blue-700/50 hover:bg-blue-800" 
                                    };
                                    
                                    rsx! {
                                        div { class: "group relative flex items-center gap-1 border pl-2 pr-1 py-0.5 rounded-full font-mono text-[9px] md:text-[10px] transition-all {badge_bg}",
                                            span { "{owner_name}: {count}" }
                                            
                                            // Hidden 'X' button
                                            button {
                                                class: "opacity-0 group-hover:opacity-100 text-red-400 hover:text-white hover:bg-red-500 rounded-full w-3.5 h-3.5 flex items-center justify-center transition-all cursor-pointer font-bold",
                                                title: "Remove 1",
                                                onclick: move |_| {
                                                    let result = props.collection.write().remove_card(&target_card, &target_owner, 1);
                                                    match result {
                                                        Ok(_) => props.toast_message.set(Some(format!("🗑️ Removed {} from {}", card_name, target_owner))),
                                                        Err(e) => props.toast_message.set(Some(format!("❌ {}", e))),
                                                    }
                                                    let mut toast = props.toast_message.clone();
                                                    spawn(async move {
                                                        gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                                        toast.set(None);
                                                    });
                                                },
                                                "✕"
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
        
        if props.collection.read().inventory.is_empty() {
            div { class: "text-center text-gray-500 mt-12", "No cards found matching your criteria." }
        }
    }
}
