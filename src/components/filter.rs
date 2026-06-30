use dioxus::prelude::*;
use crate::models::*;
use crate::components::add_card::RarityDisplay;

#[derive(PartialEq, Clone, Props)]
pub struct SearchInputProps {
    pub search_query: Signal<String>,
}

#[component]
pub fn SearchInput(mut props: SearchInputProps) -> Element {
    // Local raw input that updates instantly (no lag for the user)
    let mut raw_input = use_signal(|| props.search_query.read().clone());

    rsx! {
        div { class: "w-full md:w-96",
            div { class: "relative group",
                div { class: "absolute inset-y-0 left-0 flex items-center pl-4 pointer-events-none text-slate-500 group-focus-within:text-indigo-400 transition-colors",
                    svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-5 h-5",
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                    }
                }
                input {
                    class: "w-full bg-slate-800/60 border border-indigo-500/20 rounded-2xl pl-11 pr-4 py-3.5 text-white focus:outline-none focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 shadow-inner transition-all placeholder-slate-500 backdrop-blur-sm",
                    placeholder: "Search card name...",
                    value: "{raw_input}",
                    oninput: move |evt| {
                        raw_input.set(evt.value());
                    },
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter {
                            props.search_query.set(raw_input.read().clone());
                        }
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
            class: "group w-11 h-11 md:w-14 md:h-14 flex flex-col items-center justify-center bg-slate-800/60 border border-indigo-500/20 rounded-xl md:rounded-2xl hover:bg-slate-700/80 hover:border-indigo-400/40 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| {
                let current = *props.show_filter_menu.read();
                props.show_filter_menu.set(!current);
            },
            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-5 h-5 md:w-6 md:h-6 text-slate-400 group-hover:text-indigo-400 transition-colors",
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
    pub collection: Signal<CardCollection>,
}

// All rarity codes in display order (highest to lowest priority)
const ALL_RARITIES: &[&str] = &["UR", "SSR", "S", "IM", "SAR", "SR", "AR", "RR", "R", "U", "C"];

// Card types for the type filter
const ALL_TYPES: &[&str] = &["Pokémon", "Trainer"];

#[component]
pub fn FilterTray(mut props: FilterTrayProps) -> Element {
    rsx! {
        if *props.show_filter_menu.read() {
            div { class: "w-full bg-slate-800/60 border border-indigo-500/20 rounded-2xl p-4 mt-4 shadow-xl backdrop-blur-xl animate-fade-in-down",
                div { class: "flex flex-col gap-5",
                    
                    // --- FILTER BY ACCOUNT ---
                    div { class: "flex flex-col gap-2",
                        label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider flex items-center gap-1", 
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-3 h-3", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" } }
                            "Account" 
                        }
                        select {
                            class: "bg-slate-900/80 border border-indigo-500/20 rounded-xl px-4 py-2 text-sm text-white focus:outline-none focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 cursor-pointer min-w-[200px] shadow-inner transition-colors",
                            onchange: move |evt| props.selected_account_filter.set(evt.value()),
                            option { value: "All", "All Accounts" }
                            for account in props.collection.read().accounts.iter() {
                                option { value: "{account.name}", "{account.name}" }
                            }
                        }
                    }

                    // --- FILTER BY RARITY (icon chips) ---
                    div { class: "flex flex-col gap-2",
                        div { class: "flex items-center justify-between",
                            label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider", "Rarity" }
                            if !props.selected_rarities.read().is_empty() {
                                button { 
                                    class: "text-[10px] text-indigo-400 hover:text-indigo-300 font-bold transition-colors",
                                    onclick: move |_| props.selected_rarities.set(Vec::new()),
                                    "Clear"
                                }
                            }
                        }
                        div { class: "flex flex-wrap gap-1.5",
                            for code in ALL_RARITIES.iter() {
                                {
                                    let code_str = code.to_string();
                                    let code_for_click = code_str.clone();
                                    let is_active = props.selected_rarities.read().contains(&code_str);
                                    let is_sar = *code == "SAR";
                                    
                                    let base = "flex items-center gap-1 px-2.5 py-1.5 rounded-lg border transition-all text-xs font-medium";
                                    let state_class = match (is_sar, is_active) {
                                        (true, true) => "sar-chip-active",
                                        (true, false) => "sar-chip",
                                        (false, true) => "bg-indigo-500/20 border-indigo-400/50 text-indigo-300 shadow-[0_0_8px_rgba(99,102,241,0.2)]",
                                        (false, false) => "bg-slate-900/50 border-slate-700/50 text-slate-500 hover:border-slate-600 hover:text-slate-400",
                                    };
                                    let full_class = format!("{base} {state_class}");

                                    rsx! {
                                        button {
                                            class: "{full_class}",
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

                    // --- FILTER BY TYPE (toggle chips) ---
                    div { class: "flex flex-col gap-2",
                        div { class: "flex items-center justify-between",
                            label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider", "Type" }
                            if !props.selected_types.read().is_empty() {
                                button { 
                                    class: "text-[10px] text-indigo-400 hover:text-indigo-300 font-bold transition-colors",
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
                                    
                                    // Choose icon for each type
                                    let icon = if *type_name == "Pokémon" { "⚡" } else { "🎴" };
                                    
                                    rsx! {
                                        button {
                                            class: "flex items-center gap-2 px-4 py-2 rounded-xl border transition-all text-sm font-medium",
                                            class: if is_active { "bg-indigo-500/20 border-indigo-400/50 text-indigo-300 shadow-[0_0_8px_rgba(99,102,241,0.2)]" } else { "bg-slate-900/50 border-slate-700/50 text-slate-500 hover:border-slate-600 hover:text-slate-400" },
                                            onclick: move |_| {
                                                let mut current = props.selected_types.read().clone();
                                                if current.contains(&type_for_click) {
                                                    current.retain(|c| c != &type_for_click);
                                                } else {
                                                    current.push(type_for_click.clone());
                                                }
                                                props.selected_types.set(current);
                                            },
                                            span { "{icon}" }
                                            "{type_str}"
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
