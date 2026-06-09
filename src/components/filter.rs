use dioxus::prelude::*;
use crate::models::*;

#[derive(PartialEq, Clone, Props)]
pub struct SearchInputProps {
    pub search_query: Signal<String>,
}

#[component]
pub fn SearchInput(mut props: SearchInputProps) -> Element {
    rsx! {
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
            class: "group w-11 h-11 md:w-14 md:h-14 flex flex-col items-center justify-center bg-gray-800 border border-gray-700 rounded-xl md:rounded-2xl hover:bg-gray-700 hover:border-gray-500 transition-all shadow-lg",
            onclick: move |_| {
                let current = *props.show_filter_menu.read();
                props.show_filter_menu.set(!current);
            },
            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-5 h-5 md:w-6 md:h-6 text-gray-400 group-hover:text-white transition-colors",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct FilterTrayProps {
    pub show_filter_menu: Signal<bool>,
    pub selected_account_filter: Signal<String>,
    pub collection: Signal<CardCollection>,
}

#[component]
pub fn FilterTray(mut props: FilterTrayProps) -> Element {
    rsx! {
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
