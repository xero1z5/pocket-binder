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
    pub current_view_cards: Signal<Vec<String>>,
}

const CARD_STEP: f64 = 180.0;
const ANGLE_STEP_RAD: f64 = 25.0 * std::f64::consts::PI / 180.0;
const RADIUS: f64 = 550.0;
const VISIBLE_RADIUS: i32 = 6;

fn ease_out_quart(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(4)
}

fn start_inertia(mut translate_x: Signal<f64>, velocity: f64, mut is_inertia: Signal<bool>) {
    if velocity.abs() < 1.0 {
        snap_and_animate(translate_x, is_inertia, 250.0);
        return;
    }
    spawn(async move {
        let mut v = velocity;
        is_inertia.set(true);
        while v.abs() > 0.5 && *is_inertia.read() {
            gloo_timers::future::sleep(std::time::Duration::from_millis(16)).await;
            if !*is_inertia.read() { break; }
            v *= 0.94;
            translate_x.with_mut(|t| *t += v);
        }
        if *is_inertia.read() {
            snap_and_animate(translate_x, is_inertia, 200.0);
        }
    });
}

fn snap_and_animate(mut translate_x: Signal<f64>, mut is_inertia: Signal<bool>, duration_ms: f64) {
    let current = *translate_x.read();
    let target = (current / CARD_STEP).round() * CARD_STEP;
    if (current - target).abs() < 0.5 { return; }

    let start = current;
    let delta = target - start;

    spawn(async move {
        is_inertia.set(true);
        let mut elapsed = 0.0;
        while elapsed < duration_ms && *is_inertia.read() {
            gloo_timers::future::sleep(std::time::Duration::from_millis(16)).await;
            if !*is_inertia.read() { break; }
            elapsed += 16.0;
            let t = (elapsed / duration_ms).min(1.0);
            translate_x.set(start + delta * ease_out_quart(t));
        }
        if *is_inertia.read() {
            translate_x.set(target);
            is_inertia.set(false);
        }
    });
}

fn animate_to_card(target_offset: f64, mut translate_x: Signal<f64>, mut is_inertia: Signal<bool>) {
    if *is_inertia.read() { return; }
    let current = *translate_x.read();
    let target = current + target_offset;
    let start = current;
    let delta = target - start;
    let duration_ms = (delta.abs() / CARD_STEP * 300.0).min(500.0).max(250.0);

    spawn(async move {
        is_inertia.set(true);
        let mut elapsed = 0.0;
        while elapsed < duration_ms && *is_inertia.read() {
            gloo_timers::future::sleep(std::time::Duration::from_millis(16)).await;
            if !*is_inertia.read() { break; }
            elapsed += 16.0;
            let t = (elapsed / duration_ms).min(1.0);
            translate_x.set(start + delta * ease_out_quart(t));
        }
        if *is_inertia.read() {
            translate_x.set(target);
            is_inertia.set(false);
        }
    });
}

fn close_modal_action(mut is_closing: Signal<bool>, mut selected_card: Signal<Option<String>>, mut show_accounts: Signal<bool>) {
    is_closing.set(true);
    spawn(async move {
        gloo_timers::future::sleep(std::time::Duration::from_millis(300)).await;
        selected_card.set(None);
        is_closing.set(false);
        show_accounts.set(false);
    });
}

fn find_card_index(cards: &[String], id: &str) -> Option<usize> {
    cards.iter().position(|c| c == id)
}

fn init_translate_x(cards: &[String], selected_id: &str) -> f64 {
    if let Some(pos) = find_card_index(cards, selected_id) {
        -(pos as f64) * CARD_STEP
    } else {
        0.0
    }
}

