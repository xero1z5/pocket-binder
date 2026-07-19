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
const N: usize = (2 * VISIBLE_RADIUS + 1) as usize;

// Physics tuning — "slot machine" feel: free fling + gentle settle
const FRICTION: f64 = 0.90;
const SNAP_VEL: f64 = 0.002;
const MAX_VEL: f64 = 1.6;

fn wrap_idx(i: i32, len: i32) -> usize {
    if len <= 0 { 0 } else { (((i % len) + len) % len) as usize }
}

fn slot_transform(rel: i32, frac: f64) -> String {
    let angle = (rel as f64 - frac) * ANGLE_STEP_RAD;
    let cos = angle.cos();
    let sin = angle.sin();
    let tx = RADIUS * sin;
    let tz = -RADIUS * (1.0 - cos);
    let ry = -angle * 180.0 / std::f64::consts::PI;

    // Center is prominent; side cards recede (smaller, dimmer, darker).
    let norm = ((cos + 1.0) / 2.0).clamp(0.0, 1.0); // 0 at the far edges, 1 at center
    let scale = (0.42 + 0.58 * norm).powf(1.5) * 1.14; // center ≈ 1.14, sides fall off
    let opacity = (0.22 + 0.78 * norm).clamp(0.0, 1.0);
    let brightness = (0.32 + 0.68 * norm).clamp(0.0, 1.0);
    let z = ((cos * 50.0) as i32).max(0);
    format!(
        "transform: translateX({:.2}px) translateZ({:.2}px) rotateY({:.2}deg) scale({:.3}); opacity: {:.3}; filter: brightness({:.2}); z-index: {}; will-change: transform; backface-visibility: hidden;",
        tx, tz, ry, scale, opacity, brightness, z
    )
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

#[component]
pub fn CardDetailModal(mut props: CardDetailModalProps) -> Element {
    let CardDetailModalProps { mut selected_card_id, mut collection, image_db, mut toast_message, current_view_cards } = props;

    let mut show_accounts = use_signal(|| false);
    let mut is_closing = use_signal(|| false);

    // Continuous carousel position (in card units). Increasing = next cards to the left.
    let init_pos = {
        let cards = current_view_cards.read();
        let c = selected_card_id.read().as_ref()
            .and_then(|id| cards.iter().position(|x| x == id))
            .unwrap_or(0);
        c as f64
    };
    let mut pos = use_signal(|| init_pos);
    let mut vel = use_signal(|| 0.0f64);
    let mut dragging = use_signal(|| false);
    let mut dragged = use_signal(|| false);
    let mut last_x = use_signal(|| 0.0f64);
    let mut snap_target = use_signal(|| None::<f64>);
    let mut center_id = use_signal(|| None::<String>);
    let mut loop_started = use_signal(|| false);
    // Tween state for smooth ease-in-out snapping (clicking a side card / arrow keys)
    let mut anim_from = use_signal(|| 0.0f64);
    let mut anim_t = use_signal(|| 1.0f64);
    // True only while the carousel is at rest. Details (name/rarity/pack/actions)
    // are hidden during drag/fling so we don't thrash layout every animation frame.
    let mut settled = use_signal(|| true);

    let init_ids = {
        let cards = current_view_cards.read();
        let len = cards.len() as i32;
        if len == 0 {
            Vec::new()
        } else {
            let center = selected_card_id.read().as_ref()
                .and_then(|id| cards.iter().position(|x| x == id))
                .unwrap_or(0) as i32;
            (0..N as i32)
                .map(|s| cards[wrap_idx(center + (s - VISIBLE_RADIUS), len)].clone())
                .collect()
        }
    };
    let mut slot_ids = use_signal(|| init_ids);

    // Single requestAnimationFrame loop drives the whole carousel smoothly.
    // Global `selected_card_id` is only synced when the carousel settles, so the
    // (potentially huge) grid behind the modal is NOT re-rendered every frame.
    let onmounted_loop = {
        let mut pos = pos;
        let mut vel = vel;
        let mut dragging = dragging;
        let mut snap_target = snap_target;
        let mut slot_ids = slot_ids;
        let mut center_id = center_id;
        let mut anim_from = anim_from;
        let mut anim_t = anim_t;
        let current_view_cards = current_view_cards;
        let mut selected_card_id = selected_card_id;
        let mut loop_started = loop_started;
        move |_| {
            loop_started.set(true);
            spawn(async move {
                let mut last_center: i32 = i32::MIN;
                let mut last_settled = true;

                // Reset the carousel to the currently selected card. The outer
                // CardDetailModal stays mounted across open/close, so its signals
                // retain the previous scroll position — without this reset the
                // modal would show the last card you scrolled to instead of the
                // one you just clicked.
                {
                    let cards = current_view_cards.read();
                    let len = cards.len() as i32;
                    if len > 0 {
                        let start = selected_card_id.read().as_ref()
                            .and_then(|id| cards.iter().position(|x| x == id))
                            .unwrap_or(0) as f64;
                        drop(cards);
                        dragging.set(false);
                        pos.set(start);
                        vel.set(0.0);
                        snap_target.set(None);
                        anim_t.set(1.0);
                        let ids: Vec<String> = (0..N as i32)
                            .map(|s| current_view_cards.read()[wrap_idx(start as i32 + (s - VISIBLE_RADIUS), len)].clone())
                            .collect();
                        slot_ids.set(ids);
                        center_id.set(Some(current_view_cards.read()[start as usize].clone()));
                        settled.set(true);
                    }
                }

                loop {
                    if selected_card_id.read().is_none() { break; }

                    let dragging_now = *dragging.read();
                    let mut p = *pos.read();
                    let mut v = *vel.read();
                    let target = *snap_target.read();

                    if dragging_now {
                        // position is owned by the pointer handlers
                    } else if let Some(t) = target {
                        // Smooth ease-in-out tween to the target card.
                        let mut at = *anim_t.read();
                        if at >= 1.0 {
                            pos.set(t);
                            snap_target.set(None);
                        } else {
                            at += 0.09; // ~11 frames ≈ 180ms
                            if at > 1.0 { at = 1.0; }
                            anim_t.set(at);
                            let from = *anim_from.read();
                            let eased = at * at * (3.0 - 2.0 * at); // smoothstep
                            pos.set(from + (t - from) * eased);
                        }
                    } else if v.abs() > SNAP_VEL {
                        p += v;
                        v *= FRICTION;
                        if v.abs() > MAX_VEL { v = v.signum() * MAX_VEL; }
                        pos.set(p);
                        vel.set(v);
                    } else {
                        // gentle settle onto the nearest card
                        let nearest = p.round();
                        let diff = nearest - p;
                        if diff.abs() < 0.001 {
                            pos.set(nearest);
                        } else {
                            pos.set(p + diff * 0.25);
                        }
                        vel.set(0.0);
                    }

                    {
                        let cards = current_view_cards.read();
                        let len = cards.len() as i32;
                        if len > 0 {
                            let center = (((p.round() as i32) % len) + len) % len;
                            if center != last_center {
                                last_center = center;
                                let new_ids: Vec<String> = (0..N as i32)
                                    .map(|s| cards[wrap_idx(center + (s - VISIBLE_RADIUS), len)].clone())
                                    .collect();
                                let cid = cards[center as usize].clone();
                                drop(cards);
                                slot_ids.set(new_ids);
                                center_id.set(Some(cid.clone()));
                                if !dragging_now && target.is_none() && v.abs() <= SNAP_VEL {
                                    selected_card_id.set(Some(cid));
                                }
                            }
                        }
                    }

                    // Hide details while the carousel is moving; show them only at rest.
                    let at_rest = !dragging_now && target.is_none() && v.abs() <= SNAP_VEL;
                    if at_rest != last_settled {
                        last_settled = at_rest;
                        settled.set(at_rest);
                    }

                    gloo_timers::future::sleep(std::time::Duration::from_millis(16)).await;
                }
            });
        }
    };

    rsx! {
        if let Some(card_id) = selected_card_id.read().clone() {
            if let Some(api_map) = &*image_db.read() {
                {
                    let display_id = center_id.read().clone().unwrap_or_else(|| card_id.clone());
                    let display_card = api_map.get(&display_id)
                        .or_else(|| api_map.get(&card_id));

                    if let Some(display_api_card) = display_card {
                        let ids = slot_ids.read().clone();
                        let p = *pos.read();
                        let frac = p - p.round();

                        let collection_read = collection.read();
                        let entry_opt = collection_read.inventory.iter().find(|e| e.card.id == display_id).cloned();
                        let is_wishlisted = collection_read.is_wishlisted(&display_id);
                        let is_tradable = collection_read.is_tradable(&display_id);
                        let c = Card {
                            id: display_api_card.generated_id.clone(),
                            name: display_api_card.name.clone(),
                            rarity: display_api_card.rarity.clone(),
                            card_type: display_api_card.card_type.clone(),
                            pack: if display_api_card.packs.is_empty() { "Promo".to_string() } else { display_api_card.packs.join(", ") }
                        };
                        let c_wishlist = c.clone();
                        let c_id_tradable = c.id.clone();

                        rsx! {
                            div {
                                class: if *is_closing.read() { "fixed inset-0 bg-slate-950/80 flex items-center justify-center p-4 z-50 transition-all animate-backdrop-exit outline-none" } else { "fixed inset-0 bg-slate-950/80 flex items-center justify-center p-4 z-50 transition-all animate-backdrop-enter outline-none" },
                                tabindex: "0",
                                autofocus: "true",
                                onmounted: onmounted_loop,
                                onkeydown: move |evt| {
                                    if evt.key() == Key::ArrowLeft {
                                        // Free, continuous gallery scroll (no snapping) — holding the
                                        // key keeps adding velocity, releasing lets friction settle it.
                                        snap_target.set(None);
                                        anim_t.set(1.0);
                                        let v = (*vel.read() - 0.09).max(-MAX_VEL);
                                        vel.set(v);
                                    } else if evt.key() == Key::ArrowRight {
                                        snap_target.set(None);
                                        anim_t.set(1.0);
                                        let v = (*vel.read() + 0.09).min(MAX_VEL);
                                        vel.set(v);
                                    } else if evt.key() == Key::Escape {
                                        close_modal_action(is_closing, selected_card_id, show_accounts);
                                    }
                                },

                                div {
                                    class: "absolute inset-0 z-0",
                                    onclick: move |_| close_modal_action(is_closing, selected_card_id, show_accounts)
                                }

                                div {
                                    class: if *is_closing.read() { "glass-panel rounded-3xl w-full max-w-lg p-6 md:p-8 flex flex-col items-center gap-6 shadow-[0_20px_50px_rgba(0,0,0,0.5)] animate-card-exit relative z-10" } else { "glass-panel rounded-3xl w-full max-w-lg p-6 md:p-8 flex flex-col items-center gap-6 shadow-[0_20px_50px_rgba(0,0,0,0.5)] animate-card-enter relative z-10" },

                                    button {
                                        class: "absolute top-4 right-4 text-slate-400 hover:text-white bg-white/5 hover:bg-white/10 rounded-full w-8 h-8 flex items-center justify-center transition-colors active:scale-95 border border-white/10 z-30",
                                        onpointerdown: move |evt| { evt.stop_propagation(); },
                                        onclick: move |_| close_modal_action(is_closing, selected_card_id, show_accounts),
                                        "✕"
                                    }

                                    // Infinite horizontal film strip (slot-machine coverflow)
                                    div {
                                        class: "relative w-full h-[330px] md:h-[440px] flex items-center justify-center my-2 select-none",
                                        style: "perspective: 1000px; transform-style: preserve-3d; touch-action: manipulation;",
                                        onwheel: move |evt| {
                                            evt.prevent_default();
                                            evt.stop_propagation();
                                            let pixels = evt.delta().strip_units().y;
                                            if pixels.abs() > 0.5 {
                                                let step = -pixels / CARD_STEP;
                                                pos.with_mut(|p| *p += step);
                                                vel.with_mut(|v| *v = (*v + step * 0.3).clamp(-MAX_VEL, MAX_VEL));
                                                dragging.set(false);
                                                snap_target.set(None);
                                                anim_t.set(1.0);
                                            }
                                        },
                                        onpointerdown: move |evt| {
                                            evt.stop_propagation();
                                            dragging.set(true);
                                            dragged.set(false);
                                            snap_target.set(None);
                                            anim_t.set(1.0);
                                            vel.set(0.0);
                                            last_x.set(evt.client_coordinates().x);
                                        },
                                        onpointermove: move |evt| {
                                            if *dragging.read() {
                                                let x = evt.client_coordinates().x;
                                                let dx = x - *last_x.read();
                                                if dx.abs() > 2.0 {
                                                    dragged.set(true);
                                                }
                                                last_x.set(x);
                                                let dpos = -dx / CARD_STEP;
                                                pos.with_mut(|p| *p += dpos);
                                                vel.set(dpos);
                                            }
                                        },
                                        onpointerup: move |_| { dragging.set(false); },
                                        onpointercancel: move |_| { dragging.set(false); },
                                        onpointerleave: move |_| { if *dragging.read() { dragging.set(false); } },

                                        for s in 0..N {
                                            {
                                                let rel = s as i32 - VISIBLE_RADIUS;
                                                let target_id = ids.get(s).cloned().unwrap_or_default();
                                                let is_center = rel == 0;
                                                let transform_style = slot_transform(rel, frac);

                                                rsx! {
                                                    div {
                                                        key: "{s}",
                                                        class: "absolute cursor-pointer flex items-center justify-center",
                                                        style: "{transform_style}",
                                                        onclick: move |evt| {
                                                            evt.stop_propagation();
                                                            if !*dragged.read() && !*dragging.read() && snap_target.read().is_none() && rel != 0 {
                                                                let target = pos.read().round() + rel as f64;
                                                                anim_from.set(*pos.read());
                                                                anim_t.set(0.0);
                                                                vel.set(0.0);
                                                                dragging.set(false);
                                                                snap_target.set(Some(target));
                                                            }
                                                        },
                                                        if let Some(target_api_card) = api_map.get(&target_id) {
                                                            {
                                                                let item_image_url = optimized_image_url(&target_api_card.full_image_url, 400);
                                                                let img_class = if is_center {
                                                                    "w-44 md:w-56 rounded-2xl shadow-2xl drop-shadow-[0_20px_30px_rgba(0,0,0,0.5)] border-2 border-white/15 object-cover pointer-events-none ring-2 ring-indigo-500/30"
                                                                } else {
                                                                    "w-36 md:w-48 rounded-2xl shadow-xl drop-shadow-[0_10px_20px_rgba(0,0,0,0.4)] border border-white/5 object-cover pointer-events-none"
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

                                    // Details (name / rarity / pack / actions) are rendered only once the
                                    // carousel has settled. While scrolling we show just the coverflow
                                    // animation, avoiding per-frame layout thrash and content flicker.
                                    div { class: "w-full flex flex-col items-center min-h-[200px] relative z-[60]",
                                        if *settled.read() {
                                            {
                                                rsx! {
                                    // Title & Pack info
                                    div { class: "flex flex-col items-center w-full gap-1",
                                        h2 { class: "text-2xl font-black text-white text-center tracking-tight", "{display_api_card.name}" }
                                        div { class: "flex items-center gap-3",
                                            RarityDisplay { rarity_code: display_api_card.rarity.clone() }
                                            div { class: "w-1 h-1 rounded-full bg-slate-600" }
                                            img {
                                                src: "https://raw.githubusercontent.com/flibustier/pokemon-tcg-pocket-database/main/dist/images/sets/LOGO_expansion_{display_api_card.set}_en_US.webp",
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
                                            onpointerdown: move |evt| { evt.stop_propagation(); },
                                            onclick: move |_| { collection.write().toggle_wishlist(c_wishlist.clone()); },
                                            svg { class: "w-5 h-5 transition-all", class: if is_wishlisted { "fill-pink-400 text-pink-400" } else { "fill-none text-current" }, view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Wishlist" }
                                        }

                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if is_tradable { "bg-emerald-500/15 text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/25" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onpointerdown: move |evt| { evt.stop_propagation(); },
                                            onclick: move |_| { collection.write().toggle_tradable(&c_id_tradable.clone()); },
                                            svg { class: "w-5 h-5 transition-all", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Tradable" }
                                        }

                                        button {
                                            class: "flex flex-col items-center justify-center gap-1.5 p-3 rounded-xl transition-all active:scale-[0.95] border",
                                            class: if *show_accounts.read() { "bg-sky-500/15 text-sky-400 border-sky-500/20" } else { "bg-white/5 text-slate-400 border-white/10 hover:bg-white/10 hover:text-slate-200" },
                                            onpointerdown: move |evt| { evt.stop_propagation(); },
                                            onclick: move |_| { show_accounts.set(!show_accounts()); },
                                            svg { class: "w-5 h-5 transition-all", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                            }
                                            span { class: "text-[10px] font-bold tracking-wide uppercase", "Accounts" }
                                        }
                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                 // --- ACCOUNTS SIDE MENU ---
                                 if *show_accounts.read() {
                                     div { class: "fixed inset-0 bg-slate-950/50 backdrop-blur-md z-[70] animate-fade-in", onclick: move |_| show_accounts.set(false) }
                                     div { class: "fixed inset-y-0 right-0 w-80 glass-panel border-l border-white/10 shadow-2xl z-[80] p-6 flex flex-col animate-slide-in-right",

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
                                                                            let res = collection.write().remove_card(&target_card, &target_owner, 1);
                                                                            if res.is_ok() {
                                                                                toast_message.set(Some(format!("🗑️ Removed {} from {}", c_name, target_owner)));
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
                    } else {
                        rsx! { div { onmounted: move |_| selected_card_id.set(None) } }
                    }
                }
            }
        }
    }
}
