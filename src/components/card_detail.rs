use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct CardDetailModalProps {
    pub selected_card_id: Signal<Option<String>>,
    pub collection: Signal<CardCollection>,
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn CardDetailModal(mut props: CardDetailModalProps) -> Element {
    rsx! {
        if let Some(card_id) = props.selected_card_id.read().clone() {
            // Find the specific card in our inventory
            {
                let collection_read = props.collection.read();
                let entry_opt = collection_read.inventory.iter().find(|e| e.card.id == card_id).cloned();
                
                if let Some(entry) = entry_opt {
                    // Fetch the optimized image
                    let image_url = if let Some(Some(api_map)) = &*props.image_db.read() {
                        api_map.get(&entry.card.id).map(|c| c.full_image_url.clone()) // FIXED HERE
                    } else { None };
                    
                    let optimized_url = image_url.map(|url| format!("https://wsrv.nl/?url={}&w=400&output=webp", url.replace("https://", "")));

                    rsx! {
                        div { class: "fixed inset-0 bg-black/90 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                            div { class: "bg-gray-900 border border-gray-700 rounded-2xl w-full max-w-md overflow-hidden shadow-2xl flex flex-col animate-fade-in-down",
                                
                                // --- HEADER ---
                                div { class: "flex justify-between items-center p-4 border-b border-gray-800",
                                    h2 { class: "text-xl font-bold text-white truncate pr-4", "{entry.card.name}" }
                                    button { 
                                        class: "text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700 rounded-full w-8 h-8 flex items-center justify-center transition-colors active:scale-95",
                                        onclick: move |_| props.selected_card_id.set(None),
                                        "✕"
                                    }
                                }

                                // --- BODY ---
                                div { class: "p-5 flex flex-col md:flex-row gap-6",
                                    
                                    // Left: Image
                                    div { class: "w-full md:w-1/2 flex justify-center",
                                        if let Some(url) = optimized_url {
                                            img { src: "{url}", class: "w-48 md:w-full rounded-xl shadow-lg border border-gray-700 aspect-[63/88] object-cover" }
                                        } else {
                                            div { class: "w-48 aspect-[63/88] bg-gray-800 rounded-xl flex items-center justify-center", "🃏" }
                                        }
                                    }

                                    // Right: Details & Actions
                                    div { class: "w-full md:w-1/2 flex flex-col gap-4",
                                        
                                        // Metadata
                                        div { class: "grid grid-cols-2 gap-4",
                                            div { class: "flex flex-col gap-1",
                                                span { class: "text-[10px] text-gray-500 uppercase font-black tracking-widest", "Rarity" }
                                                div { class: "mt-1 flex justify-start",
                                                    RarityDisplay { rarity_code: entry.card.rarity.clone() }
                                                }
                                            }
                                            div { class: "flex flex-col gap-1",
                                                span { class: "text-[10px] text-gray-500 uppercase font-black tracking-widest", "Pack" }
                                                span { class: "text-sm text-blue-400 font-bold truncate", "{entry.card.pack}" }
                                            }
                                        }

                                        hr { class: "border-gray-800" }

                                        // Touch-Friendly Owner List
                                        div { class: "flex flex-col gap-2",
                                            span { class: "text-[10px] text-gray-500 uppercase font-black tracking-widest mb-1", "Owned By" }
                                            
                                            for (owner, count) in entry.owners.iter() {
                                                if *count > 0 {
                                                    {
                                                        let target_card = entry.card.clone();
                                                        let target_owner = owner.clone();
                                                        let c_name = target_card.name.clone();
                                                        
                                                        rsx! {
                                                            div { class: "flex justify-between items-center bg-gray-800/80 p-2.5 rounded-xl border border-gray-700",
                                                                div { class: "flex items-center gap-2",
                                                                    span { class: "text-sm font-bold text-white", "{owner}" }
                                                                    span { class: "bg-gray-900 text-gray-300 text-xs px-2 py-0.5 rounded border border-gray-700 font-mono", "x{count}" }
                                                                }
                                                                
                                                                // Big Touch Target for Removal
                                                                button {
                                                                    class: "bg-red-500/10 text-red-400 border border-red-500/30 hover:bg-red-500 hover:text-white px-4 py-1.5 rounded-lg text-sm font-bold transition-colors active:scale-95",
                                                                    onclick: move |_| {
                                                                        let res = props.collection.write().remove_card(&target_card, &target_owner, 1);
                                                                        if res.is_ok() {
                                                                            props.toast_message.set(Some(format!("🗑️ Removed {} from {}", c_name, target_owner)));
                                                                            let mut t = props.toast_message.clone();
                                                                            spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                                                        }
                                                                    },
                                                                    "Remove"
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
                } else {
                    // If the card was completely removed from inventory, auto-close the modal
                    rsx! {
                        div { 
                            onmounted: move |_| props.selected_card_id.set(None),
                        }
                    }
                }
            }
        }
    }
}
