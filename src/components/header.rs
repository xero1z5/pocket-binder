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
            div { class: "bg-slate-800/60 border border-indigo-500/20 px-6 py-2.5 rounded-2xl shadow-lg w-full md:w-auto text-center backdrop-blur-md relative overflow-hidden",
                // Glass reflection effect
                div { class: "absolute inset-0 bg-gradient-to-tr from-transparent via-white/5 to-transparent opacity-50 transform -skew-x-12 pointer-events-none" }
                h1 { class: "animate-gradient-x text-lg md:text-xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 via-purple-300 to-indigo-400", 
                    "🧬 POCKET BINDER" 
                }
            }
            
            div { class: "flex items-center justify-center gap-4 w-full md:w-auto",
                if props.auth_token.read().is_empty() {
                    button {
                        class: "bg-slate-800/60 border border-indigo-500/30 hover:border-indigo-400 text-slate-300 hover:text-indigo-400 font-bold py-2 px-6 rounded-xl transition-all shadow-md w-full md:w-auto backdrop-blur-sm",
                        onclick: move |_| props.show_login_modal.set(true),
                        "Login"
                    }
                } else {
                    div { class: "flex items-center justify-between md:justify-start gap-3 bg-slate-800/60 py-1.5 px-4 rounded-xl border border-indigo-500/20 w-full md:w-auto backdrop-blur-sm",
                        span { class: "text-xs md:text-sm text-indigo-400 font-mono font-bold truncate max-w-[150px] md:max-w-none", "{props.user_email}" }
                        button {
                            class: "text-xs text-slate-500 hover:text-rose-400 transition-colors ml-2 font-bold px-2 py-1 bg-slate-800 rounded-md",
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
