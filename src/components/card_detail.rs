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
    let mut show_accounts = use_signal(|| false);
    let mut is_closing = use_signal(|| false);

    rsx! {
        if let Some(card_id) = props.selected_card_id.read().clone() {
            if let Some(api_map) = &*props.image_db.read() {
                if let Some(api_card) = api_map.get(&card_id) {
                    {
                        let image_url = optimized_image_url(&api_card.full_image_url, 800); // Higher res for detail view
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
                            div { class: if *is_closing.read() { "fixed inset-0 bg-slate-950/70 flex items-center justify-center p-4 z-50 backdrop-blur-xl transition-all animate-fade-out" } else { "fixed inset-0 bg-slate-950/70 flex items-center justify-center p-4 z-50 backdrop-blur-xl transition-all animate-fade-in" },
                                
                                // Close background area (click to close)
                                div { 
                                    class: "absolute inset-0 z-0",
                                    onclick: move |_| {
                                        is_closing.set(true);
                                        let mut selected_card = props.selected_card_id.clone();
                                        let mut closing = is_closing.clone();
                                        let mut accounts_shown = show_accounts.clone();
                                        spawn(async move {
                                            gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                                            selected_card.set(None);
                                            closing.set(false);
                                            accounts_shown.set(false);
                                        });
                                    }
                                }

                                // Main Card Container (Glass)
                                div { class: if *is_closing.read() { "glass-panel rounded-3xl w-full max-w-lg p-6 md:p-8 flex flex-col items-center gap-6 shadow-[0_20px_50px_rgba(0,0,0,0.5)] animate-zoom-out relative z-10" } else { "glass-panel rounded-3xl w-full max-w-lg p-6 md:p-8 flex flex-col items-center gap-6 shadow-[0_20px_50px_rgba(0,0,0,0.5)] animate-zoom-in relative z-10" },
                                    
                                    // Close Button
                                    button { 
                                        class: "absolute top-4 right-4 text-slate-400 hover:text-white bg-white/5 hover:bg-white/10 rounded-full w-8 h-8 flex items-center justify-center transition-colors active:scale-95 border border-white/10",
                                        onclick: move |_| {
                                            is_closing.set(true);
                                            let mut selected_card = props.selected_card_id.clone();
                                            let mut closing = is_closing.clone();
                                            let mut accounts_shown = show_accounts.clone();
                                            spawn(async move {
                                                gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                                                selected_card.set(None);
                                                closing.set(false);
                                                accounts_shown.set(false);
                                            });
                                        },
                                        "✕"
                                    }

                                    // High-Res Image
                                    img { 
                                        src: "{image_url}", 
                                        sizes: "(max-width: 768px) 300px, 400px",
                                        width: "600", height: "840", 
                                        class: "w-64 md:w-80 rounded-2xl shadow-2xl drop-shadow-[0_20px_30px_rgba(0,0,0,0.4)] border border-white/10 object-cover" 
                                    }

                                    // Title & Pack info
                                    div { class: "flex flex-col items-center w-full gap-1",
                                        h2 { class: "text-2xl font-black text-white text-center tracking-tight", "{api_card.name}" }
                                        div { class: "flex items-center gap-3",
                                            RarityDisplay { rarity_code: api_card.rarity.clone() }
                                            div { class: "w-1 h-1 rounded-full bg-slate-600" }
                                            img { 
                                                src: "https://raw.githubusercontent.com/flibustier/pokemon-tcg-pocket-database/main/dist/images/sets/LOGO_expansion_{api_card.set}_en_US.webp",
                                                alt: "{c.pack}",
                                                title: "{c.pack}",
                                                class: "h-6 w-fit object-contain drop-shadow-md" 
                                            }
                                        }
                                    }

                                    // Action Buttons Row
                                    div { class: "grid grid-cols-3 gap-3 w-full mt-2",
                                        // Wishlist Toggle
                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if is_wishlisted { "bg-pink-500/20 text-pink-400 border-pink-500/30 hover:bg-pink-500/30 shadow-[0_0_15px_rgba(236,72,153,0.2)]" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onclick: move |_| { props.collection.write().toggle_wishlist(c_wishlist.clone()); },
                                            svg { class: "w-5 h-5 transition-all", class: if is_wishlisted { "fill-pink-400 text-pink-400" } else { "fill-none text-current" }, view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Wishlist" }
                                        }

                                        // Tradable Toggle
                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if is_tradable { "bg-teal-500/20 text-teal-400 border-teal-500/30 hover:bg-teal-500/30 shadow-[0_0_15px_rgba(45,212,191,0.2)]" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onclick: move |_| { props.collection.write().toggle_tradable(&c_id_tradable.clone()); },
                                            svg { class: "w-5 h-5 transition-all", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Tradable" }
                                        }

                                        // Accounts / Inventory Drawer Toggle
                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if *show_accounts.read() { "bg-cyan-500/20 text-cyan-400 border-cyan-500/30 shadow-[0_0_15px_rgba(6,182,212,0.2)]" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onclick: move |_| { show_accounts.set(!show_accounts()); },
                                            svg { class: "w-5 h-5 transition-all", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Accounts" }
                                        }
                                    }
                                }

                                // --- ACCOUNTS SIDE MENU (SLIDE-OUT) ---
                                if *show_accounts.read() {
                                    div { class: "fixed inset-y-0 right-0 w-80 glass-panel border-l border-white/10 shadow-2xl z-50 p-6 flex flex-col",
                                        // Animate via inline style since we didn't add the custom keyframe yet
                                        style: "animation: slideIn 0.3s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;",
                                        
                                        // Fake keyframe injection for slide in
                                        style { "
                                            @keyframes slideIn {{
                                                from {{ transform: translateX(100%); }}
                                                to {{ transform: translateX(0); }}
                                            }}
                                        " }
                                        
                                        div { class: "flex justify-between items-center mb-6",
                                            h3 { class: "text-lg font-bold text-white", "Inventory" }
                                            button { 
                                                class: "text-slate-400 hover:text-white p-1",
                                                onclick: move |_| show_accounts.set(false),
                                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                                                }
                                            }
                                        }

                                        if let Some(entry) = entry_opt {
                                            div { class: "flex flex-col gap-3 overflow-y-auto",
                                                for (owner, count) in entry.owners.iter() {
                                                    if *count > 0 {
                                                        {
                                                            let target_card = entry.card.clone();
                                                            let target_owner = owner.clone();
                                                            let c_name = target_card.name.clone();
                                                            
                                                            rsx! {
                                                                div { class: "flex justify-between items-center bg-white/5 p-3 rounded-xl border border-white/10 backdrop-blur-md",
                                                                    div { class: "flex flex-col",
                                                                        span { class: "text-sm font-bold text-white", "{owner}" }
                                                                        span { class: "text-teal-400 text-xs font-mono", "Qty: {count}" }
                                                                    }
                                                                    
                                                                    button {
                                                                        class: "bg-rose-500/10 text-rose-400 border border-rose-500/30 hover:bg-rose-500 hover:text-white px-3 py-1.5 rounded-lg text-xs font-bold transition-all active:scale-95",
                                                                        onclick: move |_| {
                                                                            let res = props.collection.write().remove_card(&target_card, &target_owner, 1);
                                                                            if res.is_ok() {
                                                                                props.toast_message.set(Some(format!("🗑️ Removed {} from {}", c_name, target_owner)));
                                                                                let mut t = props.toast_message.clone();
                                                                                spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                                                            }
                                                                        },
                                                                        "- Remove"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            div { class: "flex flex-col items-center justify-center flex-1 text-center opacity-70",
                                                svg { class: "w-12 h-12 text-slate-500 mb-2", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                                }
                                                span { class: "text-sm text-slate-400", "No accounts currently own this card." }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { onmounted: move |_| props.selected_card_id.set(None) }
                }
            }
        }
    }
}
