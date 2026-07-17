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
            div { class: "fixed inset-0 flex items-center justify-center p-4 z-50",
                div { class: "absolute inset-0 bg-slate-950/60 backdrop-blur-sm animate-fade-in", onclick: move |_| props.show_login_modal.set(false) }
                div { class: "glass-panel rounded-xl p-6 w-full max-w-md flex flex-col gap-4 animate-fade-in-down",
                    h2 { class: "text-2xl font-bold text-white", "Welcome to Pocket Binder" }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-slate-400 font-bold uppercase", "Email Address" }
                        input { r#type: "email", class: "bg-white/5 border border-white/10 rounded-lg px-4 py-2.5 text-white focus:border-white/30 focus:ring-1 focus:ring-white/30/30 outline-none transition-all", placeholder: "you@example.com", value: "{props.user_email}", oninput: move |evt| props.user_email.set(evt.value()) }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-slate-400 font-bold uppercase", "Password" }
                        input { r#type: "password", class: "bg-white/5 border border-white/10 rounded-lg px-4 py-2.5 text-white focus:border-white/30 focus:ring-1 focus:ring-white/30/30 outline-none transition-all", placeholder: "••••••••", value: "{props.user_password}", oninput: move |evt| props.user_password.set(evt.value()) }
                    }

                    if !props.sync_status.read().is_empty() {
                        div { class: "text-center text-sm font-mono mt-2 p-2 rounded-lg bg-slate-800/60 text-white border border-white/30/20", "{props.sync_status}" }
                    }

                    div { class: "flex gap-2 mt-4",
                        button {
                            class: "bg-emerald-500/80 hover:bg-emerald-500 text-white font-bold py-3 px-4 rounded-xl flex-1 transition-all shadow-[0_0_15px_rgba(16,185,129,0.2)] hover:shadow-[0_0_20px_rgba(16,185,129,0.4)] border border-emerald-400/50",
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
                            class: "bg-sky-500/80 hover:bg-sky-500 text-white font-bold py-3 px-4 rounded-xl flex-1 transition-all active:scale-95 shadow-[0_0_15px_rgba(14,165,233,0.2)] hover:shadow-[0_0_20px_rgba(14,165,233,0.4)] border border-sky-400/50",
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
