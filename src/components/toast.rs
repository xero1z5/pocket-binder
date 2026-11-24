use dioxus::prelude::*;
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
                    "acrylic-strong fixed top-20 inset-x-0 mx-auto w-fit max-w-[90vw] text-sm text-white font-medium pl-4 pr-5 py-3 rounded-2xl z-[90] flex items-center gap-3 transition-opacity duration-300 {}",
                    if *visible.read() { "opacity-100" } else { "opacity-0" }
                ),
                span { class: "w-2 h-2 rounded-full bg-accent flex-shrink-0" }
                "{msg}"
            }
        }
    }
}
