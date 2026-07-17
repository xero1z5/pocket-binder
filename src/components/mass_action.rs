use dioxus::prelude::*;
use crate::models::{CardCollection, Card};
use std::collections::HashMap;
use crate::models::OfficialCard;

#[derive(PartialEq, Clone, Props)]
pub struct MassActionBarProps {
    pub selected_mass_cards: Signal<Vec<String>>,
    pub mass_select_mode: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn MassActionBar(mut props: MassActionBarProps) -> Element {
    let mut show_account_picker = use_signal(|| false);

    let count = props.selected_mass_cards.read().len();
    if !*props.mass_select_mode.read() || count == 0 {
        if *props.mass_select_mode.read() {
            // Show a tiny instruction bar if no cards selected but mode is active
            return rsx! {
                div { class: "fixed bottom-0 inset-x-0 pb-4 md:pb-6 z-40 flex justify-center pointer-events-none animate-fade-in-up",
                    div { class: "glass-panel px-6 py-3 rounded-full flex items-center gap-3 shadow-2xl backdrop-blur-xl border-white/20 pointer-events-auto",
                        span { class: "text-sm font-semibold text-slate-300", "Select cards to perform mass actions" }
                        button {
                            class: "ml-2 text-xs text-rose-400 hover:text-rose-300 uppercase font-black tracking-wider transition-colors",
                            onclick: move |_| props.mass_select_mode.set(false),
                            "Cancel"
                        }
                    }
                }
            };
        }
        return rsx! {};
    }

    rsx! {
        div { class: "fixed bottom-0 inset-x-0 pb-4 md:pb-6 z-[60] flex justify-center pointer-events-none animate-fade-in-up",
            div { class: "relative group pointer-events-auto",
                div { class: "glass-panel p-2 rounded-2xl flex flex-wrap items-center gap-2 shadow-2xl backdrop-blur-xl border-white/20",
                    div { class: "px-4 py-2 bg-indigo-500/20 rounded-xl border border-indigo-500/30 flex items-center justify-center min-w-[3rem]",
                        span { class: "text-sm font-black text-indigo-300", "{count}" }
                    }

                    // Bulk Add Account Button
                    button {
                        class: "px-4 py-2 bg-white/5 hover:bg-sky-500/15 border border-white/10 hover:border-sky-500/30 text-slate-300 hover:text-sky-300 rounded-xl transition-all flex items-center gap-2 text-sm font-bold shadow-sm active:scale-95",
                        onclick: move |_| {
                            let curr = *show_account_picker.read();
                            show_account_picker.set(!curr);
                        },
                        svg { class: "w-4 h-4", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                        }
                        "Add to Account"
                    }

                    // Bulk Wishlist
                    button {
                        class: "px-4 py-2 bg-white/5 hover:bg-pink-500/15 border border-white/10 hover:border-pink-500/30 text-slate-300 hover:text-pink-300 rounded-xl transition-all flex items-center gap-2 text-sm font-bold shadow-sm active:scale-95",
                        onclick: move |_| {
                            let cards = props.selected_mass_cards.read().clone();
                            if let Some(db) = props.image_db.read().as_ref() {
                                let mut updated = props.collection.read().clone();
                                let mut added = 0;
                                for id in cards.iter() {
                                    if let Some(api_card) = db.get(id) {
                                        let c = Card {
                                            id: api_card.generated_id.clone(),
                                            name: api_card.name.clone(),
                                            rarity: api_card.rarity.clone(),
                                            card_type: api_card.card_type.clone(),
                                            pack: if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") }
                                        };
                                        if !updated.is_wishlisted(&c.id) {
                                            updated.toggle_wishlist(c);
                                            added += 1;
                                        }
                                    }
                                }
                                if added > 0 {
                                    props.collection.set(updated);
                                    props.toast_message.set(Some(format!("Added {} cards to Wishlist", added)));
                                    let mut t = props.toast_message.clone();
                                    spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                    // Clear selection
                                    props.selected_mass_cards.set(Vec::new());
                                    props.mass_select_mode.set(false);
                                } else {
                                    props.toast_message.set(Some("Cards were already wishlisted".to_string()));
                                    let mut t = props.toast_message.clone();
                                    spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                }
                            }
                        },
                        svg { class: "w-4 h-4", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                        }
                        "Wishlist"
                    }

                    // Bulk Tradable
                    button {
                        class: "px-4 py-2 bg-white/5 hover:bg-emerald-500/15 border border-white/10 hover:border-emerald-500/30 text-slate-300 hover:text-emerald-300 rounded-xl transition-all flex items-center gap-2 text-sm font-bold shadow-sm active:scale-95",
                        onclick: move |_| {
                            let cards = props.selected_mass_cards.read().clone();
                            let mut updated = props.collection.read().clone();
                            let mut added = 0;
                            for id in cards.iter() {
                                if !updated.is_tradable(id) {
                                    updated.toggle_tradable(id);
                                    added += 1;
                                }
                            }
                            if added > 0 {
                                props.collection.set(updated);
                                props.toast_message.set(Some(format!("Added {} cards to Tradable", added)));
                                let mut t = props.toast_message.clone();
                                spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                // Clear selection
                                props.selected_mass_cards.set(Vec::new());
                                props.mass_select_mode.set(false);
                            } else {
                                props.toast_message.set(Some("Cards were already tradable".to_string()));
                                let mut t = props.toast_message.clone();
                                spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                            }
                        },
                        svg { class: "w-4 h-4", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                        }
                        "Tradable"
                    }

                    // Cancel
                    button {
                        class: "px-4 py-2 bg-rose-500/10 border border-rose-500/30 text-rose-400 hover:bg-rose-500/20 hover:text-rose-300 rounded-xl transition-all text-sm font-bold shadow-sm active:scale-95",
                        onclick: move |_| {
                            props.selected_mass_cards.set(Vec::new());
                            props.mass_select_mode.set(false);
                        },
                        "Cancel"
                    }
                }

                if *show_account_picker.read() {
                    div { class: "absolute bottom-full mb-4 right-0 w-64 bg-slate-950/95 backdrop-blur-3xl rounded-xl p-2 shadow-2xl border border-white/10 animate-fade-in-up",
                        h4 { class: "text-sm font-bold text-white mb-3 tracking-wide text-center", "Select Account" }
                        div { class: "flex flex-col gap-2 max-h-48 overflow-y-auto",
                            for account in props.collection.read().accounts.iter() {
                                {
                                    let acc_name = account.name.clone();
                                    rsx! {
                                        button {
                                            class: "w-full text-left px-4 py-3 bg-white/5 hover:bg-sky-500/20 border border-white/5 hover:border-sky-500/40 rounded-xl text-sm font-semibold text-slate-300 hover:text-white transition-all",
                                            onclick: move |_| {
                                                let cards = props.selected_mass_cards.read().clone();
                                                if let Some(db) = props.image_db.read().as_ref() {
                                                    let mut updated = props.collection.read().clone();
                                                    for id in cards.iter() {
                                                        if let Some(api_card) = db.get(id) {
                                                            let c = Card {
                                                                id: api_card.generated_id.clone(),
                                                                name: api_card.name.clone(),
                                                                rarity: api_card.rarity.clone(),
                                                                card_type: api_card.card_type.clone(),
                                                                pack: if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") }
                                                            };
                                                            updated.add_card(c, &acc_name, 1);
                                                        }
                                                    }
                                                    props.collection.set(updated);
                                                    props.toast_message.set(Some(format!("Added {} cards to {}", cards.len(), acc_name)));
                                                    let mut t = props.toast_message.clone();
                                                    spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                                    
                                                    // Reset
                                                    show_account_picker.set(false);
                                                    props.selected_mass_cards.set(Vec::new());
                                                    props.mass_select_mode.set(false);
                                                }
                                            },
                                            "{acc_name}"
                                        }
                                    }
                                }
                            }
                            if props.collection.read().accounts.is_empty() {
                                div { class: "text-center text-slate-400 text-xs py-2", "No accounts found. Create one first." }
                            }
                        }
                    }
                }
            }
        }
    }
}
