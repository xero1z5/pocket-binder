use dioxus::prelude::*;
use crate::models::*;

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
        div { class: "fixed bottom-4 md:bottom-6 inset-x-0 z-50 flex justify-center px-3 pointer-events-none",
            div { class: "acrylic pointer-events-auto rounded-[1.75rem] px-2 py-1.5 flex items-center gap-0.5 sm:gap-1.5 animate-slide-up",

                DockItem {
                    label: "Collection".to_string(),
                    icon: "M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25".to_string(),
                    is_active: *props.active_view.read() == "collection",
                    onclick: move |_| {
                        props.active_view.set("collection".to_string());
                        show_packs.set(false);
                    },
                }

                DockItem {
                    label: "Wishlist".to_string(),
                    icon: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z".to_string(),
                    is_active: *props.active_view.read() == "wishlist",
                    onclick: move |_| {
                        props.active_view.set("wishlist".to_string());
                        show_packs.set(false);
                    },
                }

                DockItem {
                    label: "Tradable".to_string(),
                    icon: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.016 4.356v4.992".to_string(),
                    is_active: *props.active_view.read() == "tradable",
                    onclick: move |_| {
                        props.active_view.set("tradable".to_string());
                        show_packs.set(false);
                    },
                }

                // Add cards: the Poké Ball
                button {
                    class: "pokeball",
                    onclick: move |_| props.show_add_modal.set(true),
                    span { class: "dock-tip", "Add cards" }
                }

                DockItem {
                    label: "Trade".to_string(),
                    icon: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5".to_string(),
                    is_active: false,
                    onclick: move |_| props.show_trade_modal.set(true),
                }

                DockItem {
                    label: "Sets".to_string(),
                    icon: "M2.25 7.125C2.25 6.504 2.754 6 3.375 6h6c.621 0 1.125.504 1.125 1.125v3.75c0 .621-.504 1.125-1.125 1.125h-6a1.125 1.125 0 01-1.125-1.125v-3.75zM14.25 8.625c0-.621.504-1.125 1.125-1.125h5.25c.621 0 1.125.504 1.125 1.125v8.25c0 .621-.504 1.125-1.125 1.125h-5.25a1.125 1.125 0 01-1.125-1.125v-8.25zM3.75 16.125c0-.621.504-1.125 1.125-1.125h5.25c.621 0 1.125.504 1.125 1.125v2.25c0 .621-.504 1.125-1.125 1.125h-5.25a1.125 1.125 0 01-1.125-1.125v-2.25z".to_string(),
                    is_active: props.active_view.read().starts_with("pack:"),
                    onclick: move |_| show_packs.set(!show_packs()),
                }

                DockItem {
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

        // Sets popup menu
        if *show_packs.read() {
            if let Some(packs) = props.pack_db.read().as_ref() {
                div { class: "fixed inset-0 z-30", onclick: move |_| show_packs.set(false) }
                div { class: "fixed bottom-24 md:bottom-28 inset-x-0 z-40 flex justify-center px-3 pointer-events-none",
                    div { class: "acrylic-strong pointer-events-auto rounded-3xl p-2 w-72 md:w-80 max-h-[60vh] overflow-y-auto animate-fade-in-up",
                        div { class: "flex flex-col gap-1",
                            for pack in packs.iter() {
                                {
                                    let code_for_click = pack.code.clone();
                                    let flib_code = match pack.code.as_str() {
                                        "P-A" => "PROMO-A",
                                        "P-B" => "PROMO-B",
                                        "P-C" => "PROMO-C",
                                        other => other,
                                    };
                                    let img_src = format!("https://cdn.jsdelivr.net/gh/flibustier/pokemon-tcg-pocket-database@main/dist/images/sets/LOGO_expansion_{}_en_US.webp", flib_code);
                                    let title = pack.name.get("en").cloned().unwrap_or_else(|| pack.code.clone());
                                    let is_active = *props.active_view.read() == format!("pack:{}", code_for_click);

                                    rsx! {
                                        button {
                                            class: "row is-plain",
                                            class: if is_active { "is-on" },
                                            onclick: move |_| {
                                                props.active_view.set(format!("pack:{}", code_for_click));
                                                show_packs.set(false);
                                            },
                                            img { src: "{img_src}", alt: "", class: "h-7 w-20 object-contain flex-shrink-0" }
                                            span { class: "truncate flex-1 text-sm", "{title}" }
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
fn DockItem(label: String, icon: String, is_active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "dock-item",
            class: if is_active { "is-on" },
            onclick: move |e| onclick.call(e),
            svg { class: "w-6 h-6",
                fill: "none", view_box: "0 0 24 24", stroke_width: if is_active { "2" } else { "1.6" }, stroke: "currentColor",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "{icon}" }
            }
            span { class: "dock-tip", "{label}" }
        }
    }
}
