use dioxus::prelude::*;
use std::collections::HashMap;
use gloo_storage::{LocalStorage, Storage};
use crate::models::*;
use crate::supabase::*;
use gloo_timers::future::sleep;

#[derive(PartialEq, Clone, Props)]
pub struct ToastProps {
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn Toast(props: ToastProps) -> Element {
    let mut visible = use_signal(|| false);

    // Auto-dismiss: show for 2s, fade out over 400ms, then clear the message.
    // Centralized here so every toast (regardless of where it was triggered) goes away.
    use_effect(move || {
        if props.toast_message.read().is_some() {
            let key = props.toast_message.read().clone();
            visible.set(true);
            let mut v = visible.clone();
            let mut tm = props.toast_message.clone();
            spawn(async move {
                sleep(std::time::Duration::from_secs(2)).await;
                v.set(false);
                sleep(std::time::Duration::from_millis(400)).await;
                // Only clear if the message hasn't changed in the meantime
                if *tm.read() == key {
                    tm.set(None);
                }
            });
        }
    });

    rsx! {
        if let Some(msg) = props.toast_message.read().clone() {
            div { class: format!(
                    "fixed bottom-6 right-6 text-white font-medium px-5 py-3.5 rounded-xl shadow-2xl border border-white/10 z-50 flex items-center gap-3 backdrop-blur-2xl transition-opacity duration-300 {}",
                    if *visible.read() { "opacity-100" } else { "opacity-0" }
                ),
                span { class: "flex h-3 w-3 relative",
                    span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-75" }
                    span { class: "relative inline-flex rounded-full h-3 w-3 bg-white/30" }
                }
                "{msg}"
            }
        }
    }
}
