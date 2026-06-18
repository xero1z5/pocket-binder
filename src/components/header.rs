use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "flex justify-center items-center mb-6 pt-2 w-full px-2 md:px-0",
            div { class: "bg-slate-800/60 border border-indigo-500/20 px-6 py-2.5 rounded-2xl shadow-lg w-full md:w-auto text-center backdrop-blur-md relative overflow-hidden",
                // Glass reflection effect
                div { class: "absolute inset-0 bg-gradient-to-tr from-transparent via-white/5 to-transparent opacity-50 transform -skew-x-12 pointer-events-none" }
                h1 { class: "animate-gradient-x text-lg md:text-xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 via-purple-300 to-indigo-400", 
                    "🧬 POCKET BINDER" 
                }
            }
        }
    }
}
