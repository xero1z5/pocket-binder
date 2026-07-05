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
    pub is_sidebar_expanded: Signal<bool>,
}

// ---------------------------------------------------------
// DESKTOP SIDEBAR (Visible on md and up)
// ---------------------------------------------------------
#[component]
pub fn DesktopSidebar(mut props: NavigationProps) -> Element {
    let mut show_packs = use_signal(|| false);
    let mut failed_images = use_signal(|| HashSet::new());
    
    let is_expanded = *props.is_sidebar_expanded.read();

    rsx! {
        div { 
            class: "hidden md:flex flex-col fixed inset-y-0 left-0 z-40 glass-panel shadow-2xl overflow-y-auto overflow-x-hidden transition-all duration-300",
            class: if is_expanded { "w-64 lg:w-72 border-r border-teal-500/20" } else { "w-20 border-r border-teal-500/20 items-center" },
            
            // Header with toggle
            div { class: "p-4 flex items-center justify-between border-b border-white/5 w-full",
                class: if is_expanded { "justify-between" } else { "justify-center" },
                if is_expanded {
                    h1 { class: "text-2xl font-black bg-clip-text text-transparent bg-gradient-to-r from-teal-400 to-cyan-400 drop-shadow-md tracking-tighter whitespace-nowrap",
                        "POCKET BINDER"
                    }
                }
                button {
                    class: "p-2 rounded-lg hover:bg-white/10 text-slate-400 hover:text-white transition-colors flex-shrink-0",
                    onclick: move |_| props.is_sidebar_expanded.set(!is_expanded),
                    svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                        // chevron-double-left if expanded, chevron-double-right if collapsed
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: if is_expanded { "M18.75 19.5l-7.5-7.5 7.5-7.5m-6 15L5.25 12l7.5-7.5" } else { "M11.25 4.5l7.5 7.5-7.5 7.5m-6-15l7.5 7.5-7.5 7.5" } }
                    }
                }
            }

            // User Profile
            if !props.auth_token.read().is_empty() {
                div { class: "border-b border-white/5 mx-2 my-2 rounded-xl bg-white/5 backdrop-blur-md flex items-center gap-3 transition-all",
                    class: if is_expanded { "p-4" } else { "p-2 justify-center" },
                    div { class: "w-10 h-10 rounded-full bg-gradient-to-br from-teal-500 to-cyan-600 flex items-center justify-center shadow-lg border border-white/10 flex-shrink-0",
                        svg { class: "w-5 h-5 text-white", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                        }
                    }
                    if is_expanded {
                        div { class: "flex flex-col overflow-hidden",
                            span { class: "text-[10px] text-teal-300 font-bold uppercase tracking-wider", "Trainer" }
                            span { class: "text-xs font-mono text-white truncate", "{props.user_email}" }
                        }
                    }
                }
            }

            // Primary Navigation
            div { class: "flex-1 flex flex-col gap-1 p-3 mt-2 w-full",
                NavSidebarItem {
                    label: "My Collection".to_string(),
                    icon: "M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25".to_string(),
                    is_active: *props.active_view.read() == "collection",
                    onclick: move |_| props.active_view.set("collection".to_string()),
                    is_expanded,
                }
                NavSidebarItem {
                    label: "Wishlist".to_string(),
                    icon: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z".to_string(),
                    is_active: *props.active_view.read() == "wishlist",
                    onclick: move |_| props.active_view.set("wishlist".to_string()),
                    is_expanded,
                }
                NavSidebarItem {
                    label: "Tradable".to_string(),
                    icon: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.016 4.356v4.992".to_string(),
                    is_active: *props.active_view.read() == "tradable",
                    onclick: move |_| props.active_view.set("tradable".to_string()),
                    is_expanded,
                }

                div { class: "h-px bg-white/10 my-3 mx-2" }

                // Actions
                NavSidebarItem {
                    label: "Search & Add Cards".to_string(),
                    icon: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z".to_string(),
                    is_active: false,
                    onclick: move |_| props.show_add_modal.set(true),
                    is_expanded,
                }
                NavSidebarItem {
                    label: "Trade Room".to_string(),
                    icon: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5".to_string(),
                    is_active: false,
                    onclick: move |_| props.show_trade_modal.set(true),
                    is_expanded,
                }
                NavSidebarItem {
                    label: "Manage Accounts".to_string(),
                    icon: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z".to_string(),
                    is_active: false,
                    onclick: move |_| props.show_account_modal.set(true),
                    is_expanded,
                }

                div { class: "h-px bg-white/10 my-3 mx-2" }

                // Pack Tracker
                if let Some(packs) = props.pack_db.read().as_ref() {
                    div { class: "flex flex-col",
                        button { 
                            class: "px-4 py-3 rounded-xl transition-all flex items-center justify-between cursor-pointer hover:bg-white/5",
                            class: if !is_expanded { "justify-center px-0" },
                            onclick: move |_| {
                                if !is_expanded { props.is_sidebar_expanded.set(true); }
                                show_packs.set(!show_packs());
                            },
                            div { class: "flex items-center gap-3 text-slate-300",
                                svg { class: "w-5 h-5 flex-shrink-0", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M2.25 7.125C2.25 6.504 2.754 6 3.375 6h6c.621 0 1.125.504 1.125 1.125v3.75c0 .621-.504 1.125-1.125 1.125h-6a1.125 1.125 0 01-1.125-1.125v-3.75zM14.25 8.625c0-.621.504-1.125 1.125-1.125h5.25c.621 0 1.125.504 1.125 1.125v8.25c0 .621-.504 1.125-1.125 1.125h-5.25a1.125 1.125 0 01-1.125-1.125v-8.25zM3.75 16.125c0-.621.504-1.125 1.125-1.125h5.25c.621 0 1.125.504 1.125 1.125v2.25c0 .621-.504 1.125-1.125 1.125h-5.25a1.125 1.125 0 01-1.125-1.125v-2.25z" }
                                }
                                if is_expanded {
                                    span { class: "font-semibold text-sm whitespace-nowrap", "Expansions" }
                                }
                            }
                            if is_expanded {
                                svg { class: "w-4 h-4 text-slate-400 transition-transform duration-300", class: if *show_packs.read() { "rotate-180" } else { "" }, fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19.5 8.25l-7.5 7.5-7.5-7.5" }
                                }
                            }
                        }
                        if *show_packs.read() && is_expanded {
                            div { class: "flex flex-col gap-1 ml-4 mt-1 border-l-2 border-indigo-500/20 pl-2",
                                for pack in packs.iter() {
                                    {
                                        let code_for_click = pack.code.clone();
                                        let flib_code = match pack.code.as_str() {
                                            "P-A" => "PROMO-A",
                                            "P-B" => "PROMO-B",
                                            other => other,
                                        };
                                        let img_src = format!("https://cdn.jsdelivr.net/gh/flibustier/pokemon-tcg-pocket-database@main/dist/images/sets/LOGO_expansion_{}_en_US.webp", flib_code);
                                        let is_failed = failed_images.read().contains(&img_src);
                                        let title = pack.name.get("en").cloned().unwrap_or_else(|| pack.code.clone());
                                        let is_active = *props.active_view.read() == format!("pack:{}", code_for_click);
                                        
                                        rsx! {
                                            button {
                                                class: "flex items-center gap-3 px-3 py-2 rounded-lg transition-all text-left",
                                                class: if is_active { "bg-teal-500/20 shadow-inner border border-teal-400/30" } else { "hover:bg-white/5 opacity-70 hover:opacity-100" },
                                                onclick: move |_| props.active_view.set(format!("pack:{}", code_for_click)),
                                                if is_failed {
                                                    div { class: "h-5 w-14 rounded bg-white/5 border border-white/10 flex items-center justify-center font-bold text-[10px] text-indigo-300", "{pack.code}" }
                                                } else {
                                                    img {
                                                        src: "{img_src}",
                                                        alt: "{title}",
                                                        class: "h-5 w-14 object-contain drop-shadow-md",
                                                        onerror: move |_| { failed_images.write().insert(img_src.clone()); }
                                                    }
                                                }
                                                span { class: "truncate flex-1 text-xs font-medium text-slate-200", "{title}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Footer (Auth/Refresh)
            div { class: "p-4 mt-auto border-t border-white/5 flex flex-col gap-2 w-full",
                // Sync status & Refresh
                if !props.auth_token.read().is_empty() {
                    button {
                        class: "w-full py-2 px-3 rounded-lg flex items-center justify-center gap-2 text-xs font-medium bg-white/5 hover:bg-white/10 border border-white/10 transition-colors text-slate-300",
                        onclick: move |_| {
                            let token = props.auth_token.read().clone();
                            props.sync_status.set("🔄 Refreshing...".to_string());
                            spawn(async move {
                                if let Ok(data) = load_from_supabase(&token).await {
                                    props.collection.set(data);
                                    props.sync_status.set("✅ Refreshed!".to_string());
                                } else {
                                    props.sync_status.set("❌ Failed.".to_string());
                                }
                            });
                        },
                        svg { class: "w-3 h-3", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.016 4.356v4.992" }
                        }
                        "{props.sync_status}"
                    }
                }

                if props.auth_token.read().is_empty() {
                    button {
                        class: "w-full py-3 px-4 rounded-xl font-bold text-sm bg-gradient-to-r from-teal-500 to-cyan-500 hover:from-teal-400 hover:to-cyan-400 text-white transition-all shadow-[0_0_15px_rgba(20,184,166,0.3)] whitespace-nowrap",
                        class: if !is_expanded { "px-2" },
                        onclick: move |_| props.show_login_modal.set(true),
                        if is_expanded { "Login / Sync" } else { 
                            svg { class: "w-5 h-5 mx-auto", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15m3 0l3-3m0 0l-3-3m3 3H9" }
                            }
                        }
                    }
                } else {
                    button {
                        class: "w-full py-2.5 px-4 rounded-xl font-medium text-sm text-rose-300 bg-rose-500/10 border border-rose-500/20 hover:bg-rose-500/20 transition-all",
                        onclick: move |_| {
                            props.auth_token.set(String::new());
                            props.user_email.set(String::new());
                            LocalStorage::delete("supabase_token");
                            LocalStorage::delete("user_email");
                        },
                        if is_expanded { "Logout" } else {
                            svg { class: "w-5 h-5 mx-auto", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15M12 9l-3 3m0 0l3 3m-3-3h12.75" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavSidebarItem(label: String, icon: String, is_active: bool, is_expanded: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let active_class = if is_active { "active" } else { "" };
    
    rsx! {
        button {
            class: "glass-nav-item {active_class} w-full flex items-center gap-3 py-3 rounded-xl text-slate-400 hover:text-white group relative overflow-hidden transition-all duration-300",
            class: if is_expanded { "px-4" } else { "px-0 justify-center" },
            onclick: move |e| onclick.call(e),
            // Background glow effect for active state
            if is_active {
                div { class: "absolute inset-0 bg-gradient-to-r from-teal-500/20 to-transparent pointer-events-none" }
            }
            svg { class: "w-6 h-6 transition-colors z-10 flex-shrink-0", class: if is_active { "text-teal-400 drop-shadow-[0_0_8px_rgba(45,212,191,0.8)]" } else { "text-slate-500 group-hover:text-teal-300" }, fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "{icon}" }
            }
            if is_expanded {
                span { class: "font-semibold text-sm z-10 whitespace-nowrap", class: if is_active { "text-white" } else { "" }, "{label}" }
            }
        }
    }
}

// ---------------------------------------------------------
// MOBILE BOTTOM NAV (Visible on sm only)
// ---------------------------------------------------------
#[component]
pub fn MobileBottomNav(mut props: NavigationProps) -> Element {
    rsx! {
        div { class: "md:hidden fixed bottom-0 inset-x-0 z-40 glass-panel border-t border-white/10 shadow-[0_-10px_30px_rgba(0,0,0,0.5)] pb-safe pt-2 px-2 flex items-center justify-around",
            
            BottomNavItem {
                label: "Cards".to_string(),
                icon: "M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25".to_string(),
                is_active: *props.active_view.read() == "collection",
                onclick: move |_| props.active_view.set("collection".to_string()),
            }

            BottomNavItem {
                label: "Search".to_string(),
                icon: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z".to_string(),
                is_active: false, // Search opens a modal instead
                onclick: move |_| props.show_add_modal.set(true),
            }

            // Big FAB (Floating Action Button) in center for Trade
            button {
                class: "relative -top-5 flex flex-col items-center justify-center group outline-none",
                onclick: move |_| props.show_trade_modal.set(true),
                div { class: "w-14 h-14 rounded-full bg-gradient-to-br from-teal-500 to-cyan-600 flex items-center justify-center shadow-[0_0_20px_rgba(45,212,191,0.5)] border-4 border-slate-900 group-hover:scale-110 transition-transform duration-300",
                    svg { class: "w-6 h-6 text-white", fill: "none", view_box: "0 0 24 24", stroke_width: "2.5", stroke: "currentColor",
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                    }
                }
                span { class: "text-[10px] font-bold text-slate-300 mt-1 drop-shadow-md", "Trade" }
            }

            BottomNavItem {
                label: "Wishlist".to_string(),
                icon: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z".to_string(),
                is_active: *props.active_view.read() == "wishlist",
                onclick: move |_| props.active_view.set("wishlist".to_string()),
            }

            BottomNavItem {
                label: "Account".to_string(),
                icon: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z".to_string(),
                is_active: false,
                onclick: move |_| {
                    if props.auth_token.read().is_empty() {
                        props.show_login_modal.set(true);
                    } else {
                        props.show_account_modal.set(true);
                    }
                },
            }
        }
    }
}

#[component]
fn BottomNavItem(label: String, icon: String, is_active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "flex flex-col items-center justify-center w-16 h-12 transition-all outline-none",
            onclick: move |e| onclick.call(e),
            div { class: "relative flex items-center justify-center w-8 h-8",
                if is_active {
                    div { class: "absolute inset-0 bg-teal-500/30 blur-md rounded-full" }
                }
                svg { class: "w-6 h-6 transition-colors z-10", class: if is_active { "text-teal-400 drop-shadow-[0_0_8px_rgba(45,212,191,0.8)]" } else { "text-slate-500 hover:text-teal-300" }, fill: "none", view_box: "0 0 24 24", stroke_width: if is_active { "2" } else { "1.5" }, stroke: "currentColor",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "{icon}" }
                }
            }
            span { class: "text-[10px] font-medium transition-colors", class: if is_active { "text-indigo-300" } else { "text-slate-500" }, "{label}" }
        }
    }
}
