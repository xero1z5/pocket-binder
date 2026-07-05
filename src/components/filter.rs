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
    rsx! {
        if *props.show_filter_menu.read() {
            // Backdrop
            div { 
                class: "fixed inset-0 bg-slate-950/60 z-[60] backdrop-blur-sm animate-fade-in",
                onclick: move |_| props.show_filter_menu.set(false),
            }

            // Drawer
            div { class: "fixed top-0 right-0 h-full w-80 md:w-96 z-[70] glass-panel border-l border-indigo-500/20 shadow-2xl flex flex-col animate-slide-in-right",
                
                // Header
                div { class: "flex items-center justify-between p-5 border-b border-white/10",
                    div { class: "flex items-center gap-3",
                        svg { class: "w-5 h-5 text-indigo-400", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" }
                        }
                        h2 { class: "text-lg font-bold text-white tracking-tight", "Filters" }
                    }
                    button {
                        class: "p-2 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 rounded-xl transition-all",
                        onclick: move |_| props.show_filter_menu.set(false),
                        svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                        }
                    }
                }

                // Scrollable Content
                div { class: "flex-1 overflow-y-auto p-5 flex flex-col gap-6",
                    
                    // --- FILTER BY ACCOUNT ---
                    div { class: "flex flex-col gap-3",
                        label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider flex items-center gap-2", 
                            svg { class: "w-3 h-3 text-indigo-400", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" } }
                            "Account" 
                        }
                        select {
                            class: "w-full bg-slate-900/80 border border-white/10 rounded-xl px-4 py-3 text-sm text-white focus:outline-none focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/50 cursor-pointer shadow-inner transition-colors",
                            onchange: move |evt| props.selected_account_filter.set(evt.value()),
                            option { value: "All", "All Accounts" }
                            for account in props.collection.read().accounts.iter() {
                                option { value: "{account.name}", "{account.name}" }
                            }
                        }
                    }

                    // --- FILTER BY RARITY (icon chips) ---
                    div { class: "flex flex-col gap-3",
                        div { class: "flex items-center justify-between",
                            label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider", "Rarity" }
                            if !props.selected_rarities.read().is_empty() {
                                button { 
                                    class: "text-[10px] text-indigo-400 hover:text-indigo-300 font-bold transition-colors bg-indigo-500/10 px-2 py-1 rounded-md",
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
                                    
                                    let base = "flex items-center gap-1 px-3 py-2 rounded-xl border transition-all text-xs font-bold shadow-sm";
                                    let state_class = match (is_sar, is_active) {
                                        (true, true) => "sar-chip-active scale-105",
                                        (true, false) => "sar-chip hover:scale-105",
                                        (false, true) => "bg-gradient-to-br from-indigo-500 to-purple-600 border-indigo-400 text-white shadow-[0_4px_15px_rgba(99,102,241,0.4)] scale-105",
                                        (false, false) => "bg-slate-900/60 border-white/10 text-slate-400 hover:border-white/30 hover:bg-slate-800/80 hover:text-slate-200",
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
                    div { class: "flex flex-col gap-3 pt-4 border-t border-white/5",
                        div { class: "flex items-center justify-between",
                            label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider", "Type" }
                            if !props.selected_types.read().is_empty() {
                                button { 
                                    class: "text-[10px] text-indigo-400 hover:text-indigo-300 font-bold transition-colors bg-indigo-500/10 px-2 py-1 rounded-md",
                                    onclick: move |_| props.selected_types.set(Vec::new()),
                                    "Clear"
                                }
                            }
                        }
                        div { class: "flex gap-3",
                            for type_name in ALL_TYPES.iter() {
                                {
                                    let type_str = type_name.to_string();
                                    let type_for_click = type_str.clone();
                                    let is_active = props.selected_types.read().contains(&type_str);
                                    
                                    let icon = if *type_name == "Pokémon" { "⚡" } else { "🎴" };
                                    
                                    rsx! {
                                        button {
                                            class: "flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl border transition-all text-sm font-bold shadow-sm",
                                            class: if is_active { "bg-gradient-to-br from-indigo-500 to-purple-600 border-indigo-400 text-white shadow-[0_4px_15px_rgba(99,102,241,0.4)]" } else { "bg-slate-900/60 border-white/10 text-slate-400 hover:border-white/30 hover:bg-slate-800/80 hover:text-slate-200" },
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

                    // --- FILTER BY PACK ---
                    if let Some(packs) = props.pack_db.read().as_ref() {
                        div { class: "flex flex-col gap-3 pt-4 border-t border-white/5 pb-10",
                            div { class: "flex items-center justify-between",
                                label { class: "text-[10px] text-slate-400 uppercase font-black tracking-wider", "Expansions" }
                                if !props.selected_packs.read().is_empty() {
                                    button { 
                                        class: "text-[10px] text-indigo-400 hover:text-indigo-300 font-bold transition-colors bg-indigo-500/10 px-2 py-1 rounded-md",
                                        onclick: move |_| props.selected_packs.set(Vec::new()),
                                        "Clear"
                                    }
                                }
                            }
                            div { class: "grid grid-cols-2 gap-3",
                                for pack in packs.iter() {
                                    {
                                        let code_for_click = pack.code.clone();
                                        let is_active = props.selected_packs.read().contains(&code_for_click);
                                        
                                        let flib_code = match pack.code.as_str() {
                                            "P-A" => "PROMO-A",
                                            "P-B" => "PROMO-B",
                                            other => other,
                                        };
                                        let img_src = format!("https://cdn.jsdelivr.net/gh/flibustier/pokemon-tcg-pocket-database@main/dist/images/sets/LOGO_expansion_{}_en_US.webp", flib_code);
                                        
                                        let title = pack.name.get("en").cloned().unwrap_or_else(|| pack.code.clone());
                                        
                                        rsx! {
                                            button {
                                                class: "relative overflow-hidden rounded-xl border transition-all h-14 flex items-center justify-center p-2 shadow-sm",
                                                class: if is_active { "bg-indigo-500/20 border-indigo-400 shadow-[0_0_15px_rgba(99,102,241,0.3)] ring-2 ring-indigo-500/30" } else { "bg-slate-900/60 border-white/10 hover:border-white/30 hover:bg-slate-800/80 grayscale opacity-70 hover:grayscale-0 hover:opacity-100" },
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
                                                img { src: "{img_src}", alt: "{title}", class: "max-h-full max-w-full object-contain drop-shadow-md" }
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
