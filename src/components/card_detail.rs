use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::models::optimized_image_url;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct CardDetailModalProps {
    pub selected_card_id: Signal<Option<String>>,
    pub collection: Signal<CardCollection>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn CardDetailModal(mut props: CardDetailModalProps) -> Element {
    rsx! {
        if let Some(card_id) = props.selected_card_id.read().clone() {
            if let Some(api_map) = &*props.image_db.read() {
                if let Some(api_card) = api_map.get(&card_id) {
                    {
                        let image_url = optimized_image_url(&api_card.full_image_url, 600);
                        let collection_read = props.collection.read();
                        let entry_opt = collection_read.inventory.iter().find(|e| e.card.id == card_id).cloned();
                        let is_wishlisted = collection_read.is_wishlisted(&card_id);
                        let is_tradable = collection_read.is_tradable(&card_id);
                        let c = Card {
                            id: api_card.generated_id.clone(),
                            name: api_card.name.clone(),
                            rarity: api_card.rarity.clone(),
                            card_type: api_card.card_type.clone(),
                            pack: if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") }
                        };
                        let c_wishlist = c.clone();
                        let c_id_tradable = c.id.clone();

                        rsx! {
                            div { class: "fixed inset-0 bg-slate-950/90 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                                div { class: "bg-slate-900 border border-indigo-500/20 rounded-2xl w-full max-w-md overflow-hidden shadow-2xl flex flex-col animate-fade-in-down",
                                    
                                    // --- HEADER ---
                                    div { class: "flex justify-between items-center p-4 border-b border-indigo-500/15",
                                        h2 { class: "text-xl font-bold text-white truncate pr-4", "{api_card.name}" }
                                        button { 
                                            class: "text-slate-400 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-full w-8 h-8 flex items-center justify-center transition-colors active:scale-95",
                                            onclick: move |_| props.selected_card_id.set(None),
                                            "✕"
                                        }
                                    }

                                    // --- BODY ---
                                    div { class: "p-5 flex flex-col md:flex-row gap-6",
                                        
                                        // Left: Image
                                        div { class: "w-full md:w-1/2 flex justify-center",
                                            img { 
                                                src: "{image_url}", 
                                                sizes: "(max-width: 768px) 192px, 224px",
                                                width: "400", height: "560", 
                                                class: "w-48 md:w-full rounded-xl shadow-lg border border-indigo-500/20 aspect-[63/88] object-cover" 
                                            }
                                        }

                                        // Right: Details & Actions
                                        div { class: "w-full md:w-1/2 flex flex-col gap-4",
                                            
                                            // Metadata
                                            div { class: "grid grid-cols-2 gap-4",
                                                div { class: "flex flex-col gap-1",
                                                    span { class: "text-[10px] text-slate-500 uppercase font-black tracking-widest", "Rarity" }
                                                    div { class: "mt-1 flex justify-start",
                                                        RarityDisplay { rarity_code: api_card.rarity.clone() }
                                                    }
                                                }
                                                div { class: "flex flex-col gap-1",
                                                    span { class: "text-[10px] text-slate-500 uppercase font-black tracking-widest", "Pack" }
                                                    img { 
                                                        src: "https://raw.githubusercontent.com/flibustier/pokemon-tcg-pocket-database/main/dist/images/sets/LOGO_expansion_{api_card.set}_en_US.webp",
                                                        alt: "{c.pack}",
                                                        title: "{c.pack}",
                                                        class: "h-8 w-fit object-contain object-left" 
                                                    }
                                                }
                                            }

                                            hr { class: "border-indigo-500/10" }

                                            // Actions: Wishlist and Tradable
                                            div { class: "flex flex-col gap-2",
                                                button {
                                                    class: "w-full py-2.5 px-3 rounded-lg font-medium flex items-center justify-center gap-2 transition-all active:scale-[0.97] group border text-sm",
                                                    class: if is_wishlisted { "bg-pink-500/20 text-pink-400 border-pink-500/30 hover:bg-pink-500/30" } else { "bg-slate-800/60 text-slate-300 border-slate-600/30 hover:bg-slate-700/80 hover:text-white" },
                                                    onclick: move |_| {
                                                        props.collection.write().toggle_wishlist(c_wishlist.clone());
                                                    },
                                                    svg { class: "w-4 h-4 flex-shrink-0 transition-all", class: if is_wishlisted { "fill-pink-400 text-pink-400" } else { "fill-none text-slate-400 group-hover:text-pink-400" }, view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                                    }
                                                    span { if is_wishlisted { "Remove Wishlist" } else { "Add to Wishlist" } }
                                                }

                                                if entry_opt.is_some() {
                                                    button {
                                                        class: "w-full py-2.5 px-3 rounded-lg font-medium flex items-center justify-center gap-2 transition-all active:scale-[0.97] group border text-sm",
                                                        class: if is_tradable { "bg-emerald-500/20 text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/30" } else { "bg-slate-800/60 text-slate-300 border-slate-600/30 hover:bg-slate-700/80 hover:text-white" },
                                                        onclick: move |_| {
                                                            props.collection.write().toggle_tradable(&c_id_tradable.clone());
                                                        },
                                                        svg { class: "w-4 h-4 flex-shrink-0 transition-all text-current", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                                        }
                                                        span { if is_tradable { "Tradable" } else { "Mark as Tradable" } }
                                                    }
                                                }
                                            }

                                            // Touch-Friendly Owner List
                                            if let Some(entry) = entry_opt {
                                                hr { class: "border-indigo-500/10 mt-1" }
                                                div { class: "flex flex-col gap-2",
                                                    span { class: "text-[10px] text-slate-500 uppercase font-black tracking-widest mb-1", "Owned By" }
                                                    
                                                    for (owner, count) in entry.owners.iter() {
                                                        if *count > 0 {
                                                            {
                                                                let target_card = entry.card.clone();
                                                                let target_owner = owner.clone();
                                                                let c_name = target_card.name.clone();
                                                                
                                                                rsx! {
                                                                    div { class: "flex justify-between items-center bg-slate-800/60 p-2.5 rounded-xl border border-indigo-500/15",
                                                                        div { class: "flex items-center gap-2",
                                                                            span { class: "text-sm font-bold text-white", "{owner}" }
                                                                            span { class: "bg-slate-900 text-slate-300 text-xs px-2 py-0.5 rounded border border-indigo-500/15 font-mono", "x{count}" }
                                                                        }
                                                                        
                                                                        // Big Touch Target for Removal
                                                                        button {
                                                                            class: "bg-rose-500/10 text-rose-400 border border-rose-500/30 hover:bg-rose-500 hover:text-white px-4 py-1.5 rounded-lg text-sm font-bold transition-colors active:scale-95",
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
                                            } else {
                                                div { class: "mt-4 text-center p-4 bg-slate-800/40 rounded-xl border border-indigo-500/10",
                                                    span { class: "text-sm text-slate-400", "You don't own this card yet." }
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
                    div { 
                        onmounted: move |_| props.selected_card_id.set(None),
                    }
                }
            }
        }
    }
}
