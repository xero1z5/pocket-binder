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
        // Use flex-col on mobile, flex-row on desktop (md:)
        header { class: "flex flex-col md:flex-row justify-between items-center gap-4 mb-6 pt-2 w-full",
            div { class: "bg-gray-800/80 border border-gray-700 px-6 py-2.5 rounded-2xl shadow-lg w-full md:w-auto text-center",
                h1 { class: "text-lg md:text-xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-red-500 to-orange-400", 
                    "🧬 POCKET BINDER" 
                }
            }
            
            div { class: "flex items-center justify-center gap-4 w-full md:w-auto",
                if props.auth_token.read().is_empty() {
                    button {
                        class: "bg-gray-800 border border-gray-600 hover:border-gray-500 text-gray-300 font-bold py-2 px-6 rounded-xl transition-all shadow-md w-full md:w-auto",
                        onclick: move |_| props.show_login_modal.set(true),
                        "Login"
                    }
                } else {
                    div { class: "flex items-center justify-between md:justify-start gap-3 bg-gray-800/50 py-1.5 px-4 rounded-xl border border-gray-700 w-full md:w-auto",
                        span { class: "text-xs md:text-sm text-green-400 font-mono font-bold truncate max-w-[150px] md:max-w-none", "{props.user_email}" }
                        button {
                            class: "text-xs text-gray-500 hover:text-red-400 transition-colors ml-2 font-bold px-2 py-1 bg-gray-800 rounded-md",
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
