use dioxus::prelude::*;
use crate::models::*;

// Keep AccountButtonProps and AccountButton as they were, but update colors to slate
#[derive(PartialEq, Clone, Props)]
pub struct AccountButtonProps { pub show_account_modal: Signal<bool> }

#[component]
pub fn AccountButton(mut props: AccountButtonProps) -> Element {
    rsx! {
        button {
            class: "group w-11 h-11 md:w-14 md:h-14 flex items-center justify-center bg-slate-800/80 border border-slate-700/80 rounded-xl hover:bg-slate-700 hover:border-teal-500/50 transition-all shadow-lg backdrop-blur-sm",
            onclick: move |_| props.show_account_modal.set(true),
            svg { class: "w-5 h-5 text-slate-400 group-hover:text-teal-400 transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
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
    // NEW: Toggle between viewing accounts and adding a new one
    let mut show_add_form = use_signal(|| false);

    rsx! {
        if *props.show_account_modal.read() {
            div { class: "fixed inset-0 bg-slate-950/80 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                div { class: "bg-slate-900 border border-slate-800 rounded-2xl p-5 w-full max-w-sm shadow-2xl flex flex-col gap-4 animate-fade-in-down",
                    
                    // Header
                    div { class: "flex justify-between items-center",
                        h2 { class: "text-lg font-bold text-white tracking-tight", if *show_add_form.read() { "New Account" } else { "Accounts" } }
                        
                        div { class: "flex items-center gap-2",
                            // The "+" Button (Only show if not already adding)
                            if !*show_add_form.read() {
                                button { 
                                    class: "p-1.5 text-teal-400 hover:bg-teal-500/20 rounded-lg transition-colors", 
                                    onclick: move |_| show_add_form.set(true),
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" } }
                                }
                            }
                            // Close Modal Button
                            button { 
                                class: "p-1.5 text-slate-500 hover:bg-slate-800 rounded-lg transition-colors", 
                                onclick: move |_| { props.show_account_modal.set(false); show_add_form.set(false); }, 
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                            }
                        }
                    }

                    if *show_add_form.read() {
                        // --- ADD NEW ACCOUNT FORM ---
                        div { class: "flex flex-col gap-3",
                            input { class: "bg-slate-950/50 border border-slate-800 rounded-xl px-4 py-3 text-white text-sm focus:border-teal-500 outline-none transition-colors", placeholder: "Account Name (e.g., Main)", value: "{props.new_acc_name}", oninput: move |evt| props.new_acc_name.set(evt.value()) }
                            input { class: "bg-slate-950/50 border border-slate-800 rounded-xl px-4 py-3 text-white text-sm focus:border-teal-500 outline-none transition-colors", placeholder: "Friend ID (Optional)", value: "{props.new_acc_id}", oninput: move |evt| props.new_acc_id.set(evt.value()) }
                            
                            div { class: "flex items-center gap-3 py-2",
                                input { r#type: "checkbox", class: "w-4 h-4 accent-teal-500 rounded cursor-pointer", checked: *props.new_acc_is_main.read(), onchange: move |evt| props.new_acc_is_main.set(evt.value().parse().unwrap_or(false)) }
                                label { class: "text-sm text-slate-300", "Set as Main Account" }
                            }
                            
                            div { class: "flex gap-2 mt-2",
                                button { class: "flex-1 py-2.5 rounded-xl text-sm font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 transition-colors", onclick: move |_| show_add_form.set(false), "Cancel" }
                                button {
                                    class: "flex-1 py-2.5 rounded-xl text-sm font-bold text-slate-900 bg-teal-400 hover:bg-teal-300 transition-colors shadow-[0_0_15px_rgba(45,212,191,0.3)]",
                                    onclick: move |_| {
                                        let name = props.new_acc_name.read().clone();
                                        if !name.is_empty() {
                                            props.collection.write().accounts.push(Account { name: name.clone(), id: props.new_acc_id.read().clone(), main: *props.new_acc_is_main.read() });
                                            props.new_acc_name.set(String::new()); props.new_acc_id.set(String::new()); props.new_acc_is_main.set(true);
                                            show_add_form.set(false);
                                            props.toast_message.set(Some(format!("Account '{}' created", name)));
                                            let mut t = props.toast_message.clone(); spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                        }
                                    },
                                    "Create"
                                }
                            }
                        }
                    } else {
                        // --- ACCOUNT LIST ---
                        div { class: "flex flex-col gap-2 max-h-64 overflow-y-auto pr-1",
                            if props.collection.read().accounts.is_empty() {
                                div { class: "text-center text-sm text-slate-500 py-6", "No accounts added yet." }
                            } else {
                                for acc in props.collection.read().accounts.iter() {
                                    {
                                        let acc_name = acc.name.clone(); let acc_id = acc.id.clone(); let is_main = acc.main;
                                        let n1 = acc_name.clone(); let n2 = acc_name.clone();
                                        
                                        rsx! {
                                            div { class: "flex justify-between items-center bg-slate-800/50 p-3 rounded-xl border border-slate-700/50 group",
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-sm font-semibold text-white", "{acc_name}" }
                                                    if !acc_id.is_empty() { span { class: "text-[10px] text-slate-500 font-mono", "ID: {acc_id}" } }
                                                }
                                                
                                                div { class: "flex items-center gap-2",
                                                    // Main Toggle
                                                    label { class: "relative inline-flex items-center cursor-pointer mr-2",
                                                        input { r#type: "checkbox", class: "sr-only peer", checked: is_main, onchange: move |evt| { props.collection.write().set_account_main_status(&n1, evt.value().parse().unwrap_or(false)); } }
                                                        div { class: "w-8 h-4.5 bg-slate-700 rounded-full peer peer-checked:after:translate-x-full after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-teal-500" }
                                                    }
                                                    
                                                    // Delete Button
                                                    button {
                                                        class: "text-slate-600 hover:text-red-400 transition-colors p-1",
                                                        onclick: move |_| { props.collection.write().remove_account(&n2); props.toast_message.set(Some(format!("Account deleted"))); let mut t = props.toast_message.clone(); spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); }); },
                                                        svg { class: "w-4 h-4", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
