use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::models::optimized_image_url;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct TradeButtonProps {
    pub show_trade_modal: Signal<bool>,
}

#[component]
pub fn TradeButton(mut props: TradeButtonProps) -> Element {
    rsx! {
        button {
            class: "group w-11 h-11 md:w-14 md:h-14 flex items-center justify-center bg-slate-800/60 border border-indigo-500/20 rounded-xl md:rounded-2xl hover:bg-slate-700/80 hover:border-indigo-400/40 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| props.show_trade_modal.set(true),
            title: "Trade Cards",
            svg { class: "w-5 h-5 md:w-6 md:h-6 text-indigo-400 group-hover:text-purple-300 group-hover:scale-110 transition-all duration-200", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct TradeModalProps {
    pub show_trade_modal: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn TradeModal(mut props: TradeModalProps) -> Element {
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
            div { class: "fixed inset-0 bg-slate-950/95 flex flex-col z-50 animate-fade-in-down",
                
                div { class: "bg-slate-900/80 border-b border-indigo-500/20 p-4 pt-6 flex justify-between items-center shadow-xl z-10 backdrop-blur-xl",
                    h2 { class: "text-xl font-bold tracking-tight flex items-center gap-2 text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-purple-300", 
                        "Trade Room" 
                    }
                    button { 
                        class: "text-slate-500 hover:text-white p-2 transition-colors", 
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
                                
                                div { class: "flex-1 flex flex-col bg-slate-800/40 p-4 md:p-6 rounded-2xl border border-indigo-500/15 backdrop-blur w-full",
                                    h3 { class: "text-sm text-indigo-400 font-bold uppercase tracking-widest mb-4", "Taking From" }
                                    
                                    select {
                                        class: "w-full bg-slate-900/80 border border-indigo-500/20 rounded-xl px-4 py-3 text-white focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none mb-6 cursor-pointer transition-colors",
                                        value: "{partner_acc}",
                                        onchange: move |evt| {
                                            partner_acc.set(evt.value());
                                            card_taking.set(None); 
                                        },
                                        option { value: "Other", "Other" }
                                        for acc in props.collection.read().accounts.iter() {
                                            option { value: "{acc.name}", "Internal: {acc.name}" }
                                        }
                                    }

                                    if let Some(card) = card_taking.read().clone() {
                                        TradeCardSlot { card, image_db: props.image_db, on_click: move |_| view_state.set("pick_taking".to_string()) }
                                    } else {
                                        button {
                                            class: "w-full aspect-[63/88] max-w-[200px] mx-auto bg-slate-900/50 border-2 border-dashed border-slate-600 rounded-xl flex flex-col items-center justify-center text-slate-500 hover:text-indigo-400 hover:border-indigo-500/50 transition-colors",
                                            onclick: move |_| view_state.set("pick_taking".to_string()),
                                            span { class: "text-4xl mb-2", "+" }
                                            span { class: "text-sm font-medium", "Select Card" }
                                        }
                                    }
                                }

                                div { class: "flex-shrink-0 bg-slate-800/80 border border-indigo-500/30 p-4 rounded-full shadow-lg z-10 -my-4 md:my-0 md:-mx-8",
                                    svg { class: "w-8 h-8 text-indigo-400", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                    }
                                }

                                div { class: "flex-1 flex flex-col bg-slate-800/40 p-4 md:p-6 rounded-2xl border border-indigo-500/15 backdrop-blur w-full",
                                    h3 { class: "text-sm text-indigo-400 font-bold uppercase tracking-widest mb-4", "Giving From" }
                                    
                                    select {
                                        class: "w-full bg-slate-900/80 border border-indigo-500/20 rounded-xl px-4 py-3 text-white focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none mb-6 cursor-pointer transition-colors",
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
                                            class: "w-full aspect-[63/88] max-w-[200px] mx-auto bg-slate-900/50 border-2 border-dashed border-slate-600 rounded-xl flex flex-col items-center justify-center text-slate-500 hover:text-indigo-400 hover:border-indigo-500/50 transition-colors",
                                            onclick: move |_| view_state.set("pick_giving".to_string()),
                                            span { class: "text-4xl mb-2", "-" }
                                            span { class: "text-sm font-medium", "Select Card" }
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
                                            div { class: "text-rose-400 text-sm font-medium bg-rose-900/20 px-4 py-2 rounded-lg border border-rose-500/20", "⚠️ Accounts must be different." }
                                        } else if rarity_mismatch {
                                            div { class: "text-amber-400 text-sm font-medium bg-amber-900/20 px-4 py-2 rounded-lg border border-amber-500/20", "⚠️ Rarities must match exactly." }
                                        }

                                        button {
                                            class: "w-full md:w-auto px-12 py-4 rounded-xl font-bold text-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed",
                                            class: if !has_both || is_same_acc || rarity_mismatch { "bg-slate-700 text-slate-400 shadow-lg" } else { "bg-gradient-to-r from-indigo-500 to-purple-500 hover:from-indigo-400 hover:to-purple-400 text-white shadow-[0_0_25px_rgba(99,102,241,0.3)] hover:shadow-[0_0_35px_rgba(99,102,241,0.5)]" },
                                            disabled: !has_both || is_same_acc || rarity_mismatch,
                                            onclick: move |_| {
                                                let g_opt = card_giving.read().clone();
                                                let t_opt = card_taking.read().clone();

                                                if let (Some(g), Some(t)) = (g_opt, t_opt) {
                                                    let my_a = my_acc.read().clone();
                                                    let partner_a = partner_acc.read().clone();
                                                    
                                                    match props.collection.write().trade_card(&g, &t, &my_a, &partner_a) {
                                                        Ok(_) => {
                                                            props.toast_message.set(Some("🤝 Trade Successful!".to_string()));
                                                            let mut t_msg = props.toast_message.clone();
                                                            spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t_msg.set(None); });
                                                            
                                                            props.show_trade_modal.set(false);
                                                            view_state.set("main".to_string());
                                                            card_giving.set(None);
                                                            card_taking.set(None);
                                                        },
                                                        Err(e) => {
                                                            props.toast_message.set(Some(format!("❌ {}", e)));
                                                        }
                                                    }
                                                }
                                            },
                                            "EXECUTE TRADE"
                                        }
                                    }
                                }
                            }
                        }

                    } else {
                        div { class: "w-full max-w-5xl flex flex-col gap-4",
                            div { class: "flex items-center gap-4 mb-4",
                                button { class: "bg-slate-800/80 border border-indigo-500/20 p-2 rounded-lg text-slate-400 hover:text-white hover:border-indigo-400/40 transition-colors", onclick: move |_| view_state.set("main".to_string()), "← Back" }
                                div { class: "relative group flex-1",
                                    div { class: "absolute inset-y-0 left-0 flex items-center pl-4 pointer-events-none text-slate-500 group-focus-within:text-indigo-400 transition-colors",
                                        svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-5 h-5",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                                        }
                                    }
                                    input {
                                        class: "w-full bg-slate-900/80 border border-indigo-500/20 rounded-xl pl-11 pr-4 py-3 text-white focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors",
                                        placeholder: "Search card name...",
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

                            div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3",
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
                                            rsx! { div { class: "col-span-full text-center text-slate-500", "Loading database..." } }
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
                                                div { class: "col-span-full text-center text-slate-500 py-12", "No cards found in {target_acc}'s inventory." }
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
                    class: "w-full rounded-xl shadow-xl border-2 border-indigo-500/40 group-hover:border-purple-400 transition-colors aspect-[63/88] object-cover mb-3" 
                }
            } else {
                div { class: "w-full aspect-[63/88] bg-slate-800 rounded-xl mb-3 flex items-center justify-center border border-indigo-500/20", "🃏" }
            }
            h4 { class: "text-white font-bold text-center truncate w-full px-2", "{card.name}" }
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
            class: "bg-slate-800/60 border border-indigo-500/15 rounded-xl p-2 cursor-pointer hover:border-indigo-500/50 hover:bg-slate-800/80 transition-all flex flex-col items-center backdrop-blur-sm",
            onclick: move |e| onclick.call(e),
            if let Some(img_url) = url {
                img { 
                    src: "{img_url}", 
                    loading: "lazy", decoding: "async",
                    width: "400", height: "560",
                    class: "w-full rounded-lg mb-2 shadow-md aspect-[63/88] object-cover" 
                }
            } else {
                div { class: "w-full aspect-[63/88] bg-slate-700/50 rounded-lg mb-2 border border-indigo-500/10 animate-pulse" }
            }
            h2 { class: "text-[11px] font-bold text-center text-slate-200 truncate w-full px-1", "{card.name}" }
            div { class: "mt-1 mb-1", RarityDisplay { rarity_code: card.rarity } }
        }
    }
}
