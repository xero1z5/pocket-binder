use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "flex justify-center items-center mb-6 pt-2 w-full px-2 md:px-0",
            div { class: "bg-white/5 border border-white/10 px-6 py-2.5 rounded-2xl w-full md:w-auto text-center backdrop-blur-md relative overflow-hidden",
                h1 { class: "text-lg md:text-xl font-bold tracking-tight text-white", 
                    "🧬 POCKET BINDER" 
                }
            }
        }
    }
}
