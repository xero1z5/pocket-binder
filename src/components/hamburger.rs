use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::models::*;
use crate::supabase::*;

#[derive(PartialEq, Clone, Props)]
pub struct HamburgerMenuProps {
    pub show_account_modal: Signal<bool>,
    pub show_add_modal: Signal<bool>,
    pub show_trade_modal: Signal<bool>,
    pub auth_token: Signal<String>,
    pub user_email: Signal<String>,
    pub show_login_modal: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub sync_status: Signal<String>,
}

#[component]
pub fn HamburgerMenu(mut props: HamburgerMenuProps) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        // Hamburger Toggle Button
        button {
            class: "group w-11 h-11 md:w-12 md:h-12 flex flex-col items-center justify-center gap-[5px] bg-slate-800/60 border border-indigo-500/20 rounded-xl hover:bg-slate-700/80 hover:border-indigo-400/40 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| is_open.set(true),
            // Three-line hamburger icon
            span { class: "block w-5 h-[2px] bg-slate-400 group-hover:bg-indigo-400 transition-colors rounded-full" }
            span { class: "block w-5 h-[2px] bg-slate-400 group-hover:bg-indigo-400 transition-colors rounded-full" }
            span { class: "block w-5 h-[2px] bg-slate-400 group-hover:bg-indigo-400 transition-colors rounded-full" }
        }

        if *is_open.read() {
            // Backdrop overlay
            div {
                class: "fixed inset-0 bg-slate-950/60 z-40 backdrop-blur-sm animate-fade-in",
                onclick: move |_| is_open.set(false),
            }

            // Slide-out sidebar panel
            div { class: "fixed top-0 right-0 h-full w-72 md:w-80 bg-slate-900/95 border-l border-indigo-500/20 z-50 flex flex-col shadow-2xl backdrop-blur-xl animate-slide-in-right",

                // Header area
                div { class: "flex items-center justify-between p-5 border-b border-indigo-500/10",
                    h2 { class: "text-lg font-bold text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-purple-300 tracking-tight", "Menu" }
                    button {
                        class: "p-2 text-slate-500 hover:text-white hover:bg-slate-800 rounded-xl transition-all",
                        onclick: move |_| is_open.set(false),
                        svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                        }
                    }
                }

                // User info (if logged in)
                if !props.auth_token.read().is_empty() {
                    div { class: "px-5 py-3 border-b border-indigo-500/10",
                        div { class: "flex items-center gap-3",
                            div { class: "w-8 h-8 rounded-lg bg-indigo-500/20 flex items-center justify-center flex-shrink-0",
                                svg { class: "w-4 h-4 text-indigo-400", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                }
                            }
                            span { class: "text-xs text-indigo-400 font-mono font-bold truncate", "{props.user_email}" }
                        }
                    }
                }

                // Menu items
                div { class: "flex-1 flex flex-col p-3 gap-1 overflow-y-auto",

                    // Accounts
                    MenuItemButton {
                        label: "My Accounts",
                        icon_path: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z",
                        onclick: move |_| {
                            props.show_account_modal.set(true);
                            is_open.set(false);
                        },
                    }

                    // Add Cards
                    MenuItemButton {
                        label: "Add Cards",
                        icon_path: "M12 4.5v15m7.5-7.5h-15",
                        onclick: move |_| {
                            props.show_add_modal.set(true);
                            is_open.set(false);
                        },
                    }

                    // Trade Room
                    MenuItemButton {
                        label: "Trade Room",
                        icon_path: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5",
                        onclick: move |_| {
                            props.show_trade_modal.set(true);
                            is_open.set(false);
                        },
                    }

                    // Divider
                    div { class: "h-px bg-indigo-500/10 my-2" }

                    // Refresh
                    MenuItemButton {
                        label: "Refresh Collection",
                        icon_path: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.016 4.356v4.992",
                        onclick: move |_| {
                            let token = props.auth_token.read().clone();
                            if !token.is_empty() {
                                props.sync_status.set("🔄 Refreshing...".to_string());
                                spawn(async move {
                                    if let Ok(data) = load_from_supabase(&token).await {
                                        props.collection.set(data);
                                        props.sync_status.set("✅ Refreshed!".to_string());
                                    } else {
                                        props.sync_status.set("❌ Refresh failed.".to_string());
                                    }
                                });
                            }
                            is_open.set(false);
                        },
                    }
                }

                // Bottom: Logout / Login
                div { class: "p-4 border-t border-indigo-500/10 mt-auto",
                    if props.auth_token.read().is_empty() {
                        button {
                            class: "w-full py-3 px-4 rounded-xl font-bold text-sm bg-gradient-to-r from-indigo-500 to-purple-500 hover:from-indigo-400 hover:to-purple-400 text-white transition-all shadow-[0_0_15px_rgba(99,102,241,0.3)]",
                            onclick: move |_| {
                                props.show_login_modal.set(true);
                                is_open.set(false);
                            },
                            "Login"
                        }
                    } else {
                        button {
                            class: "w-full py-3 px-4 rounded-xl font-medium text-sm text-slate-400 bg-slate-800/60 border border-slate-700 hover:text-rose-400 hover:border-rose-500/30 hover:bg-rose-500/10 transition-all",
                            onclick: move |_| {
                                props.auth_token.set(String::new());
                                props.user_email.set(String::new());
                                LocalStorage::delete("supabase_token");
                                LocalStorage::delete("user_email");
                                is_open.set(false);
                            },
                            "Logout"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MenuItemButton(label: String, icon_path: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "w-full flex items-center gap-3 px-4 py-3 rounded-xl text-slate-300 hover:text-white hover:bg-indigo-500/10 transition-all group",
            onclick: move |e| onclick.call(e),
            div { class: "w-9 h-9 rounded-lg bg-slate-800/80 group-hover:bg-indigo-500/20 flex items-center justify-center transition-colors flex-shrink-0",
                svg { class: "w-5 h-5 text-slate-400 group-hover:text-indigo-400 transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "{icon_path}" }
                }
            }
            span { class: "font-medium text-sm", "{label}" }
        }
    }
}
