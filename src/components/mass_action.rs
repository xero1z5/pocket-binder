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
    pub active_view: Signal<String>,
}

#[component]
pub fn MassActionBar(mut props: MassActionBarProps) -> Element {
    let mut show_account_picker = use_signal(|| false);

    // Reset the account picker whenever multi-select is turned off so it never
    // re-opens on its own the next time multi-select is used.
    use_effect(move || {
        if !*props.mass_select_mode.read() {
            show_account_picker.set(false);
        }
    });

    let count = props.selected_mass_cards.read().len();
    if !*props.mass_select_mode.read() {
        return rsx! {};
    }

    let in_wishlist_view = *props.active_view.read() == "wishlist";
    let in_tradable_view = *props.active_view.read() == "tradable";

    if count == 0 {
        // Instruction state (no cards selected yet).
        return rsx! {
            div { class: "fixed bottom-4 md:bottom-6 inset-x-0 z-[60] flex justify-center px-3 pointer-events-none",
                div { class: "acrylic pointer-events-auto rounded-[1.75rem] pl-5 pr-1.5 py-1.5 flex items-center gap-3 animate-slide-up",
                    span { class: "text-sm text-slate-300", "Tap cards to select them" }
                    button {
                        class: "icon-btn is-bare is-danger",
                        title: "Stop selecting",
                        onclick: move |_| props.mass_select_mode.set(false),
                        svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "fixed bottom-4 md:bottom-6 inset-x-0 z-[60] flex justify-center px-3 pointer-events-none",
            div { class: "acrylic relative pointer-events-auto rounded-[1.75rem] pl-5 pr-1.5 py-1.5 flex items-center gap-1.5 md:gap-2",
                span { class: "text-sm font-medium text-white mr-2 whitespace-nowrap", "{count} selected" }

                // Bulk Add Account Button
                button {
                    class: "icon-btn",
                    class: if *show_account_picker.read() { "is-on" },
                    title: "Add to an account",
                    onclick: move |_| {
                        let curr = *show_account_picker.read();
                        show_account_picker.set(!curr);
                    },
                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                    }
                }

                // Wishlist / Remove from Wishlist (depending on active view)
                button {
                    class: if in_wishlist_view {
                        "icon-btn bg-[#ff5a7a]/20 border-[#ff5a7a]/50 text-[#ff8fa6]"
                    } else {
                        "icon-btn is-wish"
                    },
                    title: if in_wishlist_view { "Remove from Wishlist" } else { "Add to Wishlist" },
                    onclick: move |_| {
                        let cards = props.selected_mass_cards.read().clone();
                        if in_wishlist_view {
                            let mut updated = props.collection.read().clone();
                            for id in cards.iter() {
                                updated.remove_wishlist(id);
                            }
                            props.collection.set(updated);
                            props.toast_message.set(Some(format!("Removed {} cards from Wishlist", cards.len())));

                            props.selected_mass_cards.set(Vec::new());
                            props.mass_select_mode.set(false);
                        } else if let Some(db) = props.image_db.read().as_ref() {
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

                                props.selected_mass_cards.set(Vec::new());
                                props.mass_select_mode.set(false);
                            } else {
                                props.toast_message.set(Some("Cards were already wishlisted".to_string()));

                            }
                        }
                    },
                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                    }
                }

                // Bulk Tradable (only in collection or tradable view)
                if !in_wishlist_view && !props.active_view.read().starts_with("pack:") {
                    button {
                        class: if in_tradable_view {
                            "icon-btn bg-[#2fcf7f]/20 border-[#2fcf7f]/50 text-[#4ade9a]"
                        } else {
                            "icon-btn is-trade"
                        },
                        title: if in_tradable_view { "Remove from Tradable" } else { "Add to Tradable" },
                        onclick: move |_| {
                            let cards = props.selected_mass_cards.read().clone();
                            let mut updated = props.collection.read().clone();
                            let mut changed = 0;
                            if in_tradable_view {
                                for id in cards.iter() {
                                    if updated.is_tradable(id) {
                                        updated.toggle_tradable(id);
                                        changed += 1;
                                    }
                                }
                            } else {
                                for id in cards.iter() {
                                    if !updated.is_tradable(id) {
                                        updated.toggle_tradable(id);
                                        changed += 1;
                                    }
                                }
                            }
                            if changed > 0 {
                                props.collection.set(updated);
                                props.toast_message.set(Some(if in_tradable_view {
                                    format!("Removed {} cards from Tradable", changed)
                                } else {
                                    format!("Added {} cards to Tradable", changed)
                                }));

                                props.selected_mass_cards.set(Vec::new());
                                props.mass_select_mode.set(false);
                            } else {
                                props.toast_message.set(Some(if in_tradable_view {
                                    "Cards were not tradable".to_string()
                                } else {
                                    "Cards were already tradable".to_string()
                                }));
                            }
                        },
                        svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                        }
                    }
                }

                // Cancel
                button {
                    class: "icon-btn is-bare is-danger",
                    title: "Stop selecting",
                    onclick: move |_| {
                        props.selected_mass_cards.set(Vec::new());
                        props.mass_select_mode.set(false);
                    },
                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                    }
                }

                if *show_account_picker.read() {
                    div { class: "acrylic-strong absolute bottom-full mb-3 right-0 w-64 rounded-3xl p-2 animate-fade-in-up",
                        h4 { class: "text-xs font-medium text-slate-400 px-3 pt-2 pb-2", "Add to account" }
                        div { class: "flex flex-col gap-1 max-h-56 overflow-y-auto",
                            for account in props.collection.read().accounts.iter() {
                                {
                                    let acc_name = account.name.clone();
                                    rsx! {
                                        button {
                                            class: "row is-plain font-medium",
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
                                div { class: "text-center text-slate-400 text-xs px-3 py-3", "No accounts yet. Add one from the Account menu." }
                            }
                        }
                    }
                }
            }
        }
    }
}
