use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashMap;
use crate::models::*;
use crate::supabase::*;

#[derive(PartialEq, Clone, Props)]
pub struct AddCardModalProps {
    pub show_add_modal: Signal<bool>,
    pub add_search_query: Signal<String>,
    pub add_target_account: Signal<String>,
    pub collection: Signal<CardCollection>,
    pub image_db: Resource<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn AddCardModal(mut props: AddCardModalProps) -> Element {
    rsx! {
        if *props.show_add_modal.read() {
            div { class: "fixed inset-0 bg-black/90 flex flex-col items-center p-4 md:p-10 z-50",
                div { class: "w-full max-w-6xl bg-gray-900 border border-gray-700 rounded-t-xl p-4 flex flex-col md:flex-row gap-4 justify-between items-center shadow-2xl",
                    h2 { class: "text-2xl font-bold text-orange-400", "Add from Database" }
                    div { class: "flex w-full md:w-auto gap-2",
                        input { class: "flex-1 md:w-64 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500", placeholder: "Search official API...", value: "{props.add_search_query}", oninput: move |evt| props.add_search_query.set(evt.value()) }
                        select { class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 cursor-pointer", onchange: move |evt| props.add_target_account.set(evt.value()),
                            for account in props.collection.read().accounts.iter() {
                                option { value: "{account.name}", "{account.name}" }
                            }
                        }
                        button { class: "bg-red-900/80 hover:bg-red-800 text-red-200 px-4 py-2 rounded-lg font-bold", onclick: move |_| props.show_add_modal.set(false), "Close" }
                    }
                }

                div { class: "w-full max-w-6xl flex-1 bg-gray-800/50 border-x border-b border-gray-700 rounded-b-xl p-4 overflow-y-auto",
                    div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-3",
                        {
                            let mut display_cards = Vec::new();
                            if let Some(Some(db)) = &*props.image_db.read() {
                                let search_term = props.add_search_query.read().to_lowercase();
                                let mut count = 0;
                                for card in db.values() {
                                    if search_term.is_empty() || card.name.to_lowercase().contains(&search_term) {
                                        display_cards.push(card.clone());
                                        count += 1;
                                        if count >= 32 { break; }
                                    }
                                }
                            }

                            if display_cards.is_empty() {
                                rsx! { p { class: "col-span-full text-center text-gray-500 mt-10 animate-pulse", "Loading API Database or No Results Found..." } }
                            } else {
                                rsx! {
                                    for official_card in display_cards {
                                        div { class: "relative group cursor-pointer transition-transform hover:scale-105 hover:z-10",
                                            onclick: move |_| {
                                                let card_name = official_card.name.clone();
                                                let new_card = Card { id: official_card.id.clone(), name: official_card.name.clone(), rarity: official_card.rarity.clone(), card_type: official_card.card_type.clone(), pack: official_card.pack.clone() };
                                                props.collection.write().add_card(new_card, &*props.add_target_account.read(), 1);
                                                
                                                props.toast_message.set(Some(format!("✅ Added {}!", card_name)));
                                                let mut toast = props.toast_message.clone();
                                                spawn(async move {
                                                    gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                                    toast.set(None);
                                                });
                                            },
                                            img { src: "{official_card.image}", loading: "lazy", class: "w-full rounded border border-gray-600 shadow-md" }
                                            div { class: "absolute inset-0 bg-blue-500/50 opacity-0 group-hover:opacity-100 rounded flex items-center justify-center backdrop-blur-sm transition-opacity",
                                                span { class: "bg-gray-900 text-white text-xs font-bold px-2 py-1 rounded-full", "+ Add" }
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
    rsx! {
        if *props.show_account_modal.read() {
            div { class: "fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                div { class: "bg-gray-900 border border-gray-700 rounded-xl p-6 w-full max-w-md shadow-2xl flex flex-col gap-4",
                    
                    // Header
                    div { class: "flex justify-between items-center mb-2",
                        h2 { class: "text-2xl font-black text-white", "Manage Accounts" }
                        button { class: "text-gray-500 hover:text-white transition-colors", onclick: move |_| props.show_account_modal.set(false), "✕" }
                    }

                    // --- CURRENT ACCOUNTS LIST ---
                    div { class: "flex flex-col gap-2 max-h-48 overflow-y-auto pr-2",
                        if props.collection.read().accounts.is_empty() {
                            div { class: "text-center text-sm text-gray-500 italic py-4", "No accounts added yet." }
                        } else {
                            for acc in props.collection.read().accounts.iter() {
                                {
                                    // Extract & clone variables for our event handlers
                                    let acc_name = acc.name.clone();
                                    let acc_id = acc.id.clone();
                                    let is_main = acc.main;
                                    
                                    // Clone specifically for the closures
                                    let name_for_toggle = acc_name.clone();
                                    let name_for_delete = acc_name.clone();
                                    
                                    // FIX: Calculate colors outside of the RSX macro!
                                    let temp_color = if is_main { "text-gray-600" } else { "text-blue-400" };
                                    let main_color = if is_main { "text-green-400" } else { "text-gray-600" };
                                    
                                    rsx! {
                                        div { class: "flex justify-between items-center bg-gray-800 p-2.5 rounded-lg border border-gray-700",
                                            div { class: "flex flex-col gap-1",
                                                span { class: "text-sm font-bold text-white leading-none", "{acc_name}" }
                                                span { class: "text-[10px] text-gray-400 font-mono leading-none", "ID: {acc_id}" }
                                            }
                                            
                                            div { class: "flex items-center gap-3",
                                                
                                                // --- THE INTERACTIVE TOGGLE ---
                                                div { class: "flex items-center gap-1.5 bg-gray-900/50 px-2 py-1 rounded-md border border-gray-700",
                                                    // Use the calculated variable here cleanly
                                                    span { class: "text-[9px] font-black tracking-wide {temp_color}", "TEMP" }
                                                    
                                                    // Tailwind CSS Toggle Switch
                                                    label { class: "relative inline-flex items-center cursor-pointer",
                                                        input { 
                                                            r#type: "checkbox", 
                                                            class: "sr-only peer",
                                                            checked: is_main,
                                                            onchange: move |evt| {
                                                                let new_status = evt.value().parse().unwrap_or(false);
                                                                props.collection.write().set_account_main_status(&name_for_toggle, new_status);
                                                            }
                                                        }
                                                        div { class: "w-7 h-4 bg-gray-600 rounded-full peer peer-checked:after:translate-x-full after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-green-500" }
                                                    }
                                                    
                                                    // And use the calculated variable here cleanly
                                                    span { class: "text-[9px] font-black tracking-wide {main_color}", "MAIN" }
                                                }

                                                // Delete Account Button
                                                button {
                                                    class: "text-red-400 hover:text-white hover:bg-red-500 rounded p-1 transition-colors flex items-center justify-center w-6 h-6",
                                                    title: "Delete Account",
                                                    onclick: move |_| {
                                                        props.collection.write().remove_account(&name_for_delete);
                                                        
                                                        props.toast_message.set(Some(format!("🗑️ Account '{}' deleted!", name_for_delete)));
                                                        let mut toast = props.toast_message.clone();
                                                        spawn(async move {
                                                            gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                                            toast.set(None);
                                                        });
                                                    },
                                                    "✕"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Divider
                    hr { class: "border-gray-800 my-2" }

                    // --- ADD NEW ACCOUNT FORM ---
                    h3 { class: "text-sm font-bold text-orange-400 uppercase tracking-wide", "Create New" }
                    
                    div { class: "flex flex-col gap-1.5",
                        input { class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white text-sm focus:border-blue-500", placeholder: "Name (e.g., Ash)", value: "{props.new_acc_name}", oninput: move |evt| props.new_acc_name.set(evt.value()) }
                    }
                    div { class: "flex flex-col gap-1.5",
                        input { class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white text-sm focus:border-blue-500", placeholder: "Friend ID", value: "{props.new_acc_id}", oninput: move |evt| props.new_acc_id.set(evt.value()) }
                    }
                    div { class: "flex items-center gap-3 mt-1",
                        input { r#type: "checkbox", class: "w-4 h-4 accent-orange-500 cursor-pointer", checked: *props.new_acc_is_main.read(), onchange: move |evt| props.new_acc_is_main.set(evt.value().parse().unwrap_or(false)) }
                        label { class: "text-xs text-gray-300 font-bold", "Set as Main Account" }
                    }
                    button {
                        class: "mt-2 bg-gray-700 hover:bg-gray-600 text-white font-bold py-2.5 px-4 rounded-lg transition-transform active:scale-95 text-sm",
                        onclick: move |_| {
                            let name = props.new_acc_name.read().clone();
                            let id = props.new_acc_id.read().clone();
                            let is_main = *props.new_acc_is_main.read();

                            if !name.is_empty() {
                                props.collection.write().accounts.push(Account { name: name.clone(), id, main: is_main });
                                props.new_acc_name.set(String::new());
                                props.new_acc_id.set(String::new());
                                props.new_acc_is_main.set(true);
                                
                                props.toast_message.set(Some(format!("✅ Account '{}' created!", name)));
                                let mut toast = props.toast_message.clone();
                                spawn(async move {
                                    gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                    toast.set(None);
                                });
                            }
                        },
                        "Add Account"
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct LoginModalProps {
    pub show_login_modal: Signal<bool>,
    pub user_email: Signal<String>,
    pub user_password: Signal<String>,
    pub auth_token: Signal<String>,
    pub sync_status: Signal<String>,
    pub collection: Signal<CardCollection>,
}

#[component]
pub fn LoginModal(mut props: LoginModalProps) -> Element {
    rsx! {
        if *props.show_login_modal.read() {
            div { class: "fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                div { class: "bg-gray-900 border border-gray-700 rounded-xl p-6 w-full max-w-md shadow-2xl flex flex-col gap-4",
                    h2 { class: "text-2xl font-black text-white", "Welcome to Pocket Binder" }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-gray-400 font-bold uppercase", "Email Address" }
                        input { r#type: "email", class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:border-blue-500", placeholder: "you@example.com", value: "{props.user_email}", oninput: move |evt| props.user_email.set(evt.value()) }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-gray-400 font-bold uppercase", "Password" }
                        input { r#type: "password", class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:border-blue-500", placeholder: "••••••••", value: "{props.user_password}", oninput: move |evt| props.user_password.set(evt.value()) }
                    }

                    if !props.sync_status.read().is_empty() {
                        div { class: "text-center text-sm font-mono mt-2 p-2 rounded bg-gray-800/80 text-orange-400 border border-gray-700", "{props.sync_status}" }
                    }

                    div { class: "flex gap-2 mt-4",
                        button {
                            class: "bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-4 rounded-lg flex-1 transition-transform active:scale-95",
                            onclick: move |_| {
                                let email = props.user_email.read().clone();
                                let pass = props.user_password.read().clone();

                                if !email.is_empty() && !pass.is_empty() {
                                    props.sync_status.set("🔄 Authenticating...".to_string());
                                    
                                    spawn(async move {
                                        match supabase_auth(&email, &pass, false).await {
                                            Ok(token) => {
                                                let _ = LocalStorage::set("user_email", &email);
                                                let _ = LocalStorage::set("supabase_token", &token);
                                                props.auth_token.set(token.clone());
                                                props.show_login_modal.set(false);
                                                
                                                props.sync_status.set("🔄 Downloading Binder...".to_string());
                                                if let Ok(data) = load_from_supabase(&token).await {
                                                    props.collection.set(data);
                                                    props.sync_status.set("✅ Loaded!".to_string());
                                                }
                                            },
                                            Err(e) => props.sync_status.set(format!("❌ {}", e)),
                                        }
                                    });
                                }
                            },
                            "Sign In"
                        }
                        
                        button {
                            class: "bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-4 rounded-lg flex-1 transition-transform active:scale-95",
                            onclick: move |_| {
                                let email = props.user_email.read().clone();
                                let pass = props.user_password.read().clone();

                                if !email.is_empty() && !pass.is_empty() {
                                    props.sync_status.set("🔄 Creating Account...".to_string());
                                    spawn(async move {
                                        match supabase_auth(&email, &pass, true).await {
                                            Ok(token) => {
                                                let _ = LocalStorage::set("user_email", &email);
                                                let _ = LocalStorage::set("supabase_token", &token);
                                                props.auth_token.set(token);
                                                props.show_login_modal.set(false);
                                                props.sync_status.set("✅ Account Created!".to_string());
                                            },
                                            Err(e) => props.sync_status.set(format!("❌ {}", e)),
                                        }
                                    });
                                }
                            },
                            "Sign Up"
                        }
                    }
                }
            }
        }
    }
}
