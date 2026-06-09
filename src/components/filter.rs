use dioxus::prelude::*;
use std::collections::HashMap;
use gloo_storage::{LocalStorage, Storage};
use crate::models::*;
use crate::supabase::*;

#[derive(PartialEq, Clone, Props)]
pub struct FilterBarProps {
    pub search_query: Signal<String>,
    pub selected_account_filter: Signal<String>,
    pub collection: Signal<CardCollection>,
    pub show_add_modal: Signal<bool>,
    pub show_account_modal: Signal<bool>,
    pub sync_status: Signal<String>,
    pub auth_token: Signal<String>,
    pub show_filter_menu: Signal<bool>,
}

#[component]
pub fn FilterBar(mut props: FilterBarProps) -> Element {
    rsx! {
        div { class: "flex flex-col w-full mb-8 relative",
            
            // --- TOP ROW: SEARCH & ACTION BUTTONS ---
            div { class: "flex flex-col md:flex-row justify-between items-end gap-4 w-full",
                
                // LEFT: Search Input
                div { class: "w-full md:w-96",
                    div { class: "relative group",
                        div { class: "absolute inset-y-0 left-0 flex items-center pl-4 pointer-events-none text-gray-500 group-focus-within:text-orange-400 transition-colors",
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-5 h-5",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                            }
                        }
                        input {
                            class: "w-full bg-gray-800/80 border border-gray-700 rounded-2xl pl-11 pr-4 py-3.5 text-white focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500 shadow-inner transition-all placeholder-gray-500",
                            placeholder: "Search cards...",
                            value: "{props.search_query}",
                            oninput: move |evt| props.search_query.set(evt.value())
                        }
                    }
                }

                // RIGHT: Sync Status & 4 Square Buttons
                div { class: "flex flex-col items-end gap-2 w-full md:w-auto",
                    
                    div { class: "h-4 flex items-center pr-1",
                        if !props.sync_status.read().is_empty() {
                            span { class: "text-[10px] text-gray-400 font-mono tracking-widest", "{props.sync_status}" }
                        }
                    }

                    div { class: "flex items-center gap-3",
                        
                        button {
                            class: "group w-14 h-14 flex flex-col items-center justify-center gap-1 bg-gray-800 border border-gray-700 rounded-2xl hover:bg-gray-700 hover:border-gray-500 transition-all shadow-lg",
                            onclick: move |_| {
                                let current = *props.show_filter_menu.read();
                                props.show_filter_menu.set(!current);
                            },
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-6 h-6 text-gray-400 group-hover:text-white transition-colors",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" }
                            }
                        }

                        button {
                            class: "group w-14 h-14 flex flex-col items-center justify-center gap-1 bg-gray-800 border border-gray-700 rounded-2xl hover:bg-gray-700 hover:border-gray-500 transition-all shadow-lg",
                            onclick: move |_| props.show_account_modal.set(true),
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-6 h-6 text-gray-400 group-hover:text-white transition-colors",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0Zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0Z" }
                            }
                        }

                        button {
                            class: "group w-14 h-14 flex flex-col items-center justify-center gap-1 bg-orange-600/20 border border-orange-500/50 rounded-2xl hover:bg-orange-500 hover:border-orange-400 transition-all shadow-lg shadow-orange-900/20",
                            onclick: move |_| props.show_add_modal.set(true),
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-7 h-7 text-orange-400 group-hover:text-white transition-colors",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                            }
                        }

                        button {
                            class: "group w-14 h-14 flex flex-col items-center justify-center gap-1 bg-blue-600/20 border border-blue-500/50 rounded-2xl hover:bg-blue-500 hover:border-blue-400 transition-all shadow-lg shadow-blue-900/20",
                            onclick: move |_| {
                                props.sync_status.set("Syncing...".to_string());
                                let current_collection = props.collection.read().clone();
                                let token_to_use = props.auth_token.read().clone();
                                
                                spawn(async move {
                                    match save_to_supabase(current_collection, token_to_use).await {
                                        Ok(_) => props.sync_status.set("Last sync: Just now".to_string()),
                                        Err(_) => props.sync_status.set("Sync Failed!".to_string()),
                                    }
                                });
                            },
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-6 h-6 text-blue-400 group-hover:text-white transition-colors",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 16.5V9.75m0 0l3 3m-3-3l-3 3M6.75 19.5a4.5 4.5 0 01-1.41-8.775 5.25 5.25 0 0110.233-2.33 3 3 0 013.758 3.848A3.752 3.752 0 0118 19.5H6.75z" }
                            }
                        }
                    }
                }
            }

            // --- THE EXPANDABLE FILTERS TRAY ---
            if *props.show_filter_menu.read() {
                div { class: "w-full bg-gray-800/90 border border-gray-700 rounded-2xl p-4 mt-4 shadow-xl backdrop-blur-md animate-fade-in-down",
                    div { class: "flex flex-col md:flex-row gap-6",
                        
                        div { class: "flex flex-col gap-2",
                            label { class: "text-[10px] text-gray-400 uppercase font-black tracking-wider flex items-center gap-1", 
                                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-3 h-3", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" } }
                                "Filter by Account" 
                            }
                            select {
                                class: "bg-gray-900 border border-gray-700 rounded-xl px-4 py-2 text-sm text-white focus:outline-none focus:border-orange-500 cursor-pointer min-w-[200px] shadow-inner",
                                onchange: move |evt| props.selected_account_filter.set(evt.value()),
                                option { value: "All", "All Accounts" }
                                for account in props.collection.read().accounts.iter() {
                                    option { value: "{account.name}", "{account.name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
