use dioxus::prelude::*;
use std::collections::HashMap;
use gloo_storage::{LocalStorage, Storage};
use crate::models::*;
use crate::supabase::*;

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
                                                    props.sync_status.set("Last sync: Just now".to_string());
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
