use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::models::*;
use crate::supabase::*;

#[derive(PartialEq, Clone, Props)]
pub struct HeaderProps {
    pub auth_token: Signal<String>,
    pub user_email: Signal<String>,
    pub show_login_modal: Signal<bool>,
}

#[component]
pub fn Header(mut props: HeaderProps) -> Element {
    rsx! {
        header { class: "border-b border-gray-800 pb-4 mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4",
            h1 { class: "text-2xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-red-500 to-orange-400", 
                "🧬 POCKET TCG BINDER" 
            }
            
            // Login Button / User Profile
            div { class: "flex items-center gap-4",
                if props.auth_token.read().is_empty() {
                    button {
                        class: "bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-600 font-bold py-2 px-4 rounded-lg transition-colors flex items-center gap-2 shadow-lg",
                        onclick: move |_| props.show_login_modal.set(true),
                        "🔑 Cloud Login"
                    }
                } else {
                    div { class: "flex items-center gap-3 bg-gray-800/50 py-1.5 px-4 rounded-full border border-gray-700",
                        span { class: "text-sm text-green-400 font-mono font-bold", "👤 {props.user_email}" }
                        button {
                            class: "text-xs text-gray-500 hover:text-red-400 hover:underline transition-colors ml-2",
                            onclick: move |_| {
                                props.auth_token.set(String::new());
                                props.user_email.set(String::new());
                                LocalStorage::delete("supabase_token");
                                LocalStorage::delete("user_email");
                            },
                            "Logout"
                        }
                    }
                }
            }
        }
    }
}


#[derive(PartialEq, Clone, Props)]
pub struct FilterBarProps {
    pub search_query: Signal<String>,
    pub selected_account_filter: Signal<String>,
    pub collection: Signal<CardCollection>,
    pub show_add_modal: Signal<bool>,
    pub show_account_modal: Signal<bool>,
    pub sync_status: Signal<String>,
    pub auth_token: Signal<String>,
}

#[component]
pub fn FilterBar(mut props: FilterBarProps) -> Element {
    rsx! {
        div { class: "flex flex-col md:flex-row gap-4 mb-6",
            // Search Input
            input {
                class: "flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500",
                placeholder: "Search by card name...",
                value: "{props.search_query}",
                oninput: move |evt| props.search_query.set(evt.value())
            }

            // Account Filter Dropdown
            select {
                class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500 cursor-pointer",
                onchange: move |evt| props.selected_account_filter.set(evt.value()),
                option { value: "All", "All Accounts" }
                for account in props.collection.read().accounts.iter() {
                    option { value: "{account.name}", "{account.name}" }
                }
            }

            // New Card Button
            button {
                class: "bg-orange-600 hover:bg-orange-500 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95",
                onclick: move |_| props.show_add_modal.set(true),
                "➕ Add Card"
            }

            // Manage account Button
            button {
                class: "bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95",
                onclick: move |_| props.show_account_modal.set(true),
                "👥 Accounts"
            }

            // Save to Supabase Button
            button {
                class: "bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 px-6 rounded-lg shadow-lg transition-transform active:scale-95 flex items-center gap-2",
                onclick: move |_| {
                    props.sync_status.set("⏳ Saving to Cloud...".to_string());
                    
                    let current_collection = props.collection.read().clone();
                    let token_to_use = props.auth_token.read().clone();
                    
                    spawn(async move {
                        match save_to_supabase(current_collection, token_to_use).await {
                            Ok(_) => props.sync_status.set("✅ Cloud Synced!".to_string()),
                            Err(e) => props.sync_status.set(format!("❌ {}", e)),
                        }
                    });
                },
                "☁️ Sync to Cloud"
            }

            // status indicator
            if !props.sync_status.read().is_empty() {
                span { class: "text-sm font-mono flex items-center", "{props.sync_status}" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ToastProps {
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn Toast(props: ToastProps) -> Element {
    rsx! {
        if let Some(msg) = props.toast_message.read().clone() {
            div { class: "fixed bottom-6 right-6 bg-green-600 text-white font-bold px-6 py-3 rounded-lg shadow-2xl border border-green-400 z-50 animate-bounce",
                "{msg}"
            }
        }
    }
}
