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
            div { class: "fixed inset-0 bg-slate-950/85 flex items-center justify-center p-4 z-50 backdrop-blur-sm",
                div { class: "bg-slate-900 border border-indigo-500/20 rounded-xl p-6 w-full max-w-md shadow-2xl flex flex-col gap-4 animate-fade-in-down",
                    h2 { class: "text-2xl font-black text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 via-purple-300 to-pink-400", "Welcome to Pocket Binder" }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-slate-400 font-bold uppercase", "Email Address" }
                        input { r#type: "email", class: "bg-slate-800/60 border border-indigo-500/20 rounded-lg px-4 py-2.5 text-white focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors", placeholder: "you@example.com", value: "{props.user_email}", oninput: move |evt| props.user_email.set(evt.value()) }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-slate-400 font-bold uppercase", "Password" }
                        input { r#type: "password", class: "bg-slate-800/60 border border-indigo-500/20 rounded-lg px-4 py-2.5 text-white focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400/30 outline-none transition-colors", placeholder: "••••••••", value: "{props.user_password}", oninput: move |evt| props.user_password.set(evt.value()) }
                    }

                    if !props.sync_status.read().is_empty() {
                        div { class: "text-center text-sm font-mono mt-2 p-2 rounded-lg bg-slate-800/60 text-indigo-400 border border-indigo-500/20", "{props.sync_status}" }
                    }

                    div { class: "flex gap-2 mt-4",
                        button {
                            class: "bg-gradient-to-r from-indigo-500 to-purple-500 hover:from-indigo-400 hover:to-purple-400 text-white font-bold py-3 px-4 rounded-lg flex-1 transition-all active:scale-95 shadow-[0_0_20px_rgba(99,102,241,0.2)]",
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
                            class: "bg-slate-700/80 hover:bg-slate-600/80 text-white font-bold py-3 px-4 rounded-lg flex-1 transition-all active:scale-95 border border-indigo-500/10",
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
