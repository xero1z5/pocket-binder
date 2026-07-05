use dioxus::prelude::*;
use crate::models::*;

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
    let mut show_add_form = use_signal(|| false);

    // Edit mode state: which account name is being edited
    let mut editing_account = use_signal(|| None::<String>);
    let mut edit_name = use_signal(|| String::new());
    let mut edit_id = use_signal(|| String::new());
    let mut edit_main = use_signal(|| false);

    rsx! {
        if *props.show_account_modal.read() {
            div { class: "fixed inset-0 flex items-center justify-center p-4 z-50",
                div { class: "absolute inset-0 bg-slate-950/60 backdrop-blur-sm animate-fade-in", onclick: move |_| props.show_account_modal.set(false) }
                div { class: "glass-panel rounded-2xl p-5 w-full max-w-sm flex flex-col gap-4 animate-fade-in-down",
                    
                    // Header
                    div { class: "flex justify-between items-center",
                        h2 { class: "text-lg font-bold text-white tracking-tight", 
                            if *show_add_form.read() { "New Account" } 
                            else if editing_account.read().is_some() { "Edit Account" } 
                            else { "Accounts" } 
                        }
                        
                        div { class: "flex items-center gap-2",
                            // "+" button only when listing
                            if !*show_add_form.read() && editing_account.read().is_none() {
                                button { 
                                    class: "p-1.5 text-indigo-400 hover:bg-indigo-500/20 rounded-lg transition-colors", 
                                    onclick: move |_| show_add_form.set(true),
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" } }
                                }
                            }
                            // Close modal
                            button { 
                                class: "p-1.5 text-slate-500 hover:bg-slate-800 rounded-lg transition-colors", 
                                onclick: move |_| { props.show_account_modal.set(false); show_add_form.set(false); editing_account.set(None); }, 
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                            }
                        }
                    }

                    if *show_add_form.read() {
                        // --- ADD NEW ACCOUNT FORM ---
                        div { class: "flex flex-col gap-3",
                            input { class: "bg-slate-950/50 border border-indigo-500/15 rounded-xl px-4 py-3 text-white text-sm focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors", placeholder: "Account Name (e.g., Main)", value: "{props.new_acc_name}", oninput: move |evt| props.new_acc_name.set(evt.value()) }
                            input { class: "bg-slate-950/50 border border-indigo-500/15 rounded-xl px-4 py-3 text-white text-sm focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors", placeholder: "Friend ID (Optional)", value: "{props.new_acc_id}", oninput: move |evt| props.new_acc_id.set(evt.value()) }
                            
                            div { class: "flex items-center gap-3 py-2",
                                input { r#type: "checkbox", class: "w-4 h-4 accent-indigo-500 rounded cursor-pointer", checked: *props.new_acc_is_main.read(), onchange: move |evt| props.new_acc_is_main.set(evt.value().parse().unwrap_or(false)) }
                                label { class: "text-sm text-slate-300", "Set as Main Account" }
                            }
                            
                            div { class: "flex gap-2 mt-2",
                                button { class: "flex-1 py-2.5 rounded-xl text-sm font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 transition-colors", onclick: move |_| show_add_form.set(false), "Cancel" }
                                button {
                                    class: "flex-1 py-2.5 rounded-xl text-sm font-bold text-slate-900 bg-gradient-to-r from-indigo-400 to-purple-400 hover:from-indigo-300 hover:to-purple-300 transition-all shadow-[0_0_15px_rgba(99,102,241,0.3)]",
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
                    } else if let Some(ref original_name) = *editing_account.read() {
                        // --- EDIT ACCOUNT FORM ---
                        {
                            let orig = original_name.clone();
                            rsx! {
                                div { class: "flex flex-col gap-3",
                                    label { class: "text-[10px] text-slate-500 uppercase font-black tracking-wider", "Account Name" }
                                    input { class: "bg-slate-950/50 border border-indigo-500/15 rounded-xl px-4 py-3 text-white text-sm focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors", value: "{edit_name}", oninput: move |evt| edit_name.set(evt.value()) }
                                    
                                    label { class: "text-[10px] text-slate-500 uppercase font-black tracking-wider", "Friend ID" }
                                    input { class: "bg-slate-950/50 border border-indigo-500/15 rounded-xl px-4 py-3 text-white text-sm focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors", placeholder: "Optional", value: "{edit_id}", oninput: move |evt| edit_id.set(evt.value()) }
                                    
                                    div { class: "flex items-center gap-3 py-2",
                                        input { r#type: "checkbox", class: "w-4 h-4 accent-indigo-500 rounded cursor-pointer", checked: *edit_main.read(), onchange: move |evt| edit_main.set(evt.value().parse().unwrap_or(false)) }
                                        label { class: "text-sm text-slate-300", "Main Account" }
                                    }
                                    
                                    div { class: "flex gap-2 mt-2",
                                        button { class: "flex-1 py-2.5 rounded-xl text-sm font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 transition-colors", onclick: move |_| editing_account.set(None), "Cancel" }
                                        button {
                                            class: "flex-1 py-2.5 rounded-xl text-sm font-bold text-slate-900 bg-gradient-to-r from-indigo-400 to-purple-400 hover:from-indigo-300 hover:to-purple-300 transition-all shadow-[0_0_15px_rgba(99,102,241,0.3)]",
                                            onclick: move |_| {
                                                let new_name = edit_name.read().clone();
                                                let new_id_val = edit_id.read().clone();
                                                let new_main_val = *edit_main.read();
                                                if !new_name.is_empty() {
                                                    props.collection.write().update_account(&orig, &new_name, &new_id_val, new_main_val);
                                                    editing_account.set(None);
                                                    props.toast_message.set(Some(format!("Account '{}' updated", new_name)));
                                                    let mut t = props.toast_message.clone(); spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                                }
                                            },
                                            "Save"
                                        }
                                    }
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
                                        let n1 = acc_name.clone(); let n2 = acc_name.clone(); let n3 = acc_name.clone();
                                        let edit_id_val = acc_id.clone();
                                        
                                        rsx! {
                                            div { class: "flex justify-between items-center bg-slate-800/50 p-3 rounded-xl border border-indigo-500/15 group",
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-sm font-semibold text-white", "{acc_name}" }
                                                    if !acc_id.is_empty() { span { class: "text-[10px] text-slate-500 font-mono", "ID: {acc_id}" } }
                                                }
                                                
                                                div { class: "flex items-center gap-2",
                                                    // Main Toggle
                                                    label { class: "relative inline-flex items-center cursor-pointer mr-1",
                                                        input { r#type: "checkbox", class: "sr-only peer", checked: is_main, onchange: move |evt| { props.collection.write().set_account_main_status(&n1, evt.value().parse().unwrap_or(false)); } }
                                                        div { class: "w-8 h-4.5 bg-slate-700 rounded-full peer peer-checked:after:translate-x-full after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-indigo-500" }
                                                    }
                                                    
                                                    // Edit Button
                                                    button {
                                                        class: "text-slate-600 hover:text-indigo-400 transition-colors p-1",
                                                        onclick: move |_| {
                                                            edit_name.set(n3.clone());
                                                            edit_id.set(edit_id_val.clone());
                                                            edit_main.set(is_main);
                                                            editing_account.set(Some(n3.clone()));
                                                        },
                                                        svg { class: "w-4 h-4", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", 
                                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" }
                                                        }
                                                    }

                                                    // Delete Button
                                                    button {
                                                        class: "text-slate-600 hover:text-rose-400 transition-colors p-1",
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