#[component]
pub fn CardDetailModal(mut props: CardDetailModalProps) -> Element {
    let mut show_accounts = use_signal(|| false);
    let mut is_closing = use_signal(|| false);
    let mut translate_x = use_signal(|| 0.0f64);
    let mut is_dragging = use_signal(|| false);
    let mut start_x = use_signal(|| 0.0f64);
    let mut last_x = use_signal(|| 0.0f64);
    let mut velocity = use_signal(|| 0.0f64);
    let mut is_inertia = use_signal(|| false);
    let mut last_selected = use_signal(|| None::<String>);

    use_effect(move || {
        let tx = *translate_x.read();
        let cards = props.current_view_cards.read();
        if cards.is_empty() { return; }
        let len = cards.len();
        let center_card_f = -tx / CARD_STEP;
        let center_idx = (center_card_f.round() as i32).rem_euclid(len as i32) as usize;
        let center_card_id = cards[center_idx].clone();
        drop(cards);
        let mut selected = props.selected_card_id.write();
        if selected.is_some() && *selected != Some(center_card_id.clone()) {
            *selected = Some(center_card_id.clone());
            last_selected.set(Some(center_card_id));
        }
    });

    rsx! {
        if let Some(card_id) = props.selected_card_id.read().clone() {
            if let Some(api_map) = &*props.image_db.read() {
                if let Some(api_card) = api_map.get(&card_id) {
                    {
                        if *last_selected.read() != Some(card_id.clone()) {
                            let cards = props.current_view_cards.read();
                            let pos = init_translate_x(&cards, &card_id);
                            translate_x.set(pos);
                            last_selected.set(Some(card_id.clone()));
                        }

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

                        let cards = props.current_view_cards.read();
                        let len = cards.len() as i32;
                        let current_tx = *translate_x.read();
                        let center_card_f = -current_tx / CARD_STEP;
                        let center_idx = if len > 0 {
                            let rounded = center_card_f.round() as i32;
                            ((rounded % len) + len) % len
                        } else { 0 };

                        let mut carousel_slots: Vec<(i32, i32, String)> = Vec::new();
                        if len > 0 {
                            for rel in -VISIBLE_RADIUS..=VISIBLE_RADIUS {
                                let abs_idx = ((center_idx as i32 + rel) % len + len) % len;
                                let target_id = cards[abs_idx as usize].clone();
                                carousel_slots.push((rel, abs_idx, target_id));
                            }
                        }
                        drop(cards);

                        rsx! {
                            div {
                                class: if *is_closing.read() { "fixed inset-0 bg-slate-950/30 flex items-center justify-center p-4 z-50 backdrop-blur-xl transition-all animate-backdrop-exit outline-none" } else { "fixed inset-0 bg-slate-950/30 flex items-center justify-center p-4 z-50 backdrop-blur-xl transition-all animate-backdrop-enter outline-none" },
                                tabindex: "0",
                                autofocus: "true",
                                onkeydown: move |evt| {
                                    if evt.key() == Key::ArrowLeft {
                                        animate_to_card(CARD_STEP, translate_x, is_inertia);
                                    } else if evt.key() == Key::ArrowRight {
                                        animate_to_card(-CARD_STEP, translate_x, is_inertia);
                                    } else if evt.key() == Key::Escape {
                                        close_modal_action(is_closing, props.selected_card_id, show_accounts);
                                    }
                                },

                                div {
                                    class: "absolute inset-0 z-0",
                                    onclick: move |_| close_modal_action(is_closing, props.selected_card_id, show_accounts)
                                }

                                div {
                                    class: if *is_closing.read() { "glass-panel rounded-3xl w-full max-w-lg p-6 md:p-8 flex flex-col items-center gap-6 shadow-[0_20px_50px_rgba(0,0,0,0.5)] animate-card-exit relative z-10 overflow-hidden" } else { "glass-panel rounded-3xl w-full max-w-lg p-6 md:p-8 flex flex-col items-center gap-6 shadow-[0_20px_50px_rgba(0,0,0,0.5)] animate-card-enter relative z-10 overflow-hidden" },
                                    style: "touch-action: none;",

                                    onmousedown: move |evt| {
                                        is_inertia.set(false);
                                        is_dragging.set(true);
                                        let cx = evt.client_coordinates().x;
                                        start_x.set(cx);
                                        last_x.set(cx);
                                        velocity.set(0.0);
                                    },
                                    onmousemove: move |evt| {
                                        if *is_dragging.read() {
                                            let cx = evt.client_coordinates().x;
                                            let delta = cx - *last_x.read();
                                            last_x.set(cx);
                                            velocity.set(delta);
                                            translate_x.with_mut(|t| *t += delta);
                                        }
                                    },
                                    onmouseup: move |_| {
                                        if *is_dragging.read() {
                                            is_dragging.set(false);
                                            let vel = *velocity.read();
                                            start_inertia(translate_x, vel, is_inertia);
                                        }
                                    },
                                    onmouseleave: move |_| {
                                        if *is_dragging.read() {
                                            is_dragging.set(false);
                                            let vel = *velocity.read();
                                            start_inertia(translate_x, vel, is_inertia);
                                        }
                                    },
                                    ontouchstart: move |evt| {
                                        if let Some(touch) = evt.touches().first() {
                                            is_inertia.set(false);
                                            is_dragging.set(true);
                                            let cx = touch.client_coordinates().x;
                                            start_x.set(cx);
                                            last_x.set(cx);
                                            velocity.set(0.0);
                                        }
                                    },
                                    ontouchmove: move |evt| {
                                        if *is_dragging.read() {
                                            if let Some(touch) = evt.touches().first() {
                                                let cx = touch.client_coordinates().x;
                                                let delta = cx - *last_x.read();
                                                last_x.set(cx);
                                                velocity.set(delta);
                                                translate_x.with_mut(|t| *t += delta);
                                            }
                                        }
                                    },
                                    ontouchend: move |_| {
                                        if *is_dragging.read() {
                                            is_dragging.set(false);
                                            let vel = *velocity.read();
                                            start_inertia(translate_x, vel, is_inertia);
                                        }
                                    },

                                    button {
                                        class: "absolute top-4 right-4 text-slate-400 hover:text-white bg-white/5 hover:bg-white/10 rounded-full w-8 h-8 flex items-center justify-center transition-colors active:scale-95 border border-white/10 z-30",
                                        onclick: move |_| close_modal_action(is_closing, props.selected_card_id, show_accounts),
                                        "✕"
                                    }

                                    // Infinite horizontal film strip
                                    div {
                                        class: "relative w-full h-[320px] md:h-[420px] flex items-center justify-center my-2 select-none overflow-hidden",
                                        style: "perspective: 1000px; transform-style: preserve-3d;",

                                        for (rel, _abs_idx, target_id) in carousel_slots {
                                            {
                                                let is_center = rel == 0;
                                                let card_angle = (center_card_f.round() + rel as f64 - center_card_f) * ANGLE_STEP_RAD;

                                                let tx = RADIUS * card_angle.sin();
                                                let tz = -RADIUS * (1.0 - card_angle.cos());
                                                let ry_deg = -card_angle * 180.0 / std::f64::consts::PI;

                                                let scale = (0.5 + 0.5 * card_angle.cos()).max(0.25);
                                                let opacity = (0.3 + 0.7 * card_angle.cos()).clamp(0.0, 1.0);
                                                let z_idx = ((card_angle.cos() * 100.0) as i32).max(0);

                                                let transform_style = if *is_dragging.read() || *is_inertia.read() {
                                                    format!(
                                                        "transform: translateX({}px) translateZ({}px) rotateY({}deg) scale({}); opacity: {}; z-index: {};",
                                                        tx, tz, ry_deg, scale, opacity, z_idx
                                                    )
                                                } else {
                                                    format!(
                                                        "transform: translateX({}px) translateZ({}px) rotateY({}deg) scale({}); transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.4s cubic-bezier(0.22, 1, 0.36, 1); opacity: {}; z-index: {};",
                                                        tx, tz, ry_deg, scale, opacity, z_idx
                                                    )
                                                };

                                                rsx! {
                                                    div {
                                                        key: "{target_id}_{rel}",
                                                        class: "absolute cursor-pointer flex items-center justify-center",
                                                        style: "{transform_style}",
                                                        onclick: move |evt| {
                                                            evt.stop_propagation();
                                                            if !*is_dragging.read() && !*is_inertia.read() && rel != 0 {
                                                                animate_to_card(-rel as f64 * CARD_STEP, translate_x, is_inertia);
                                                            }
                                                        },
                                                        if let Some(target_api_card) = api_map.get(&target_id) {
                                                            {
                                                                let item_image_url = optimized_image_url(&target_api_card.full_image_url, 400);
                                                                let img_class = if is_center {
                                                                    "w-44 md:w-56 rounded-2xl shadow-2xl drop-shadow-[0_20px_30px_rgba(0,0,0,0.5)] border-2 border-white/15 object-cover pointer-events-none ring-2 ring-indigo-500/30"
                                                                } else {
                                                                    "w-44 md:w-56 rounded-2xl shadow-xl drop-shadow-[0_10px_20px_rgba(0,0,0,0.4)] border border-white/5 object-cover pointer-events-none"
                                                                };
                                                                rsx! {
                                                                    img {
                                                                        src: "{item_image_url}",
                                                                        class: "{img_class}",
                                                                        width: "400", height: "560",
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
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
                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if is_wishlisted { "bg-pink-500/15 text-pink-400 border-pink-500/20 hover:bg-pink-500/25" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onclick: move |_| { props.collection.write().toggle_wishlist(c_wishlist.clone()); },
                                            svg { class: "w-5 h-5 transition-all", class: if is_wishlisted { "fill-pink-400 text-pink-400" } else { "fill-none text-current" }, view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Wishlist" }
                                        }

                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if is_tradable { "bg-emerald-500/15 text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/25" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onclick: move |_| { props.collection.write().toggle_tradable(&c_id_tradable.clone()); },
                                            svg { class: "w-5 h-5 transition-all", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Tradable" }
                                        }

                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if *show_accounts.read() { "bg-sky-500/15 text-sky-400 border-sky-500/20" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onclick: move |_| { show_accounts.set(!show_accounts()); },
                                            svg { class: "w-5 h-5 transition-all", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Accounts" }
                                        }
                                    }
                                }

                                // --- ACCOUNTS SIDE MENU ---
                                if *show_accounts.read() {
                                    div { class: "fixed inset-0 bg-slate-950/50 backdrop-blur-md z-[55] animate-fade-in", onclick: move |_| show_accounts.set(false) }
                                    div { class: "fixed inset-y-0 right-0 w-80 glass-panel border-l border-white/10 shadow-2xl z-[60] p-6 flex flex-col animate-slide-in-right",

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
                                                                        span { class: "text-sky-400 text-xs font-mono", "Qty: {count}" }
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
