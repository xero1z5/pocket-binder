use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::*;
use crate::models::optimized_image_url;

// --- 1. LOAD ASSETS AT COMPILE TIME ---
const DIAMOND_IMG: Asset = asset!("/assets/diamond.webp");
const STAR_IMG: Asset = asset!("/assets/star.webp");
const CROWN_IMG: Asset = asset!("/assets/crown.webp");
const SHINY_IMG: Asset = asset!("/assets/shiny-star.webp");

// --- 2. RARITY LOGIC & COMPONENT ---
pub fn get_rarity_data(code: &str) -> (&'static str, Option<Asset>, u8) {
    match code {
        "C" => ("Common", Some(DIAMOND_IMG), 1),
        "U" => ("Uncommon", Some(DIAMOND_IMG), 2),
        "R" => ("Rare", Some(DIAMOND_IMG), 3),
        "RR" => ("Double Rare", Some(DIAMOND_IMG), 4),
        "AR" => ("Art Rare", Some(STAR_IMG), 1),
        "SR" => ("Super Rare", Some(STAR_IMG), 2),
        "SAR" => ("Special Art Rare", Some(STAR_IMG), 2),
        "IM" => ("Immersive Rare", Some(STAR_IMG), 3),
        "UR" => ("Crown Rare", Some(CROWN_IMG), 1),
        "S" => ("Shiny", Some(SHINY_IMG), 1),
        "SSR" => ("Shiny Super Rare", Some(SHINY_IMG), 2),
        "PROMO" => ("Promo", None, 0),
        _ => ("Unknown", None, 0),
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct RarityDisplayProps {
    pub rarity_code: String,
}

#[component]
pub fn RarityDisplay(props: RarityDisplayProps) -> Element {
    let (label, image_asset, count) = get_rarity_data(&props.rarity_code);

    if count == 0 || image_asset.is_none() {
        return rsx! {
            span { class: "text-[10px] font-semibold text-slate-200", "{label}" }
        };
    }

    rsx! {
        div { class: "flex items-center justify-center gap-0.5", title: "{label}",
            for _ in 0..count {
                img { 
                    src: image_asset.clone().unwrap(),
                    class: "h-3.5 w-auto object-contain",
                    alt: "{label}"
                }
            }
        }
    }
}

// --- THE MODAL ---
#[derive(PartialEq, Clone, Props)]
pub struct AddCardModalProps {
    pub show_add_modal: Signal<bool>,
    pub collection: Signal<CardCollection>,
    pub image_db: Signal<Option<HashMap<String, OfficialCard>>>,
    pub toast_message: Signal<Option<String>>,
}

#[component]
pub fn AddCardModal(mut props: AddCardModalProps) -> Element {
    if !*props.show_add_modal.read() {
        return rsx! {};
    }

    let mut selected_card_to_add = use_signal(|| None::<OfficialCard>);
    let mut show_account_list = use_signal(|| false);
    let mut search_input = use_signal(|| String::new());
    // Debounced query so we don't filter the entire card database on every keystroke.
    let mut debounced_query = use_signal(|| String::new());
    let mut search_token = use_signal(|| 0u32);

    // Multi-select state is LOCAL to this modal so it never touches the
    // collection grid's selection (independent menus).
    let mut local_mass_select = use_signal(|| false);
    let mut local_mass_cards = use_signal(|| Vec::<String>::new());

    // Account picker for bulk-add action bar
    let mut show_add_account_picker = use_signal(|| false);

    // Reset account picker when multi-select is turned off
    use_effect(move || {
        if !*local_mass_select.read() {
            show_add_account_picker.set(false);
        }
    });

    // Local filter state — isolated from the main collection filters
    let mut local_rarities = use_signal(|| Vec::<String>::new());
    let mut local_types   = use_signal(|| Vec::<String>::new());
    let mut show_local_filters = use_signal(|| false);

    // Memoized filtered results — live search as-you-type (driven by the debounced query)
    let filtered_api_cards = use_memo(move || {
        let query = debounced_query.read().trim().to_lowercase();

        if let Some(api_map) = &*props.image_db.read() {
            let selected_r = local_rarities.read().clone();
            let selected_t = local_types.read().clone();

            // Only show cards when a search/filter is active (something typed or filters chosen).
            // Mass-select also relies on this, so it never renders the full list.
            let has_filters = !query.is_empty() || !selected_r.is_empty() || !selected_t.is_empty();
            if !has_filters {
                return Vec::new();
            }

            let mut results: Vec<OfficialCard> = api_map.values()
                .filter(|c| {
                    let matches_q = query.is_empty() || c.name.to_lowercase().contains(&query);
                    let matches_r = selected_r.is_empty() || selected_r.contains(&c.rarity);
                    let matches_t = selected_t.is_empty() || selected_t.contains(&c.card_type);
                    matches_q && matches_r && matches_t
                })
                .cloned()
                .collect();
            results.sort_by(|a, b| a.set.cmp(&b.set).then(a.number.cmp(&b.number)));
            results
        } else {
            Vec::new()
        }
    });


    rsx! {
        if *props.show_add_modal.read() {
            // Semi-transparent overlay so the background collection is still visible
            div { class: "fixed inset-0 bg-slate-950/90 flex flex-col z-50 animate-fade-in",
                div { class: "acrylic border-x-0 border-t-0 px-3 md:px-6 py-3 flex flex-col gap-3 z-10",
                    div { class: "flex items-center justify-between pl-1",
                        h2 { class: "text-lg font-semibold tracking-tight text-white", "Add cards" }
                        button {
                            class: "icon-btn is-bare",
                            title: "Close",
                            onclick: move |_| { props.show_add_modal.set(false); selected_card_to_add.set(None); show_account_list.set(false); local_mass_select.set(false); local_mass_cards.set(Vec::new()); search_input.set(String::new()); debounced_query.set(String::new()); }, 
                            svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" } }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        div { class: "relative group flex-1",
                            div { class: "absolute inset-y-0 left-0 flex items-center pl-3.5 pointer-events-none text-slate-500 group-focus-within:text-accent transition-colors",
                                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor", class: "w-4 h-4",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" }
                                }
                            }
                            input { 
                                class: "field pl-10",
                                r#type: "search",
                                placeholder: if *local_mass_select.read() { "Search cards to select" } else { "Search by card name" },
                                value: "{search_input}", 
                                oninput: move |evt| {
                                    let val = evt.value();
                                    search_input.set(val.clone());
                                    selected_card_to_add.set(None);
                                    show_account_list.set(false);
                                    // Debounce the heavy DB filter so typing stays responsive
                                    search_token.with_mut(|t| *t += 1);
                                    let token = *search_token.read();
                                    let mut debounced_query = debounced_query;
                                    spawn(async move {
                                        gloo_timers::future::sleep(std::time::Duration::from_millis(160)).await;
                                        if *search_token.read() == token {
                                            debounced_query.set(val);
                                        }
                                    });
                                },
                            }
                        }

                        // Multi-select toggle
                        button {
                            class: "icon-btn",
                            class: if *local_mass_select.read() { "is-on" },
                            title: if *local_mass_select.read() { "Stop selecting" } else { "Select several cards" },
                            onclick: move |_| {
                                let curr = *local_mass_select.read();
                                local_mass_select.set(!curr);
                                if curr { local_mass_cards.set(Vec::new()); }
                            },
                            if *local_mass_select.read() {
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2.5", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" }
                                }
                            } else {
                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }
                                }
                            }
                        }

                        // Local filter button — does NOT affect the main collection view
                        button {
                            class: "icon-btn",
                            class: if *show_local_filters.read() || !local_rarities.read().is_empty() || !local_types.read().is_empty() { "is-on" },
                            title: "Filter by rarity or type",
                            onclick: move |_| { let v = *show_local_filters.read(); show_local_filters.set(!v); },
                            svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor", class: "w-5 h-5",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" }
                            }
                        }
                    }

                    // Local filter chips (does NOT touch global collection filters)
                    if *show_local_filters.read() {
                        div { class: "flex flex-col gap-2.5 pt-1 pb-1 animate-fade-in-down",
                            div { class: "flex flex-wrap gap-1.5 items-center",
                                span { class: "field-label w-12", "Rarity" }
                                for rarity in ["C", "U", "R", "RR", "AR", "SR", "SAR", "IM", "UR", "S", "SSR", "PROMO"] {
                                    {
                                        let r = rarity.to_string();
                                        let is_active = local_rarities.read().contains(&r);
                                        rsx! {
                                            button {
                                                class: "chip",
                                                class: if is_active { "is-on" },
                                                onclick: move |_| {
                                                    let mut v = local_rarities.read().clone();
                                                    if v.contains(&r) { v.retain(|x| x != &r); } else { v.push(r.clone()); }
                                                    local_rarities.set(v);
                                                },
                                                RarityDisplay { rarity_code: r.clone() }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "flex flex-wrap gap-1.5 items-center",
                                span { class: "field-label w-12", "Type" }
                                for card_type in ["Pokémon", "Trainer"] {
                                    {
                                        let t = card_type.to_string();
                                        let is_active = local_types.read().contains(&t);
                                        rsx! {
                                            button {
                                                class: "chip",
                                                class: if is_active { "is-on" },
                                                onclick: move |_| {
                                                    let mut v = local_types.read().clone();
                                                    if v.contains(&t) { v.retain(|x| x != &t); } else { v.push(t.clone()); }
                                                    local_types.set(v);
                                                },
                                                "{card_type}"
                                            }
                                        }
                                    }
                                }
                                if !local_rarities.read().is_empty() || !local_types.read().is_empty() {
                                    button {
                                        class: "text-link ml-2",
                                        onclick: move |_| { local_rarities.set(Vec::new()); local_types.set(Vec::new()); },
                                        "Clear filters"
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "flex-1 overflow-y-auto p-4 relative",
                    if let Some(card) = selected_card_to_add.read().clone() {
                        {
                            let packs_text = if card.packs.is_empty() { "Promo".to_string() } else { card.packs.join(", ") };
                            
                            rsx! {
                                // Side-by-side layout: card on left, actions on right
                                div { class: "flex flex-col md:flex-row items-center md:items-start justify-center gap-6 md:gap-10 h-full max-w-2xl mx-auto py-4",
                                    
                                    // LEFT: Card Image + Metadata
                                    div { class: "flex flex-col items-center gap-3 flex-shrink-0",
                                        img { 
                                            src: "{optimized_image_url(&card.full_image_url, 600)}", 
                                            class: "w-40 md:w-60 rounded-[4.5%/3.2%] shadow-[0_24px_60px_rgba(2,6,18,0.6)]"  
                                        }
                                        
                                        // Card info below image
                                        div { class: "flex flex-col items-center gap-1.5 mt-1",
                                            h3 { class: "text-xl font-semibold tracking-tight text-white text-center", "{card.name}" }
                                            RarityDisplay { rarity_code: card.rarity.clone() }
                                            span { class: "text-slate-400 text-xs", "{packs_text} ({card.set})" }
                                        }
                                    }

                                    // RIGHT: Actions
                                    div { class: "flex flex-col gap-3 w-full md:w-64 md:pt-2",
                                        {
                                            let is_wishlisted = props.collection.read().is_wishlisted(&card.generated_id);
                                            let c = card.clone();
                                            rsx! {
                                                button {
                                                    class: "btn w-full group",
                                                    class: if is_wishlisted { "bg-[#ff5a7a]/15 border-[#ff5a7a]/40 text-[#ff8fa6]" } else { "btn-glass" },
                                                    onclick: move |_| {
                                                        let card_to_add = Card { 
                                                            id: c.generated_id.clone(), 
                                                            name: c.name.clone(), 
                                                            rarity: c.rarity.clone(),
                                                            card_type: c.card_type.clone(), 
                                                            pack: if c.packs.is_empty() { "Promo".to_string() } else { c.packs.join(", ") }
                                                        };
                                                        props.collection.write().toggle_wishlist(card_to_add);
                                                    },
                                                    svg { class: "w-5 h-5 flex-shrink-0", class: if is_wishlisted { "fill-current" } else { "fill-none text-slate-400 group-hover:text-[#ff8fa6]" }, view_box: "0 0 24 24", stroke_width: "1.5", stroke: "currentColor",
                                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                                    }
                                                    span { if is_wishlisted { "On your wishlist" } else { "Add to wishlist" } }
                                                }
                                            }
                                        }

                                        if !*show_account_list.read() {
                                            button {
                                                class: "btn btn-primary w-full",
                                                onclick: move |_| show_account_list.set(true),
                                                svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                                                }
                                                span { "Add to collection" }
                                            }
                                        }

                                        if *show_account_list.read() {
                                            div { class: "flex flex-col animate-fade-in-down w-full",
                                                div { class: "flex items-center justify-between mb-2 mt-1 px-1",
                                                    span { class: "field-label", "Choose an account" }
                                                    button {
                                                        class: "text-link",
                                                        onclick: move |_| show_account_list.set(false),
                                                        "Cancel"
                                                    }
                                                }
                                                div { class: "flex flex-col gap-1.5",
                                                    for acc in props.collection.read().accounts.iter() {
                                                        {
                                                            let c = card.clone(); 
                                                            let target_acc = acc.name.clone();
                                                            let is_main = acc.main;
                                                            rsx! {
                                                                button {
                                                                    class: "row group",
                                                                    onclick: move |_| {
                                                                        let card_to_add = Card { 
                                                                            id: c.generated_id.clone(), 
                                                                            name: c.name.clone(), 
                                                                            rarity: c.rarity.clone(),
                                                                            card_type: c.card_type.clone(), 
                                                                            pack: if c.packs.is_empty() { "Promo".to_string() } else { c.packs.join(", ") }
                                                                        };
                                                                        props.collection.write().add_card(card_to_add, &target_acc, 1);
                                                                        
                                                                        props.toast_message.set(Some(format!("Added to {}", target_acc)));
                                                                        let mut t = props.toast_message.clone(); 
                                                                        spawn(async move { 
                                                                            gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await; 
                                                                            t.set(None); 
                                                                        });
                                                                        selected_card_to_add.set(None);
                                                                        show_account_list.set(false);
                                                                    },
                                                                    span { class: "font-medium", "{acc.name}" }
                                                                    if is_main {
                                                                        span { class: "text-[11px] font-medium text-accent", "Main" }
                                                                    }
                                                                    svg { class: "w-4 h-4 text-slate-500 group-hover:text-accent ml-auto transition-colors", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        button { 
                                            class: "btn btn-glass w-full mt-2",  
                                            onclick: move |_| {
                                                selected_card_to_add.set(None);
                                                show_account_list.set(false);
                                            }, 
                                            "Back to results"  
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        if filtered_api_cards.read().is_empty() {
                            div { class: "flex flex-col items-center text-center mt-20 gap-2",
                                if search_input.read().trim().is_empty() && local_rarities.read().is_empty() && local_types.read().is_empty() {
                                    span { class: "text-base font-medium text-white", "Find a card" }
                                    span { class: "text-sm text-slate-400", "Search by name, or filter by rarity or type." }
                                } else {
                                    span { class: "text-base font-medium text-white", "No cards match" }
                                    span { class: "text-sm text-slate-400", "Check the spelling or clear the filters." }
                                }
                            }
                        }
                        div { class: "grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 gap-x-3 gap-y-5 md:gap-x-5 max-w-6xl mx-auto",
                            for api_card in filtered_api_cards() {
                                {
                                    let c = api_card.clone();
                                    let packs_display = if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") };

                                    let is_selected = local_mass_cards.read().contains(&c.generated_id);
                                    rsx! {
                                        div { 
                                            class: "card-tile",
                                            class: if is_selected { "is-selected" },
                                            onclick: move |_| {
                                                if *local_mass_select.read() {
                                                    let mut current = local_mass_cards.read().clone();
                                                    if current.contains(&c.generated_id) {
                                                        current.retain(|id| id != &c.generated_id);
                                                    } else {
                                                        current.push(c.generated_id.clone());
                                                    }
                                                    local_mass_cards.set(current);
                                                } else {
                                                    selected_card_to_add.set(Some(c.clone()));
                                                }
                                            },
                                            div { class: "card-art",
                                                img {
                                                    src: "{optimized_image_url(&api_card.full_image_url, 400)}",
                                                    alt: "{api_card.name}",
                                                    loading: "lazy", decoding: "async",
                                                    width: "400", height: "560",
                                                }
                                                // Selection indicator
                                                if is_selected {
                                                    div { class: "mark mark-select is-on absolute top-1.5 right-1.5",
                                                        svg { class: "w-3.5 h-3.5", fill: "none", view_box: "0 0 24 24", stroke_width: "3.5", stroke: "currentColor", path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" } }
                                                    }
                                                }
                                            }
                                            div { class: "card-name", "{api_card.name}" }
                                            div { class: "flex justify-center mt-1",
                                                RarityDisplay { rarity_code: api_card.rarity.clone() }
                                            }
                                            div { class: "text-[10px] text-center text-slate-500 truncate mt-0.5", "{packs_display} ({api_card.set})" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Floating multi-select dock — rendered outside the dark overlay so it has no
            // dark container behind it (floaty feel), matching the collection's MassActionBar.
            {
                let local_count = local_mass_cards.read().len();
                if !*local_mass_select.read() {
                    rsx! { Fragment { } }
                } else if local_count == 0 {
                    rsx! {
                        div { class: "fixed bottom-4 md:bottom-6 inset-x-0 z-[60] flex justify-center px-3 pointer-events-none",
                            div { class: "acrylic pointer-events-auto rounded-[1.75rem] pl-5 pr-1.5 py-1.5 flex items-center gap-3 animate-slide-up",
                                span { class: "text-sm text-slate-300", "Tap cards to select them" }
                                button {
                                    class: "icon-btn is-bare is-danger",
                                    title: "Stop selecting",
                                    onclick: move |_| { local_mass_cards.set(Vec::new()); local_mass_select.set(false); show_add_account_picker.set(false); },
                                    svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let accounts = props.collection.read().accounts.clone();
                    let has_accounts = !accounts.is_empty();
                    rsx! {
                        div { class: "fixed bottom-4 md:bottom-6 inset-x-0 z-[60] flex justify-center px-3 pointer-events-none",
                            div { class: "acrylic relative pointer-events-auto rounded-[1.75rem] pl-5 pr-1.5 py-1.5 flex items-center gap-1.5 md:gap-2",
                                span { class: "text-sm font-medium text-white mr-2 whitespace-nowrap", "{local_count} selected" }
                                button {
                                    class: "icon-btn",
                                    class: if *show_add_account_picker.read() { "is-on" },
                                    title: "Add to an account",
                                    onclick: move |_| {
                                        let curr = *show_add_account_picker.read();
                                        show_add_account_picker.set(!curr);
                                    },
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                    }
                                }
                                button {
                                    class: "icon-btn is-wish",
                                    title: "Add to wishlist",
                                    onclick: move |_| {
                                        let cards = local_mass_cards.read().clone();
                                        if let Some(db) = props.image_db.read().as_ref() {
                                            let mut updated = props.collection.read().clone();
                                            let mut added = 0;
                                            for id in cards.iter() {
                                                if let Some(api_card) = db.get(id) {
                                                    let c = Card {
                                                        id: api_card.generated_id.clone(),
                                                        name: api_card.name.clone(),
                                                        rarity: api_card.rarity.clone(),
                                                        card_type: api_card.card_type.clone(),
                                                        pack: if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") }
                                                    };
                                                    if !updated.is_wishlisted(&c.id) {
                                                        updated.toggle_wishlist(c);
                                                        added += 1;
                                                    }
                                                }
                                            }
                                            if added > 0 {
                                                props.collection.set(updated);
                                                props.toast_message.set(Some(format!("Added {} cards to Wishlist", added)));
                                                local_mass_cards.set(Vec::new());
                                                local_mass_select.set(false);
                                            } else {
                                                props.toast_message.set(Some("Cards were already wishlisted".to_string()));
                                            }
                                        }
                                    },
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" }
                                    }
                                }
                                button {
                                    class: "icon-btn is-trade",
                                    title: "Mark as tradable",
                                    onclick: move |_| {
                                        let cards = local_mass_cards.read().clone();
                                        let mut updated = props.collection.read().clone();
                                        let mut changed = 0;
                                        for id in cards.iter() {
                                            if !updated.is_tradable(id) {
                                                updated.toggle_tradable(id);
                                                changed += 1;
                                            }
                                        }
                                        if changed > 0 {
                                            props.collection.set(updated);
                                            props.toast_message.set(Some(format!("Added {} cards to Tradable", changed)));
                                            local_mass_cards.set(Vec::new());
                                            local_mass_select.set(false);
                                        } else {
                                            props.toast_message.set(Some("Cards were already tradable".to_string()));
                                        }
                                    },
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" }
                                    }
                                }
                                button {
                                    class: "icon-btn is-bare is-danger",
                                    title: "Stop selecting",
                                    onclick: move |_| { local_mass_cards.set(Vec::new()); local_mass_select.set(false); show_add_account_picker.set(false); },
                                    svg { class: "w-5 h-5", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                                    }
                                }
                                if *show_add_account_picker.read() {
                                    div { class: "acrylic-strong absolute bottom-full right-0 mb-3 w-64 rounded-3xl p-2 animate-fade-in-up",
                                        h4 { class: "text-xs font-medium text-slate-400 px-3 pt-2 pb-2", "Add to account" }
                                        div { class: "flex flex-col gap-1 max-h-56 overflow-y-auto",
                                            for acc in props.collection.read().accounts.iter() {
                                                {
                                                    let acc_name = acc.name.clone();
                                                    let cards_to_add = local_mass_cards.read().clone();
                                                    rsx! {
                                                        button {
                                                            class: "row is-plain font-medium",
                                                            onclick: move |_| {
                                                                if let Some(db) = props.image_db.read().as_ref() {
                                                                    let mut updated = props.collection.read().clone();
                                                                    for id in cards_to_add.iter() {
                                                                        if let Some(api_card) = db.get(id) {
                                                                            let c = Card {
                                                                                id: api_card.generated_id.clone(),
                                                                                name: api_card.name.clone(),
                                                                                rarity: api_card.rarity.clone(),
                                                                                card_type: api_card.card_type.clone(),
                                                                                pack: if api_card.packs.is_empty() { "Promo".to_string() } else { api_card.packs.join(", ") }
                                                                            };
                                                                            updated.add_card(c, &acc_name, 1);
                                                                        }
                                                                    }
                                                                    props.collection.set(updated);
                                                                    props.toast_message.set(Some(format!("Added {} cards to {}", cards_to_add.len(), acc_name)));
                                                                    let mut t = props.toast_message.clone();
                                                                    spawn(async move { gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await; t.set(None); });
                                                                    show_add_account_picker.set(false);
                                                                    local_mass_cards.set(Vec::new());
                                                                    local_mass_select.set(false);
                                                                }
                                                            },
                                                            "{acc_name}"
                                                        }
                                                    }
                                                }
                                            }
                                            if !has_accounts {
                                                div { class: "text-center text-slate-400 text-xs px-3 py-3", "No accounts yet. Add one from the Account menu." }
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
