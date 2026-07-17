use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashSet;
use crate::models::*;
use crate::supabase::*;

#[derive(PartialEq, Clone, Props)]
pub struct NavigationProps {
    pub show_account_modal: Signal<bool>,
    pub show_add_modal: Signal<bool>,
    pub show_trade_modal: Signal<bool>,
    pub auth_token: Signal<String>,
    pub user_email: Signal<String>,
    pub show_login_modal: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub sync_status: Signal<String>,
    pub active_view: Signal<String>,
    pub pack_db: Signal<Option<Vec<PackSet>>>,
}

// ---------------------------------------------------------
// FLOATING DOCK (Modern unified navigation)
// ---------------------------------------------------------
#[component]
pub fn FloatingDock(mut props: NavigationProps) -> Element {
    let mut show_packs = use_signal(|| false);
    
    rsx! {
        div { class: "fixed bottom-4 md:bottom-8 left-1/2 -translate-x-1/2 z-50 rounded-[2rem] px-4 md:px-6 py-2.5 md:py-3 flex items-center gap-2 md:gap-6 shadow-elevated border border-white/10 animate-slide-up backdrop-blur-xl bg-slate-900/40",
            
            DockItem {
                label: "Collection".to_string(),
                icon: "M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25".to_string(),
                is_active: *props.active_view.read() == "collection",
                color_theme: "sky".to_string(),
                onclick: move |_| {
                    props.active_view.set("collection".to_string());
                    show_packs.set(false);
                },
            }

            DockItem {
                label: "Wishlist".to_string(),
                icon: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z".to_string(),
                is_active: *props.active_view.read() == "wishlist",
                color_theme: "pink".to_string(),
                onclick: move |_| {
                    props.active_view.set("wishlist".to_string());
                    show_packs.set(false);
                },
            }
            
            DockItem {
                label: "Tradable".to_string(),
                icon: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.016 4.356v4.992".to_string(),
                is_active: *props.active_view.read() == "tradable",
                color_theme: "emerald".to_string(),
                onclick: move |_| {
                    props.active_view.set("tradable".to_string());
                    show_packs.set(false);
                },
            }

            // Divider
            div { class: "w-px h-8 bg-white/10 mx-1 md:mx-2 hidden sm:block" }

            DockItem {
                label: "Add Card".to_string(),
                icon: "M12 4.5v15m7.5-7.5h-15".to_string(),
                is_active: false,
                color_theme: "slate".to_string(),
                onclick: move |_| props.show_add_modal.set(true),
            }

            DockItem {
                label: "Trade".to_string(),
                icon: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5".to_string(),
                is_active: false,
                color_theme: "slate".to_string(),
                onclick: move |_| props.show_trade_modal.set(true),
            }

            // Sets/Packs toggle
            div { class: "relative" }
            DockItem {
                label: "Sets".to_string(),
                icon: "M2.25 7.125C2.25 6.504 2.754 6 3.375 6h6c.621 0 1.125.504 1.125 1.125v3.75c0 .621-.504 1.125-1.125 1.125h-6a1.125 1.125 0 01-1.125-1.125v-3.75zM14.25 8.625c0-.621.504-1.125 1.125-1.125h5.25c.621 0 1.125.504 1.125 1.125v8.25c0 .621-.504 1.125-1.125 1.125h-5.25a1.125 1.125 0 01-1.125-1.125v-8.25zM3.75 16.125c0-.621.504-1.125 1.125-1.125h5.25c.621 0 1.125.504 1.125 1.125v2.25c0 .621-.504 1.125-1.125 1.125h-5.25a1.125 1.125 0 01-1.125-1.125v-2.25z".to_string(),
                is_active: props.active_view.read().starts_with("pack:"),
                color_theme: "fuchsia".to_string(),
                onclick: move |_| show_packs.set(!show_packs()),
            }

            // Divider
            div { class: "w-px h-8 bg-white/10 mx-1 md:mx-2 hidden sm:block" }

            DockItem {
                label: "Account".to_string(),
                icon: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z".to_string(),
                is_active: false,
                color_theme: "slate".to_string(),
                onclick: move |_| {
                    if props.auth_token.read().is_empty() {
                        props.show_login_modal.set(true);
                    } else {
                        props.show_account_modal.set(true);
                    }
                },
            }
        }
        
        // Sets popup menu
        if *show_packs.read() {
            if let Some(packs) = props.pack_db.read().as_ref() {
                div { class: "fixed bottom-24 left-1/2 -translate-x-1/2 z-40 bg-slate-950/90 backdrop-blur-3xl rounded-2xl p-2 max-h-[60vh] overflow-y-auto shadow-2xl border border-white/10 animate-zoom-in w-64 md:w-80",
                    div { class: "flex flex-col gap-1",
                        for pack in packs.iter() {
                            {
                                let code_for_click = pack.code.clone();
                                let flib_code = match pack.code.as_str() {
                                    "P-A" => "PROMO-A",
                                    "P-B" => "PROMO-B",
                                    other => other,
                                };
                                let img_src = format!("https://cdn.jsdelivr.net/gh/flibustier/pokemon-tcg-pocket-database@main/dist/images/sets/LOGO_expansion_{}_en_US.webp", flib_code);
                                let title = pack.name.get("en").cloned().unwrap_or_else(|| pack.code.clone());
                                let is_active = *props.active_view.read() == format!("pack:{}", code_for_click);
                                
                                rsx! {
                                    button {
                                        class: "flex items-center gap-3 px-3 py-2 rounded-xl transition-all text-left group",
                                        class: if is_active { "bg-fuchsia-500/10 border-fuchsia-400/30 border" } else { "hover:bg-white/5 border border-transparent" },
                                        onclick: move |_| {
                                            props.active_view.set(format!("pack:{}", code_for_click));
                                            show_packs.set(false);
                                        },
                                        img { src: "{img_src}", class: "h-6 w-16 object-contain" }
                                        span { class: "truncate flex-1 text-sm font-medium", class: if is_active { "text-white" } else { "text-slate-300 group-hover:text-white" }, "{title}" }
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
fn DockItem(label: String, icon: String, is_active: bool, color_theme: String, onclick: EventHandler<MouseEvent>) -> Element {
    let (bg_glow, text_glow, dot_color) = match color_theme.as_str() {
        "sky" => ("bg-sky-500/30", "text-sky-400 drop-shadow-[0_0_8px_rgba(56,189,248,0.5)]", "bg-sky-400"),
        "pink" => ("bg-pink-500/30", "text-pink-400 drop-shadow-[0_0_8px_rgba(244,114,182,0.5)]", "bg-pink-400"),
        "emerald" => ("bg-emerald-500/30", "text-emerald-400 drop-shadow-[0_0_8px_rgba(52,211,153,0.5)]", "bg-emerald-400"),
        "fuchsia" => ("bg-fuchsia-500/30", "text-fuchsia-400 drop-shadow-[0_0_8px_rgba(232,121,249,0.5)]", "bg-fuchsia-400"),
        _ => ("bg-slate-500/30", "text-slate-300 drop-shadow-[0_0_8px_rgba(203,213,225,0.5)]", "bg-slate-300"),
    };

    rsx! {
        button {
            class: "flex flex-col items-center justify-center group relative transition-all duration-300 outline-none p-2 rounded-2xl",
            class: if is_active { "bg-white/10" } else { "hover:bg-white/5" },
            onclick: move |e| onclick.call(e),
            
            // Icon
            div { class: "relative flex items-center justify-center transition-transform duration-300",
                class: if is_active { "-translate-y-1 scale-110" } else { "group-hover:-translate-y-1 group-hover:scale-110" },
                
                // Active glow
                if is_active {
                    div { class: "absolute -inset-2 {bg_glow} blur-lg rounded-full" }
                }
                
                svg { class: "w-6 h-6 md:w-7 md:h-7 transition-colors z-10", 
                    class: if is_active { "{text_glow}" } else { "text-slate-400 group-hover:text-slate-200" }, 
                    fill: "none", view_box: "0 0 24 24", stroke_width: if is_active { "2" } else { "1.5" }, stroke: "currentColor",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "{icon}" }
                }
            }
            
            // Dot indicator instead of text label to save space
            div { class: "h-1 w-1 rounded-full mt-1.5 transition-all duration-300",
                class: if is_active { "{dot_color} scale-100 opacity-100" } else { "bg-slate-400 scale-0 opacity-0 group-hover:scale-100 group-hover:opacity-50" }
            }
            
            // Tooltip on hover
            div { class: "absolute -top-10 scale-0 opacity-0 group-hover:scale-100 group-hover:opacity-100 transition-all duration-200 bg-slate-800 text-white text-[10px] font-bold px-2 py-1 rounded-lg pointer-events-none whitespace-nowrap shadow-xl border border-white/10",
                "{label}"
            }
        }
    }
}

