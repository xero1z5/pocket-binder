use dioxus::prelude::*;
use crate::models::*;
use gloo_storage::{LocalStorage, Storage};

#[derive(PartialEq, Clone, Props)]
pub struct AccountModalProps {
    pub show_account_modal: Signal<bool>,
    pub new_acc_name: Signal<String>,
    pub new_acc_id: Signal<String>,
    pub new_acc_is_main: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub toast_message: Signal<Option<String>>,
    pub user_email: Signal<String>,
    pub auth_token: Signal<String>,
}

#[component]
pub fn AccountModal(mut props: AccountModalProps) -> Element {
    if !*props.show_account_modal.read() {
        return rsx! {};
    }

    let mut show_add_form = use_signal(|| false);

    // Edit mode state: which account name is being edited
    let mut editing_account = use_signal(|| None::<String>);
    let mut edit_name = use_signal(|| String::new());
    let mut edit_id = use_signal(|| String::new());
    let mut edit_main = use_signal(|| false);

    rsx! {
        if *props.show_account_modal.read() {
            div { class: "fixed inset-0 flex items-center justify-center p-4 z-50",
                div { class: "absolute inset-0 scrim animate-fade-in", onclick: move |_| props.show_account_modal.set(false) }
                div { class: "acrylic-strong relative rounded-[1.75rem] p-5 w-full max-w-sm flex flex-col gap-4 animate-fade-in-down",
                    
                    // Header
                    div { class: "flex justify-between items-center",
                        h2 { class: "text-lg font-semibold text-white tracking-tight pl-1", 
                            if *show_add_form.read() { "New Account" } 
                            else if editing_account.read().is_some() { "Edit Account" } 
                            else { "Accounts" } 
                        }
                        
                        div { class: "flex items-center gap-2",
                            // "+" button only when listing
                            if !*show_add_form.read() && editing_account.read().is_none() {
                                button { 
                                    class: "icon-btn is-bare",
                                    title: "Add account",
                                    onclick: move |_| show_add_form.set(true),
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" } }
                                }
                            }
                            // Close modal
                            button { 
                                class: "icon-btn is-bare",
                                title: "Close",
                                onclick: move |_| { props.show_account_modal.set(false); show_add_form.set(false); editing_account.set(None); }, 
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                            }
                        }
                    }

                    if !*show_add_form.read() && editing_account.read().is_none() {
                        div { class: "row justify-between py-3",
                            div { class: "flex items-center gap-3 min-w-0",
                                div { class: "w-9 h-9 rounded-full bg-accent/15 text-accent flex items-center justify-center flex-shrink-0",
                                    svg { class: "w-4 h-4", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" } }
                                }
                                div { class: "flex flex-col min-w-0",
                                    span { class: "text-xs text-slate-400", "Signed in as" }
                                    span { class: "text-sm font-medium text-white truncate", "{props.user_email}" }
                                }
                            }
                            button {
                                class: "btn btn-danger min-h-0 h-9 px-3 text-xs flex-shrink-0",
                                onclick: move |_| {
                                    let _ = LocalStorage::delete("supabase_token");
                                    let _ = LocalStorage::delete("user_email");
                                    props.auth_token.set(String::new());
                                    props.user_email.set(String::new());
                                    props.collection.set(CardCollection { accounts: Vec::new(), inventory: Vec::new(), wishlist: Vec::new(), tradable: Vec::new() });
                                    props.show_account_modal.set(false);
                                },
                                "Sign out"
                            }
                        }
                    }

                    if *show_add_form.read() {
                        // --- ADD NEW ACCOUNT FORM ---
                        div { class: "flex flex-col gap-3",
                            input { class: "field", placeholder: "Account name, like Main", value: "{props.new_acc_name}", oninput: move |evt| props.new_acc_name.set(evt.value()) }
                            input { class: "field", placeholder: "Friend ID (optional)", value: "{props.new_acc_id}", oninput: move |evt| props.new_acc_id.set(evt.value()) }
                            
                            label { class: "flex items-center gap-3 py-2 cursor-pointer w-fit",
                                input { r#type: "checkbox", class: "w-4 h-4 accent-[#ffcb05] cursor-pointer", checked: *props.new_acc_is_main.read(), onchange: move |evt| props.new_acc_is_main.set(evt.value().parse().unwrap_or(false)) }
                                span { class: "text-sm text-slate-300", "Main account" }
                            }
                            
                            div { class: "flex gap-2 mt-2",
                                button { class: "btn btn-glass flex-1", onclick: move |_| show_add_form.set(false), "Cancel" }
                                button {
                                    class: "btn btn-primary flex-1",
                                    onclick: move |_| {
                                        let name = props.new_acc_name.read().clone();
                                        if !name.is_empty() {
                                            props.collection.write().accounts.push(Account { name: name.clone(), id: props.new_acc_id.read().clone(), main: *props.new_acc_is_main.read() });
                                            props.new_acc_name.set(String::new()); props.new_acc_id.set(String::new()); props.new_acc_is_main.set(true);
                                            show_add_form.set(false);
                                            props.toast_message.set(Some(format!("Account '{}' created", name)));
                                            
                                        }
                                    },
                                    "Create account"
                                }
                            }
                        }
                    } else if let Some(ref original_name) = *editing_account.read() {
                        // --- EDIT ACCOUNT FORM ---
                        {
                            let orig = original_name.clone();
                            rsx! {
                                div { class: "flex flex-col gap-3",
                                    label { class: "field-label", "Account name" }
                                    input { class: "field", value: "{edit_name}", oninput: move |evt| edit_name.set(evt.value()) }
                                    
                                    label { class: "field-label", "Friend ID" }
                                    input { class: "field", placeholder: "Optional", value: "{edit_id}", oninput: move |evt| edit_id.set(evt.value()) }
                                    
                                    label { class: "flex items-center gap-3 py-2 cursor-pointer w-fit",
                                        input { r#type: "checkbox", class: "w-4 h-4 accent-[#ffcb05] cursor-pointer", checked: *edit_main.read(), onchange: move |evt| edit_main.set(evt.value().parse().unwrap_or(false)) }
                                        span { class: "text-sm text-slate-300", "Main account" }
                                    }
                                    
                                    div { class: "flex gap-2 mt-2",
                                        button { class: "btn btn-glass flex-1", onclick: move |_| editing_account.set(None), "Cancel" }
                                        button {
                                            class: "btn btn-primary flex-1",
                                            onclick: move |_| {
                                                let new_name = edit_name.read().clone();
                                                let new_id_val = edit_id.read().clone();
                                                let new_main_val = *edit_main.read();
                                                if !new_name.is_empty() {
                                                    props.collection.write().update_account(&orig, &new_name, &new_id_val, new_main_val);
                                                    editing_account.set(None);
                                                    props.toast_message.set(Some(format!("Account '{}' updated", new_name)));
                                                    
                                                }
                                            },
                                            "Save changes"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // --- ACCOUNT LIST ---
                        div { class: "flex flex-col gap-2 max-h-64 overflow-y-auto pr-1",
                            if props.collection.read().accounts.is_empty() {
                                div { class: "text-center text-sm text-slate-400 py-6", "No accounts yet. Use + to add one." }
                            } else {
                                for acc in props.collection.read().accounts.iter() {
                                    {
                                        let acc_name = acc.name.clone(); let acc_id = acc.id.clone(); let is_main = acc.main;
                                        let n1 = acc_name.clone(); let n2 = acc_name.clone(); let n3 = acc_name.clone();
                                        let edit_id_val = acc_id.clone();
                                        
                                        rsx! {
                                            div { class: "row justify-between py-2.5 group",
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-sm font-medium text-white", "{acc_name}" }
                                                    if !acc_id.is_empty() { span { class: "text-xs text-slate-400", "Friend ID {acc_id}" } }
                                                }
                                                
                                                div { class: "flex items-center gap-2",
                                                    // Main Toggle
                                                    label { class: "relative inline-flex items-center cursor-pointer mr-1", title: "Main account",
                                                        input { r#type: "checkbox", class: "sr-only peer", checked: is_main, onchange: move |evt| { props.collection.write().set_account_main_status(&n1, evt.value().parse().unwrap_or(false)); } }
                                                        div { class: "w-8 h-4.5 bg-slate-700 rounded-full peer peer-checked:after:translate-x-full after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent" }
                                                    }
                                                    
                                                    // Edit Button
                                                    button {
                                                        class: "icon-btn is-bare w-9 h-9", title: "Edit account",
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
                                                        class: "icon-btn is-bare is-danger w-9 h-9", title: "Delete account",
                                                        onclick: move |_| { props.collection.write().remove_account(&n2); props.toast_message.set(Some(format!("Account deleted")));  },
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
