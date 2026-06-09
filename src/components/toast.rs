use dioxus::prelude::*;
use std::collections::HashMap;
use gloo_storage::{LocalStorage, Storage};
use crate::models::*;
use crate::supabase::*;

#[derive(PartialEq, Clone, Props)]
pub struct ToastProps {
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn Toast(props: ToastProps) -> Element {
    rsx! {
        if let Some(msg) = props.toast_message.read().clone() {
            div { class: "fixed bottom-6 right-6 bg-gray-800 text-white font-bold px-6 py-4 rounded-xl shadow-2xl border border-gray-700 z-50 flex items-center gap-3",
                span { class: "flex h-3 w-3 relative",
                    span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75" }
                    span { class: "relative inline-flex rounded-full h-3 w-3 bg-green-500" }
                }
                "{msg}"
            }
        }
    }
}
