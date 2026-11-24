use dioxus::prelude::*;
use crate::models::*;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct SearchInputProps {
    pub search_query: Signal<String>,
}

#[component]
pub fn SearchInput(mut props: SearchInputProps) -> Element {
    // Local raw input that updates instantly so the input never lags behind typing
    let mut raw_input = use_signal(|| props.search_query.read().clone());
    // Monotonic token used to cancel stale debounce timers (only the last keystroke wins)
    let mut debounce_token = use_signal(|| 0u32);

    rsx! {
        div { class: "relative group w-full",
            div { class: "absolute inset-y-0 left-0 flex items-center pl-3.5 pointer-events-none text-slate-500 group-focus-within:text-accent transition-colors",
                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-4 h-4",
                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                }
            }
            input {
                class: "field pl-10",
                r#type: "search",
                placeholder: "Search cards",
                value: "{raw_input}",
                oninput: move |evt| {
                    let val = evt.value();
                    raw_input.set(val.clone());
                    // Debounce the global query so the (potentially huge) grid only
                    // re-renders after the user pauses typing, instead of every keystroke.
                    debounce_token.with_mut(|t| *t += 1);
                    let token = *debounce_token.read();
                    let mut search_query = props.search_query;
                    spawn(async move {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(160)).await;
                        if *debounce_token.read() == token {
                            search_query.set(val);
                        }
                    });
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        // Apply immediately on Enter
                        debounce_token.with_mut(|t| *t += 1);
                        props.search_query.set(raw_input.read().clone());
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct FilterButtonProps {
    pub show_filter_menu: Signal<bool>,
}

#[component]
pub fn FilterButton(mut props: FilterButtonProps) -> Element {
    rsx! {
        button {
            class: "icon-btn",
            title: "Filters",
            onclick: move |_| {
                let current = *props.show_filter_menu.read();
                props.show_filter_menu.set(!current);
            },
            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.75", stroke: "currentColor", class: "w-5 h-5",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct FilterTrayProps {
    pub show_filter_menu: Signal<bool>,
    pub selected_account_filter: Signal<String>,
    pub selected_rarities: Signal<Vec<String>>,
    pub selected_types: Signal<Vec<String>>,
    pub selected_packs: Signal<Vec<String>>,
    pub collection: Signal<CardCollection>,
    pub pack_db: Signal<Option<Vec<PackSet>>>,
}

// All rarity codes in display order (highest to lowest priority)
const ALL_RARITIES: &[&str] = &["UR", "SSR", "S", "IM", "SAR", "SR", "AR", "RR", "R", "U", "C"];

// Card types for the type filter
const ALL_TYPES: &[&str] = &["Pokémon", "Trainer"];

#[component]
pub fn FilterTray(mut props: FilterTrayProps) -> Element {
    if !*props.show_filter_menu.read() {
        return rsx! {};
    }

    rsx! {
        if *props.show_filter_menu.read() {
            // Backdrop
            div {
                class: "fixed inset-0 scrim z-[60] animate-fade-in",
                onclick: move |_| props.show_filter_menu.set(false),
            }

            // Drawer
            div { class: "acrylic-strong fixed top-3 bottom-3 right-3 w-[22rem] max-w-[calc(100vw-1.5rem)] rounded-3xl z-[70] flex flex-col animate-slide-in-right",

                // Header
                div { class: "flex items-center justify-between pl-6 pr-3 py-3 border-b border-white/[0.06]",
                    h2 { class: "text-lg font-semibold text-white tracking-tight", "Filters" }
                    button {
                        class: "icon-btn is-bare",
                        title: "Close filters",
                        onclick: move |_| props.show_filter_menu.set(false),
                        svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                        }
                    }
                }

                // Scrollable Content
                div { class: "flex-1 overflow-y-auto px-6 py-5 flex flex-col gap-7",

                    // --- Account ---
                    div { class: "flex flex-col gap-2.5",
                        label { class: "field-label", "Account" }
                        select {
                            class: "field",
                            onchange: move |evt| props.selected_account_filter.set(evt.value()),
                            option { value: "All", "All accounts" }
                            for account in props.collection.read().accounts.iter() {
                                option { value: "{account.name}", "{account.name}" }
                            }
                        }
                    }

                    // --- Rarity ---
                    div { class: "flex flex-col gap-2.5",
                        div { class: "flex items-center justify-between",
                            label { class: "field-label", "Rarity" }
                            if !props.selected_rarities.read().is_empty() {
                                button {
                                    class: "text-link",
                                    onclick: move |_| props.selected_rarities.set(Vec::new()),
                                    "Clear"
                                }
                            }
                        }
                        div { class: "flex flex-wrap gap-2",
                            for code in ALL_RARITIES.iter() {
                                {
                                    let code_str = code.to_string();
                                    let code_for_click = code_str.clone();
                                    let is_active = props.selected_rarities.read().contains(&code_str);
                                    let is_sar = *code == "SAR";

                                    rsx! {
                                        button {
                                            class: "chip",
                                            class: if is_sar { "chip-sar" },
                                            class: if is_active { "is-on" },
                                            onclick: move |_| {
                                                let mut current = props.selected_rarities.read().clone();
                                                if current.contains(&code_for_click) {
                                                    current.retain(|c| c != &code_for_click);
                                                } else {
                                                    current.push(code_for_click.clone());
                                                }
                                                props.selected_rarities.set(current);
                                            },
                                            RarityDisplay { rarity_code: code_str }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // --- Type ---
                    div { class: "flex flex-col gap-2.5",
                        div { class: "flex items-center justify-between",
                            label { class: "field-label", "Type" }
                            if !props.selected_types.read().is_empty() {
                                button {
                                    class: "text-link",
                                    onclick: move |_| props.selected_types.set(Vec::new()),
                                    "Clear"
                                }
                            }
                        }
                        div { class: "flex gap-2",
                            for type_name in ALL_TYPES.iter() {
                                {
                                    let type_str = type_name.to_string();
                                    let type_for_click = type_str.clone();
                                    let is_active = props.selected_types.read().contains(&type_str);

                                    rsx! {
                                        button {
                                            class: "chip flex-1",
                                            class: if is_active { "is-on" },
                                            onclick: move |_| {
                                                let mut current = props.selected_types.read().clone();
                                                if current.contains(&type_for_click) {
                                                    current.retain(|c| c != &type_for_click);
                                                } else {
                                                    current.push(type_for_click.clone());
                                                }
                                                props.selected_types.set(current);
                                            },
                                            "{type_str}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // --- Sets ---
                    if let Some(packs) = props.pack_db.read().as_ref() {
                        div { class: "flex flex-col gap-2.5 pb-4",
                            div { class: "flex items-center justify-between",
                                label { class: "field-label", "Sets" }
                                if !props.selected_packs.read().is_empty() {
                                    button {
                                        class: "text-link",
                                        onclick: move |_| props.selected_packs.set(Vec::new()),
                                        "Clear"
                                    }
                                }
                            }
                            div { class: "grid grid-cols-2 gap-2",
                                for pack in packs.iter() {
                                    {
                                        let code_for_click = pack.code.clone();
                                        let is_active = props.selected_packs.read().contains(&code_for_click);

                                        let flib_code = match pack.code.as_str() {
                                            "P-A" => "PROMO-A",
                                            "P-B" => "PROMO-B",
                                            "P-C" => "PROMO-C",
                                            other => other,
                                        };
                                        let img_src = format!("https://cdn.jsdelivr.net/gh/flibustier/pokemon-tcg-pocket-database@main/dist/images/sets/LOGO_expansion_{}_en_US.webp", flib_code);

                                        let title = pack.name.get("en").cloned().unwrap_or_else(|| pack.code.clone());

                                        rsx! {
                                            button {
                                                class: "h-14 flex items-center justify-center p-2 rounded-2xl border transition",
                                                class: if is_active { "border-accent bg-accent/10" } else { "border-white/[0.08] bg-white/[0.03] hover:bg-white/[0.07] opacity-60 grayscale hover:opacity-100 hover:grayscale-0" },
                                                onclick: move |_| {
                                                    let mut current = props.selected_packs.read().clone();
                                                    if current.contains(&code_for_click) {
                                                        current.retain(|c| c != &code_for_click);
                                                    } else {
                                                        current.push(code_for_click.clone());
                                                    }
                                                    props.selected_packs.set(current);
                                                },
                                                title: "{title}",
                                                img { src: "{img_src}", alt: "{title}", class: "max-h-full max-w-full object-contain" }
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
