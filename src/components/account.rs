use dioxus::prelude::*;
use crate::models::*;

#[derive(PartialEq, Clone, Props)]
pub struct AccountButtonProps {
    pub show_account_modal: Signal<bool>,
}

#[component]
pub fn AccountButton(mut props: AccountButtonProps) -> Element {
    rsx! {
        button {
            class: "group w-11 h-11 md:w-14 md:h-14 flex flex-col items-center justify-center bg-gray-800 border border-gray-700 rounded-xl md:rounded-2xl hover:bg-gray-700 hover:border-gray-500 transition-all shadow-lg",
            onclick: move |_| props.show_account_modal.set(true),
            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-5 h-5 md:w-6 md:h-6 text-gray-400 group-hover:text-white transition-colors",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0Zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0Z" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct AccountModalProps {
    pub show_account_modal: Signal<bool>,
    pub new_acc_name: Signal<String>,
    pub new_acc_id: Signal<String>,
    pub new_acc_is_main: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn AccountModal(mut props: AccountModalProps) -> Element {
    // (Paste the exact AccountModal implementation code here from the previous step. 
    // It is identical, it just lives in this file now.)
    rsx! { div {} } // Placeholder to keep snippet short, replace with full Modal rsx!
}
