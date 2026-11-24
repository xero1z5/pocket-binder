use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::models::optimized_image_url;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct TradeModalProps {
    pub show_trade_modal: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn TradeModal(mut props: TradeModalProps) -> Element {
    if !*props.show_trade_modal.read() {
        return rsx! {};
    }

    let mut my_acc = use_signal(|| String::new());
    let mut partner_acc = use_signal(|| "Other".to_string());
    
    let mut card_giving = use_signal(|| None::<Card>);
    let mut card_taking = use_signal(|| None::<Card>);
    
    let mut view_state = use_signal(|| "main".to_string());
    let mut search_query = use_signal(|| String::new());
    let mut raw_search_query = use_signal(|| String::new());

    use_effect(move || {
        let accounts = &props.collection.read().accounts;
        if my_acc.read().is_empty() && !accounts.is_empty() {
            my_acc.set(accounts[0].name.clone());
        }
    });

    rsx! {
        if *props.show_trade_modal.read() {
            div { class: "fixed inset-0 bg-slate-950/90 flex flex-col z-50 animate-fade-in",
                
                div { class: "acrylic border-x-0 border-t-0 pl-5 pr-3 md:pl-7 py-3 flex justify-between items-center z-10",
                    h2 { class: "text-lg font-semibold tracking-tight text-white", "Trade" }
                    button {
                        class: "icon-btn is-bare",
                        title: "Close", 
                        onclick: move |_| { 
                            props.show_trade_modal.set(false); 
                            view_state.set("main".to_string());
                            card_giving.set(None);
                            card_taking.set(None);
                        }, 
                        svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                    }
                }

                div { class: "flex-1 overflow-y-auto p-4 md:p-8 flex justify-center",
                    
                    if *view_state.read() == "main" {
                        div { class: "w-full max-w-4xl flex flex-col gap-8",
                            
                            div { class: "flex flex-col md:flex-row gap-6 md:gap-12 items-center w-full",
                                
                                div { class: "acrylic flex-1 flex flex-col p-5 md:p-6 rounded-3xl w-full",
                                    h3 { class: "text-base font-semibold text-white mb-3", "You get" }

                                    select {
                                        class: "field mb-5",
                                        value: "{partner_acc}",
                                        onchange: move |evt| {
                                            partner_acc.set(evt.value());
                                            card_taking.set(None); 
                                        },
                                        option { value: "Other", "Someone else" }
                                        for acc in props.collection.read().accounts.iter() {
                                            option { value: "{acc.name}", "{acc.name} (your account)" }
                                        }
                                    }

                                    if let Some(card) = card_taking.read().clone() {
                                        TradeCardSlot { card, image_db: props.image_db, on_click: move |_| view_state.set("pick_taking".to_string()) }
                                    } else {
                                        button {
                                            class: "w-full aspect-[63/88] max-w-[200px] mx-auto rounded-[4.5%/3.2%] border-2 border-dashed border-white/15 bg-white/[0.02] flex flex-col items-center justify-center gap-1 text-slate-400 hover:text-accent hover:border-accent/60 transition-colors",
                                            onclick: move |_| view_state.set("pick_taking".to_string()),
                                            span { class: "text-3xl font-light", "+" }
                                            span { class: "text-sm font-medium", "Choose a card" }
                                        }
                                    }
                                }

                                div { class: "flex-shrink-0 w-14 h-14 rounded-full bg-slate-800 border border-white/10 text-slate-200 flex items-center justify-center z-10 -my-3 md:my-0 md:-mx-9",
                                    svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                    }
                                }

                                div { class: "acrylic flex-1 flex flex-col p-5 md:p-6 rounded-3xl w-full",
                                    h3 { class: "text-base font-semibold text-white mb-3", "You give" }

                                    select {
                                        class: "field mb-5",
                                        value: "{my_acc}",
                                        onchange: move |evt| {
                                            my_acc.set(evt.value());
                                            card_giving.set(None);
                                        },
                                        for acc in props.collection.read().accounts.iter() {
                                            option { value: "{acc.name}", "{acc.name}" }
                                        }
                                    }

                                    if let Some(card) = card_giving.read().clone() {
                                        TradeCardSlot { card, image_db: props.image_db, on_click: move |_| view_state.set("pick_giving".to_string()) }
                                    } else {
                                        button {
                                            class: "w-full aspect-[63/88] max-w-[200px] mx-auto rounded-[4.5%/3.2%] border-2 border-dashed border-white/15 bg-white/[0.02] flex flex-col items-center justify-center gap-1 text-slate-400 hover:text-accent hover:border-accent/60 transition-colors",
                                            onclick: move |_| view_state.set("pick_giving".to_string()),
                                            span { class: "text-3xl font-light", "+" }
                                            span { class: "text-sm font-medium", "Choose a card" }
                                        }
                                    }
                                }
                            }

                            div { class: "w-full flex flex-col items-center gap-4 mt-4",
                                {
                                    let is_same_acc = *my_acc.read() == *partner_acc.read();
                                    let has_both = card_giving.read().is_some() && card_taking.read().is_some();
                                    let mut rarity_mismatch = false;

                                    if has_both {
                                        if let (Some(g), Some(t)) = (&*card_giving.read(), &*card_taking.read()) {
                                            if g.rarity != t.rarity { rarity_mismatch = true; }
                                        }
                                    }

                                    rsx! {
                                        if is_same_acc {
                                            div { class: "text-sm text-rose-300 bg-rose-500/10 px-4 py-2 rounded-xl border border-rose-500/20", "Pick two different accounts." }
                                        } else if rarity_mismatch {
                                            div { class: "text-sm text-amber-200 bg-amber-500/10 px-4 py-2 rounded-xl border border-amber-500/20", "Both cards need the same rarity." }
                                        }

                                        button {
                                            class: "btn btn-primary w-full md:w-auto px-12 min-h-[3.25rem] text-base",
                                            disabled: !has_both || is_same_acc || rarity_mismatch,
                                            onclick: move |_| {
                                                let g_opt = card_giving.read().clone();
                                                let t_opt = card_taking.read().clone();

                                                if let (Some(g), Some(t)) = (g_opt, t_opt) {
                                                    let my_a = my_acc.read().clone();
                                                    let partner_a = partner_acc.read().clone();
                                                    
                                                    match props.collection.write().trade_card(&g, &t, &my_a, &partner_a) {
                                                        Ok(_) => {
                                                            props.toast_message.set(Some("Trade complete".to_string()));
                                                            
                                                            
                                                            props.show_trade_modal.set(false);
                                                            view_state.set("main".to_string());
                                                            card_giving.set(None);
                                                            card_taking.set(None);
                                                        },
                                                        Err(e) => {
                                                            props.toast_message.set(Some(format!("Trade failed: {}", e)));
                                                        }
                                                    }
                                                }
                                            },
                                            "Complete trade"
                                        }
                                    }
                                }
                            }
                        }

                    } else {
                        div { class: "w-full max-w-5xl flex flex-col gap-4",
                            div { class: "flex items-center gap-4 mb-4",
                                button { class: "btn btn-glass", onclick: move |_| view_state.set("main".to_string()), "Back" }
                                div { class: "relative group flex-1",
                                    div { class: "absolute inset-y-0 left-0 flex items-center pl-3.5 pointer-events-none text-slate-500 group-focus-within:text-accent transition-colors",
                                        svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-5 h-5",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                                        }
                                    }
                                    input {
                                        class: "field pl-11",
                                        r#type: "search",
                                        placeholder: "Search by card name, then press Enter",
                                        value: "{raw_search_query}",
                                        oninput: move |evt| raw_search_query.set(evt.value()),
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                search_query.set(raw_search_query.read().clone());
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 gap-x-3 gap-y-5 md:gap-x-5",
                                {
                                    let query = search_query.read().to_lowercase();
                                    let v_state = view_state.read().clone();
                                    let p_acc = partner_acc.read().clone();
                                    let m_acc = my_acc.read().clone();

                                    if v_state == "pick_taking" && p_acc == "Other" {
                                        if let Some(api_map) = &*props.image_db.read() {
                                            rsx! {
                                                for (_, api_card) in api_map.iter().filter(|(_, c)| query.is_empty() || c.name.to_lowercase().contains(&query)).take(30) {
                                                    {
                                                        let c = api_card.clone();
                                                        let new_card = Card { 
                                                            id: c.generated_id.clone(), name: c.name.clone(), rarity: c.rarity.clone(), card_type: c.card_type.clone(), pack: if c.packs.is_empty() { "Promo".to_string() } else { c.packs.join(", ") }
                                                        };
                                                        rsx! {
                                                            PickerSlot { card: new_card.clone(), image_db: props.image_db, onclick: move |_| { card_taking.set(Some(new_card.clone())); view_state.set("main".to_string()); search_query.set(String::new()); raw_search_query.set(String::new()); } }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! { div { class: "col-span-full text-center text-slate-400 py-12", "Loading cards…" } }
                                        }
                                    } else {
                                        let target_acc = if v_state == "pick_giving" { m_acc } else { p_acc };
                                        let available_entries: Vec<Inventory> = props.collection.read().inventory.iter().filter(|e| {
                                            let owns_it = e.owners.get(&target_acc).copied().unwrap_or(0) > 0;
                                            let matches_search = query.is_empty() || e.card.name.to_lowercase().contains(&query);
                                            owns_it && matches_search
                                        }).cloned().collect();

                                        rsx! {
                                            if available_entries.is_empty() {
                                                div { class: "col-span-full text-center text-slate-400 py-12", "{target_acc} has no matching cards." }
                                            }
                                            for entry in available_entries {
                                                {
                                                    let c = entry.card.clone();
                                                    rsx! {
                                                        PickerSlot { card: c.clone(), image_db: props.image_db, onclick: move |_| { 
                                                            if view_state.read().clone() == "pick_giving" { card_giving.set(Some(c.clone())); } else { card_taking.set(Some(c.clone())); }
                                                            view_state.set("main".to_string());
                                                            search_query.set(String::new());
                                                            raw_search_query.set(String::new());
                                                        }}
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

#[component]
fn TradeCardSlot(card: Card, image_db: Signal<Option<HashMap<String, OfficialCard>>>, on_click: EventHandler<MouseEvent>) -> Element {
    let url = if let Some(api_map) = &*image_db.read() {
        api_map.get(&card.id).map(|c| optimized_image_url(&c.full_image_url, 400))
    } else { None };

    rsx! {
        div { 
            class: "w-full max-w-[200px] mx-auto flex flex-col items-center cursor-pointer group",
            onclick: move |e| on_click.call(e),
            if let Some(img_url) = url {
                img { 
                    src: "{img_url}", 
                    alt: "{card.name}",
                    class: "w-full rounded-[4.5%/3.2%] aspect-[63/88] object-cover mb-3 shadow-[0_16px_40px_rgba(2,6,18,0.55)] group-hover:ring-2 ring-accent transition"  
                }
            } else {
                div { class: "w-full aspect-[63/88] rounded-[4.5%/3.2%] mb-3 shimmer" }
            }
            h4 { class: "text-white font-medium text-center truncate w-full px-2", "{card.name}" }
            div { class: "mt-1", RarityDisplay { rarity_code: card.rarity } }
        }
    }
}

#[component]
fn PickerSlot(card: Card, image_db: Signal<Option<HashMap<String, OfficialCard>>>, onclick: EventHandler<MouseEvent>) -> Element {
    let url = if let Some(api_map) = &*image_db.read() {
        api_map.get(&card.id).map(|c| optimized_image_url(&c.full_image_url, 400))
    } else { None };

    rsx! {
        div {
            class: "card-tile",
            onclick: move |e| onclick.call(e),
            div { class: "card-art",
                if let Some(img_url) = url {
                    img {
                        src: "{img_url}",
                        alt: "{card.name}",
                        loading: "lazy", decoding: "async",
                        width: "400", height: "560",
                    }
                } else {
                    div { class: "w-full aspect-[63/88] shimmer" }
                }
            }
            div { class: "card-name", "{card.name}" }
            div { class: "flex justify-center mt-1", RarityDisplay { rarity_code: card.rarity } }
        }
    }
}
