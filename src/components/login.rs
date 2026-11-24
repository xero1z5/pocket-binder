use dioxus::prelude::*;
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
                div { class: "absolute inset-0 scrim animate-fade-in", onclick: move |_| props.show_login_modal.set(false) }
                div { class: "acrylic-strong relative rounded-[1.75rem] p-7 w-full max-w-md flex flex-col gap-4 animate-fade-in-down",
                    div { class: "flex items-center gap-3",
                        div { class: "brand-ball w-9 h-9 border-[3px]" }
                        h2 { class: "text-2xl font-semibold tracking-tight text-white", "Pocket Binder" }
                    }
                    p { class: "text-sm text-slate-400 -mt-1 mb-1", "Sign in to keep your binder synced across devices." }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "field-label", "Email" }
                        input { r#type: "email", class: "field", placeholder: "you@example.com", value: "{props.user_email}", oninput: move |evt| props.user_email.set(evt.value()) }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "field-label", "Password" }
                        input { r#type: "password", class: "field", placeholder: "••••••••", value: "{props.user_password}", oninput: move |evt| props.user_password.set(evt.value()) }
                    }

                    if !props.sync_status.read().is_empty() {
                        div { class: "text-center text-sm mt-1 px-3 py-2 rounded-xl bg-white/5 text-slate-200 border border-white/10", "{props.sync_status}" }
                    }

                    div { class: "flex gap-2 mt-3",
                        button {
                            class: "btn btn-primary flex-1",
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
                            "Sign in"
                        }
                        
                        button {
                            class: "btn btn-glass flex-1",
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
                            "Create account"
                        }
                    }
                }
            }
        }
    }
}
