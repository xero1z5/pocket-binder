use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;

#[derive(PartialEq, Clone, Props)]
pub struct AddCardButtonProps {
    pub show_add_modal: Signal<bool>,
}

#[component]
pub fn AddCardButton(mut props: AddCardButtonProps) -> Element {
    rsx! {
        button {
            class: "group w-11 h-11 md:w-14 md:h-14 flex flex-col items-center justify-center bg-orange-600/20 border border-orange-500/50 rounded-xl md:rounded-2xl hover:bg-orange-500 hover:border-orange-400 transition-all shadow-lg shadow-orange-900/20",
            onclick: move |_| props.show_add_modal.set(true),
            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-6 h-6 md:w-7 md:h-7 text-orange-400 group-hover:text-white transition-colors",
                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
            }
        }
    }
}

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
                                            img { src: "{official_card.image}", loading: "lazy", decoding: "async", class: "w-full rounded border border-gray-600 shadow-md aspect-[63/88] object-cover" }
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
